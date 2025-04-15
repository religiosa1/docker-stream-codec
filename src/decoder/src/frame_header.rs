use crate::errors::DockerDecoderError;
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
    pub fn parse(buffer: &[u8; FRAME_HEADER_LENGTH]) -> Result<Self, DockerDecoderError> {
        if buffer[1] != 0u8 || buffer[2] != 0u8 || buffer[3] != 0u8 {
            return Err(DockerDecoderError::MalformedHeader(buffer.clone()));
        }
        let length = BigEndian::read_u32(&buffer[4..]);
        Ok(Self {
            stream_type: buffer[0],
            length: length,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header_parse() {
        let buffer: [u8; FRAME_HEADER_LENGTH] = [0x01, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78];
        let header = FrameHeader::parse(&buffer).unwrap();
        assert_eq!(header.length, 0x12345678);
        assert_eq!(header.stream_type, 1);
    }

    #[test]
    fn mallformed_header_error() {
        let buffer: [u8; FRAME_HEADER_LENGTH] = [0x00, 0x22, 0x00, 0x00, 0x11, 00, 0x00, 0x22];
        let header = FrameHeader::parse(&buffer);
        assert!(header.is_err());
    }
}
