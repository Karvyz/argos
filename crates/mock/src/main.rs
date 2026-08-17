use std::{env, path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use comms::{CameraFrame, Comms, RG24_FOURCC, topics::CameraFrames};
use tokio::time::{MissedTickBehavior, interval};

#[tokio::main]
async fn main() -> Result<()> {
	let image_path = env::args_os()
		.nth(1)
		.map(PathBuf::from)
		.context("usage: mock <image-path>")?;

	let image = image::ImageReader::open(&image_path)
		.with_context(|| format!("failed to open image: {}", image_path.display()))?
		.decode()
		.with_context(|| format!("failed to decode image: {}", image_path.display()))?
		.to_rgb8();

	let frame = CameraFrame {
		width: image.width(),
		height: image.height(),
		fourcc: RG24_FOURCC,
		sequence: 0,
		data: image.into_raw(),
	};

	let comms = Comms::open().await?;
	let publisher = comms.publisher::<CameraFrames>().await?;
	let mut timer = interval(Duration::from_secs_f64(1.0 / 30.0));
	timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
	let mut sequence = 0;

	loop {
		timer.tick().await;
		let mut next_frame = frame.clone();
		next_frame.sequence = sequence;
		publisher.send(next_frame).await?;
		sequence = sequence.wrapping_add(1);
        println!("Sent image {sequence}");
	}
}
