use colored::Colorize;
use comms::{AudioFrame, Comms, Subscriber, topics::Voice};
use rig_core::message::{Audio, Message};
use tokio::sync::mpsc::Sender;

use crate::{argos::Action, core::llm::LLM};

mod llm;
mod tts;

enum CmdRes {
    Ok(String),
    None,
    Exit,
}

pub struct Core {
    llm: LLM,
    tx: Sender<Action>,
    voice: Subscriber<Voice>,
}

impl Core {
    pub async fn new(url: &str, tx: Sender<Action>, comms: Comms) -> Self {
        Core {
            llm: LLM::new(url, tx.clone(), comms.clone()).await,
            tx,
            voice: comms
                .subscriber::<Voice>()
                .await
                .expect("Failed to subscribe to voice"),
        }
    }

    pub async fn run(&mut self) {
        loop {
            let message: Message = tokio::select! {
                result = self.voice.recv() => match result {
                    Ok(frame) => {
                        let wav = audio_frame_to_wav(frame);
                        Audio {
                            data: rig_core::message::DocumentSourceKind::Raw(wav),
                            media_type: Some(rig_core::message::AudioMediaType::WAV),
                            additional_params: None,
                        }.into()
                    }
                    Err(err) => {
                        eprintln!("Voice subscriber error: {err}");
                        break;
                    }
                },
            };
            self.llm.ask(message).await;
        }
    }
}

fn audio_frame_to_wav(frame: AudioFrame) -> Vec<u8> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    let spec = hound::WavSpec {
        channels: frame.channels,
        sample_rate: frame.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(&mut bytes, spec).expect("Failed to create WAV");
    for sample in frame.samples {
        let sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer
            .write_sample(sample)
            .expect("Failed to write WAV sample");
    }
    writer.finalize().expect("Failed to finalize WAV");
    bytes.into_inner()
}

#[cfg(test)]
mod tests {
    use super::audio_frame_to_wav;
    use comms::AudioFrame;

    #[test]
    fn audio_frame_is_encoded_as_wav() {
        let wav = audio_frame_to_wav(AudioFrame {
            sample_rate: 16_000,
            channels: 1,
            samples: vec![0.0, 1.0, -1.0],
        });

        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }
}
