use crate::{AudioFrame, CameraFrame, Error, MotorCommand};

pub trait Topic: Send + Sync + 'static {
    type Message: Send + Sync + 'static;

    const KEY: &'static str;

    fn encode(message: &Self::Message) -> Vec<u8>;
    fn decode(payload: &[u8]) -> Result<Self::Message, Error>;
}

pub struct MotorCommands;

impl Topic for MotorCommands {
    type Message = MotorCommand;

    const KEY: &'static str = "robot/motors";

    fn encode(message: &Self::Message) -> Vec<u8> {
        message
            .angles
            .iter()
            .flat_map(|angle| angle.to_le_bytes())
            .collect()
    }

    fn decode(payload: &[u8]) -> Result<Self::Message, Error> {
        if payload.len() != 15 * std::mem::size_of::<f32>() {
            return Err(Error::InvalidPayload {
                topic: Self::KEY,
                reason: "expected 15 f32 joint angles",
            });
        }
        let angles = std::array::from_fn(|index| {
            let offset = index * std::mem::size_of::<f32>();
            f32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap())
        });
        Ok(MotorCommand { angles })
    }
}

pub struct SpeakerAudio;

impl Topic for SpeakerAudio {
    type Message = AudioFrame;

    const KEY: &'static str = "robot/speaker";

    fn encode(message: &Self::Message) -> Vec<u8> {
        message
            .samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect()
    }

    fn decode(payload: &[u8]) -> Result<Self::Message, Error> {
        if !payload.len().is_multiple_of(std::mem::size_of::<f32>()) {
            return Err(Error::InvalidPayload {
                topic: Self::KEY,
                reason: "sample data is not aligned to f32 values",
            });
        }
        let samples = payload
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        Ok(AudioFrame { samples })
    }
}

pub struct CameraFrames;

impl Topic for CameraFrames {
    type Message = CameraFrame;

    const KEY: &'static str = "robot/camera";

    fn encode(message: &Self::Message) -> Vec<u8> {
        let mut payload = Vec::with_capacity(16 + message.data.len());
        payload.extend_from_slice(&message.width.to_le_bytes());
        payload.extend_from_slice(&message.height.to_le_bytes());
        payload.extend_from_slice(&message.fourcc.to_le_bytes());
        payload.extend_from_slice(&message.sequence.to_le_bytes());
        payload.extend_from_slice(&message.data);
        payload
    }

    fn decode(payload: &[u8]) -> Result<Self::Message, Error> {
        if payload.len() < 16 {
            return Err(Error::InvalidPayload {
                topic: Self::KEY,
                reason: "missing 16-byte frame header",
            });
        }
        Ok(CameraFrame {
            width: u32::from_le_bytes(payload[0..4].try_into().unwrap()),
            height: u32::from_le_bytes(payload[4..8].try_into().unwrap()),
            fourcc: u32::from_le_bytes(payload[8..12].try_into().unwrap()),
            sequence: u32::from_le_bytes(payload[12..16].try_into().unwrap()),
            data: payload[16..].to_vec(),
        })
    }
}
