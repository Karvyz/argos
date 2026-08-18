use serde::{Serialize, de::DeserializeOwned};

use crate::{AudioFrame, CameraFrame, Error, MotorCommand};

pub trait Topic: Send + Sync + 'static {
    type Message: Serialize + DeserializeOwned + Send + Sync + 'static;

    const KEY: &'static str;
    const BUFFER: usize;

    fn encode(message: &Self::Message) -> Result<Vec<u8>, Error> {
        Ok(postcard::to_allocvec(message)?)
    }

    fn decode(payload: &[u8]) -> Result<Self::Message, Error> {
        Ok(postcard::from_bytes(payload)?)
    }
}

pub struct MotorCommands;
impl Topic for MotorCommands {
    type Message = MotorCommand;

    const KEY: &'static str = "robot/motors";
    const BUFFER: usize = 5;
}

pub struct Voice;
impl Topic for Voice {
    type Message = AudioFrame;

    const KEY: &'static str = "robot/mic";
    const BUFFER: usize = 5;
}

pub struct SpeakerAudio;
impl Topic for SpeakerAudio {
    type Message = AudioFrame;

    const KEY: &'static str = "robot/speaker";
    const BUFFER: usize = 5;
}

pub struct CameraFrames;
impl Topic for CameraFrames {
    type Message = CameraFrame;

    const KEY: &'static str = "robot/camera";
    const BUFFER: usize = 1;
}
