use byteorder::{BigEndian, ByteOrder};
pub const FRAME_HEADER_LENGTH: usize = 8;

pub struct FrameHeader {
    pub stream_type: u8,
    pub length: u32,
}

impl FrameHeader {
    pub fn new(stream_type: u8, length: u32) -> Self {
        Self {
            stream_type,
            length,
        }
    }
    pub fn serialize(&self, buffer: &mut [u8]) {
        assert!(
            buffer.len() >= 8,
            "Buffer has enough space to write frame header in it"
        );
        buffer[0] = self.stream_type;
        BigEndian::write_u32(&mut buffer[4..], self.length);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header_parse() {
        let header = FrameHeader::new(2, 0x25);
        let mut buffer = [0u8; FRAME_HEADER_LENGTH];
        header.serialize(&mut buffer);
        assert_eq!(buffer, [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x25])
    }
}
