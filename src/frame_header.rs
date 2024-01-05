use crate::docker_decoder_error::DockerDecoderError;
use byteorder::{BigEndian, ByteOrder};

pub const FRAME_HEADER_LENGTH: usize = 8;

pub struct FrameHeader {
    pub stream_type: u8,
    pub length: u32,
}

impl FrameHeader {
    pub fn parse(buffer: &mut [u8; FRAME_HEADER_LENGTH]) -> Result<Self, DockerDecoderError> {
        if buffer[1] != 0u8 && buffer[2] != 0u8 && buffer[3] != 0u8 {
            return Err(DockerDecoderError::MalformedHeader);
        }
        let length = BigEndian::read_u32(&buffer[4..]);
        Ok(FrameHeader {
            stream_type: buffer[0],
            length: length,
        })
    }
}
