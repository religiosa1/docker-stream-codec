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
