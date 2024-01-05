use crate::docker_decoder_error::DockerDecoderError;
use byteorder::{BigEndian, ByteOrder};

pub const FRAME_HEADER_LENGTH: usize = 8;

pub enum StreamType {
    Stdin = 0,
    Stdout = 1,
    Stderr = 2,
}

impl TryFrom<u8> for StreamType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(StreamType::Stdin),
            1 => Ok(StreamType::Stdout),
            2 => Ok(StreamType::Stderr),
            _ => Err(()),
        }
    }
}

pub struct FrameHeader {
    pub stream_type: u8,
    pub length: u32,
}

impl FrameHeader {
    pub fn parse(buffer: &mut [u8; FRAME_HEADER_LENGTH]) -> Result<Self, DockerDecoderError> {
        if buffer[1] != 0u8 && buffer[2] != 0u8 && buffer[3] != 0u8 {
            return Err(DockerDecoderError::MalformedHeader(buffer.clone()));
        }
        let length = BigEndian::read_u32(&buffer[4..]);
        Ok(FrameHeader {
            stream_type: buffer[0],
            length: length,
        })
    }
}
