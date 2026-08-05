use comms::{AudioFrame, Comms, topics::SpeakerAudio};
use kokoro_micro::TtsEngine;
use tokio::sync::mpsc::{self, Sender};

pub struct TTS {}

impl TTS {
    pub async fn run(comms: Comms) -> Sender<String> {
        let (tx, mut rx) = mpsc::channel::<String>(10);
        tokio::spawn(async move {
            let mut tts = TtsEngine::new().await.expect("TTS failed to load");
            let speaker = comms
                .publisher::<SpeakerAudio>()
                .await
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

                speaker
                    .send(AudioFrame { samples: audio })
                    .await
                    .expect("Failed to publish audio");
            }
        });
        tx
    }
}
