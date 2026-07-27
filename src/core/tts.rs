use kokoro_micro::TtsEngine;
use rodio::{Player, buffer::SamplesBuffer, nz};
use tokio::sync::mpsc::{self, Sender};

pub struct TTS {}

impl TTS {
    pub async fn run() -> Sender<String> {
        let (tx, mut rx) = mpsc::channel::<String>(10);
        tokio::spawn(async move {
            let mut tts = TtsEngine::new().await.expect("TTS failed to load");
            let sink_handle = rodio::DeviceSinkBuilder::open_default_sink()
                .expect("Failed to open default audio stream");

            while let Some(message) = rx.recv().await {
                let audio = tts
                    .synthesize_with_options(
                        &message,
                        None,       // voice: None = default "af_sky"
                        1.0,        // speed: 1.0 = normal
                        1.0,        // gain: 1.0 = normal volume
                        Some("en"), // language
                    )
                    .expect("Failed to synthesize audio");
                let source = SamplesBuffer::new(
                    nz!(1),     // ChannelCount
                    nz!(24000), // SampleRate
                    audio,      // Your Vec<f32>
                );
                let player = Player::connect_new(&sink_handle.mixer());
                player.append(source);
                player.sleep_until_end();
            }
            println!("TTS dropped");
        });
        tx
    }
}
