use kokoro_micro::TtsEngine;
use tokio::sync::mpsc::{self, Sender};
use zenoh::{Session, pubsub::Publisher};

pub struct TTS {}

impl TTS {
    pub async fn run(session: Session) -> Sender<String> {
        let (tx, mut rx) = mpsc::channel::<String>(10);
        tokio::spawn(async move {
            let mut tts = TtsEngine::new().await.expect("TTS failed to load");
            let speaker: Publisher<'static> = session
                .declare_publisher(comms::keys::SPEAKER)
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
                    .put(comms::audio::encode(&audio))
                    .await
                    .expect("Failed to publish audio");
            }
        });
        tx
    }
}
