use crate::frame_header::{FrameHeader, FRAME_HEADER_LENGTH};

use super::errors::DockerDecoderError;

enum ParsingMode {
    Header,
    Body(FrameHeader),
}

pub struct DockerStreamDecoder {
    n_bytes_read: u32,
    header_buffer: [u8; FRAME_HEADER_LENGTH],
    mode: ParsingMode,
}

impl<'a> DockerStreamDecoder {
    pub fn new() -> Self {
        Self {
            n_bytes_read: 0,
            header_buffer: [0u8; FRAME_HEADER_LENGTH],
            mode: ParsingMode::Header,
        }
    }

    pub fn decode(&'a mut self, buffer: &'a [u8]) -> DockerStreamDecoderChunks<'a> {
        DockerStreamDecoderChunks {
            decoder: self,
            chunk: buffer,
        }
    }
}

pub struct DockerStreamDecoderChunks<'a> {
    decoder: &'a mut DockerStreamDecoder,
    chunk: &'a [u8],
}

pub struct DockerDecoderChunk<'a> {
    pub stream_type: u8,
    pub body: &'a [u8],
}

impl<'a> Iterator for DockerStreamDecoderChunks<'a> {
    type Item = Result<DockerDecoderChunk<'a>, DockerDecoderError>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.chunk.len() > 0 {
            if let ParsingMode::Header = self.decoder.mode {
                let n_bytes_read = self.decoder.n_bytes_read as usize;
                let bytes_to_copy = std::cmp::min(
                    FRAME_HEADER_LENGTH - n_bytes_read,
                    self.chunk.len() as usize,
                );
                assert!(
                    bytes_to_copy > 0,
                    "Remaining sizes of buffers must allow header parsing"
                );
                self.decoder.header_buffer[n_bytes_read..]
                    .copy_from_slice(&self.chunk[0..bytes_to_copy]);

                self.chunk = &self.chunk[bytes_to_copy..];
                self.decoder.n_bytes_read += bytes_to_copy as u32;
                if self.decoder.n_bytes_read >= FRAME_HEADER_LENGTH as u32 {
                    self.decoder.n_bytes_read = 0;
                    let header = FrameHeader::parse(&self.decoder.header_buffer);
                    match header {
                        Ok(h) => {
                            if h.length > 0 {
                                self.decoder.mode = ParsingMode::Body(h);
                            }
                        }
                        Err(err) => return Some(Err(err)),
                    }
                }
            }
            if let ParsingMode::Body(header) = &self.decoder.mode {
                if self.chunk.len() == 0 {
                    continue;
                }
                let bytes_to_read = std::cmp::min(
                    (header.length - self.decoder.n_bytes_read) as usize,
                    self.chunk.len(),
                );
                assert!(
                    bytes_to_read > 0,
                    "DockerStreamDecoder should have some data to read"
                );
                self.decoder.n_bytes_read += bytes_to_read as u32;
                let stream_type = header.stream_type;
                let body = &self.chunk[0..bytes_to_read];
                self.chunk = &self.chunk[bytes_to_read..];
                if self.decoder.n_bytes_read >= header.length {
                    self.decoder.n_bytes_read = 0;
                    self.decoder.mode = ParsingMode::Header;
                }
                return Some(Ok(DockerDecoderChunk {
                    stream_type: stream_type,
                    body: body,
                }));
            }
        }
        None
    }
}
