use anyhow::Result;
use earshot::Detector;
use zenoh::Session;

use crate::mic_driver::{self, AudioStream, save_wav};

/// Mic capture is not implemented yet (no driver wired). Placeholder task so the
/// `robot/mic` topic slot exists and the controller keeps its one-task-per-topic shape.
pub async fn run(_session: Session) -> Result<()> {
    let mut stream = AudioStream::new().expect("Fail to capture audio stream");

    println!("Capturing audio — 10 chunks of 256 samples at 16 kHz...");
    let mut detector = Detector::default();
    let mut recording = Vec::new();
    let mut i = 0;
    loop {
        let chunk = stream.next_chunk().unwrap();
        assert_eq!(chunk.len(), 256);

        recording.extend(chunk.iter().copied());

        let score = detector.predict_i16(&chunk);
        // Score is between 0-1; 0 = no voice, 1 = voice.
        let voice = match score >= 0.5 {
            true => "voice",
            false => "nothing",
        };
        let rms: f32 =
            (chunk.iter().map(|&s| (s as f32).powi(2)).sum::<f32>() / chunk.len() as f32).sqrt();
        println!(
            "chunk {:>2}  |  len={}  |  RMS={:.6}  |  first={:+}  last={:+} | {}",
            i + 1,
            chunk.len(),
            rms,
            chunk.first().unwrap(),
            chunk.last().unwrap(),
            voice,
        );
        i += 1;

        if i >= 1000 {
            break;
        }
    }

    let path = mic_driver::DEFAULT_OUTPUT_FILE;
    save_wav(&recording, Some(path))?;
    println!("Saved {} samples to {}", recording.len(), path);
    Ok(())
}
