use anyhow::{Result, bail};
use comms::{CameraFrame, Comms, RG24_FOURCC, topics::CameraFrames};
use image::{DynamicImage, RgbImage};
use object_detector::{DetectorType, ObjectDetector};

#[tokio::main]
async fn main() -> Result<()> {
    let comms = Comms::open().await?;
    let subscriber = comms.subscriber::<CameraFrames>().await?;

    println!("Loading model...");
    let detector = ObjectDetector::from_hf(DetectorType::PromptFree)
        .build()
        .await?;
    println!("Model loaded");

    loop {
        let frame = subscriber.recv().await?;
        let image = image_from_frame(&frame)?;
        println!(
            "Received frame {} ({}x{})",
            frame.sequence,
            image.width(),
            image.height()
        );

        let image = DynamicImage::ImageRgb8(image);
        let results = detector.predict(&image).call()?;

        for det in results {
            println!("[{:>10}] Score: {:.4}", det.tag, det.score);
        }
    }
}

fn image_from_frame(frame: &CameraFrame) -> Result<RgbImage> {
    if frame.fourcc != RG24_FOURCC {
        bail!("unsupported camera pixel format: {:#x}", frame.fourcc);
    }

    RgbImage::from_raw(frame.width, frame.height, frame.data.clone())
        .ok_or_else(|| anyhow::anyhow!("camera frame has invalid RGB24 data length"))
}
