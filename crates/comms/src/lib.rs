use zenoh::{Config, Session, bytes::ZBytes};
use zenoh_ext::{z_deserialize, z_serialize};

pub mod keys {
    pub const CAMERA: &str = "robot/camera";
    pub const MIC: &str = "robot/mic";
    pub const SPEAKER: &str = "robot/speaker";
    pub const MOTORS: &str = "robot/motors";
}

/// Agreed audio wire format for `mic` and `speaker`: mono `f32` PCM at 24 kHz.
pub const AUDIO_SAMPLE_RATE: u32 = 24_000;
pub const AUDIO_CHANNELS: u16 = 1;

pub async fn open_session() -> Session {
    zenoh::init_log_from_env_or("error");
    zenoh::open(Config::default()).await.unwrap()
}

/// Motor commands: 15 joint angles in `xgo::Motor::ALL` order.
pub mod motors {
    use super::*;

    pub fn encode(angles: &[f32; 15]) -> ZBytes {
        z_serialize(angles)
    }

    pub fn decode(bytes: &ZBytes) -> Option<[f32; 15]> {
        z_deserialize::<[f32; 15]>(bytes).ok()
    }
}

/// Audio payload: raw interleaved little-endian `f32` samples.
pub mod audio {
    use super::*;

    pub fn encode(samples: &[f32]) -> ZBytes {
        let mut buf = Vec::with_capacity(samples.len() * 4);
        for s in samples {
            buf.extend_from_slice(&s.to_le_bytes());
        }
        ZBytes::from(buf)
    }

    pub fn decode(bytes: &ZBytes) -> Vec<f32> {
        bytes
            .to_bytes()
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }
}

/// Camera payload: a fixed 16-byte header followed by the raw frame bytes.
pub mod camera {
    use super::*;

    pub const RG24_FOURCC: u32 = u32::from_le_bytes([b'R', b'G', b'2', b'4']);
    pub const HEADER_LEN: usize = 16;

    #[derive(Debug, Clone, Copy)]
    pub struct Header {
        pub width: u32,
        pub height: u32,
        pub fourcc: u32,
        pub seq: u32,
    }

    impl Header {
        pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
            let mut b = [0u8; HEADER_LEN];
            b[0..4].copy_from_slice(&self.width.to_le_bytes());
            b[4..8].copy_from_slice(&self.height.to_le_bytes());
            b[8..12].copy_from_slice(&self.fourcc.to_le_bytes());
            b[12..16].copy_from_slice(&self.seq.to_le_bytes());
            b
        }

        pub fn from_bytes(b: &[u8]) -> Option<Header> {
            if b.len() < HEADER_LEN {
                return None;
            }
            Some(Header {
                width: u32::from_le_bytes(b[0..4].try_into().unwrap()),
                height: u32::from_le_bytes(b[4..8].try_into().unwrap()),
                fourcc: u32::from_le_bytes(b[8..12].try_into().unwrap()),
                seq: u32::from_le_bytes(b[12..16].try_into().unwrap()),
            })
        }
    }

    pub fn encode(header: &Header, frame: &[u8]) -> ZBytes {
        let mut buf = Vec::with_capacity(HEADER_LEN + frame.len());
        buf.extend_from_slice(&header.to_bytes());
        buf.extend_from_slice(frame);
        ZBytes::from(buf)
    }

    pub fn decode(bytes: &ZBytes) -> Option<(Header, Vec<u8>)> {
        let raw = bytes.to_bytes();
        let header = Header::from_bytes(&raw)?;
        Some((header, raw[HEADER_LEN..].to_vec()))
    }
}
