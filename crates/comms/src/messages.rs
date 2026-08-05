pub const AUDIO_SAMPLE_RATE: u32 = 24_000;
pub const AUDIO_CHANNELS: u16 = 1;
pub const RG24_FOURCC: u32 = u32::from_le_bytes([b'R', b'G', b'2', b'4']);

#[derive(Debug, Clone, Copy)]
pub struct MotorCommand {
    pub angles: [f32; 15],
}

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub fourcc: u32,
    pub sequence: u32,
    pub data: Vec<u8>,
}
