use std::time::Duration;

use anyhow::Result;
use argos::comms;
use libcamera::{
    camera::CameraConfigurationStatus,
    camera_manager::CameraManager,
    framebuffer::AsFrameBuffer,
    framebuffer_allocator::{FrameBuffer, FrameBufferAllocator},
    framebuffer_map::MemoryMappedFrameBuffer,
    pixel_format::PixelFormat,
    properties,
    request::ReuseFlag,
    stream::StreamRole,
};
use zenoh::{Session, bytes::ZBytes};

// drm-fourcc does not have RG24 type yet, construct it from raw fourcc identifier
const PIXEL_FORMAT_RG24: PixelFormat =
    PixelFormat::new(u32::from_le_bytes([b'R', b'G', b'2', b'4']), 0);

pub async fn run(session: Session) -> Result<()> {
    let publisher = session
        .declare_publisher(comms::keys::CAMERA)
        .congestion_control(zenoh::qos::CongestionControl::Drop)
        .await
        .map_err(anyhow::Error::msg)?;

    // libcamera is blocking/callback based, so capture runs on a dedicated thread
    // and forwards encoded frames here. A small bounded channel drops stale frames.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ZBytes>(4);
    std::thread::spawn(move || capture_loop(tx));

    while let Some(payload) = rx.recv().await {
        publisher.put(payload).await.map_err(anyhow::Error::msg)?;
    }
    Ok(())
}

fn capture_loop(tx: tokio::sync::mpsc::Sender<ZBytes>) {
    let mgr = CameraManager::new().expect("Failed to create CameraManager");
    let cameras = mgr.cameras();
    let cam = cameras.get(0).expect("No cameras found");

    println!(
        "Using camera: {}",
        *cam.properties().get::<properties::Model>().unwrap()
    );

    let mut cam = cam.acquire().expect("Unable to acquire camera");

    // This will generate default configuration for each specified role
    let mut cfgs = cam
        .generate_configuration(&[StreamRole::VideoRecording])
        .unwrap();

    cfgs.get_mut(0).unwrap().set_pixel_format(PIXEL_FORMAT_RG24);

    match cfgs.validate() {
        CameraConfigurationStatus::Valid => {}
        CameraConfigurationStatus::Adjusted => {
            println!("Camera configuration was adjusted")
        }
        CameraConfigurationStatus::Invalid => panic!("Error validating camera configuration"),
    }

    // Ensure that pixel format was unchanged
    assert_eq!(
        cfgs.get(0).unwrap().get_pixel_format(),
        PIXEL_FORMAT_RG24,
        "RG24 is not supported by the camera"
    );

    let size = cfgs.get(0).unwrap().get_size();
    let (width, height) = (size.width, size.height);

    cam.configure(&mut cfgs).expect("Unable to configure camera");

    let mut alloc = FrameBufferAllocator::new(&cam);

    // Allocate frame buffers for the stream
    let cfg = cfgs.get(0).unwrap();
    let stream = cfg.stream().unwrap();
    let buffers = alloc.alloc(&stream).unwrap();

    // Convert FrameBuffer to MemoryMappedFrameBuffer, which allows reading &[u8]
    let buffers = buffers
        .into_iter()
        .map(|buf| MemoryMappedFrameBuffer::new(buf).unwrap())
        .collect::<Vec<_>>();

    // Create capture requests and attach buffers
    let reqs = buffers
        .into_iter()
        .enumerate()
        .map(|(i, buf)| {
            let mut req = cam.create_request(Some(i as u64)).unwrap();
            req.add_buffer(&stream, buf).unwrap();
            req
        })
        .collect::<Vec<_>>();

    // Completed capture requests are returned as a callback
    let (req_tx, req_rx) = std::sync::mpsc::channel();
    cam.on_request_completed(move |req| {
        req_tx.send(req).unwrap();
    });

    // TODO: Set `Control::FrameDuration()` here. Blocked on https://github.com/lit-robotics/libcamera-rs/issues/2
    cam.start(None).unwrap();

    // Enqueue all requests to the camera
    for req in reqs {
        cam.queue_request(req).map_err(|(_, e)| e).unwrap();
    }

    let mut seq: u32 = 0;
    loop {
        // Allow extra time for slower pipelines/first frame startup.
        let mut req = req_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Camera request failed");

        // Get framebuffer for our stream
        let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> = req.buffer(&stream).unwrap();

        // RG24 format has only one data plane containing the encoded data with all the headers
        let planes = framebuffer.data();
        let frame_data = planes.first().unwrap();
        // Actual encoded data will be smaller than framebuffer size, its length can be obtained from metadata.
        let bytes_used = framebuffer
            .metadata()
            .unwrap()
            .planes()
            .get(0)
            .unwrap()
            .bytes_used as usize;

        let header = comms::camera::Header {
            width,
            height,
            fourcc: comms::camera::RG24_FOURCC,
            seq,
        };
        let payload = comms::camera::encode(&header, &frame_data[..bytes_used]);
        // Drop the frame if the publisher is behind rather than stalling capture.
        let _ = tx.try_send(payload);
        seq = seq.wrapping_add(1);

        // Recycle the request back to the camera for execution
        req.reuse(ReuseFlag::REUSE_BUFFERS);
        cam.queue_request(req).map_err(|(_, e)| e).unwrap();
    }
}
