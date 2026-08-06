use serde::{Deserialize, Serialize};

pub const RG24_FOURCC: u32 = u32::from_le_bytes([b'R', b'G', b'2', b'4']);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MotorCommand {
    pub angles: [f32; 15],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub fourcc: u32,
    pub sequence: u32,
    pub data: Vec<u8>,
}
