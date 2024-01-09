use rand::prelude::*;
use std::io::Read;

use crate::frame_header::{FrameHeader, FRAME_HEADER_LENGTH};

pub struct StreamSourceInfo<'a> {
    pub stream_type: u8,
    pub source: Box<(dyn Read + 'a)>,
}

#[derive(Clone, Copy)]
struct RemainderInfo {
    bytes_written: usize,
    body_length: usize,
}

#[derive(Clone, Copy)]
enum OperationMode {
    Read,
    CopyHeader(RemainderInfo),
    CopyBody(RemainderInfo),
}

pub struct DockerStreamMultiplexer<'a> {
    operation_mode: OperationMode,
    body_buffer: Vec<u8>,
    header_buffer: [u8; FRAME_HEADER_LENGTH],

    frame_min: u32,
    frame_max: u32,
    sources: Vec<StreamSourceInfo<'a>>,
    rand_rng: ThreadRng,
}

impl<'a> DockerStreamMultiplexer<'a> {
    pub fn new(sources: Vec<StreamSourceInfo<'a>>, frame_max: u32, frame_min: u32) -> Self {
        Self {
            operation_mode: OperationMode::Read,
            body_buffer: Vec::with_capacity(frame_max as usize),
            header_buffer: [0u8; FRAME_HEADER_LENGTH],
            frame_max: frame_max,
            frame_min: frame_min,
            sources: sources,
            rand_rng: rand::thread_rng(),
        }
    }

    fn get_random_chunk_size(&mut self) -> usize {
        if self.frame_min == self.frame_max {
            return self.frame_max as usize;
        }
        self.rand_rng.gen_range(self.frame_min..=self.frame_max) as usize
    }

    fn get_random_source_index(&mut self) -> usize {
        if self.sources.len() == 0 {
            return 0;
        }
        self.rand_rng.gen_range(0..self.sources.len())
    }

    /** Reads a new chunk from a random source and generates its header */
    fn read_chunk(&mut self) -> std::io::Result<Option<FrameHeader>> {
        while self.sources.len() > 0 {
            let bytes_to_read = self.get_random_chunk_size();
            let source_index = self.get_random_source_index();

            let source_info = &mut self.sources[source_index];
            let n_bytes_read = source_info
                .source
                .read(&mut self.body_buffer[0..bytes_to_read])?;

            if n_bytes_read == 0 {
                self.sources.remove(source_index);
            } else {
                return Ok(Some(FrameHeader::new(
                    source_info.stream_type,
                    n_bytes_read as u32,
                )));
            }
        }
        Ok(None)
    }

    fn copy_header(&mut self, buf: &mut [u8], header_info: &RemainderInfo) -> usize {
        let remainder = FRAME_HEADER_LENGTH - header_info.bytes_written;
        let bytes_to_write = std::cmp::min(remainder, buf.len());
        buf.copy_from_slice(&self.header_buffer[remainder..]);
        assert!(
            bytes_to_write <= remainder,
            "shouldn't write more, than we have space"
        );
        if bytes_to_write == remainder {
            self.operation_mode = OperationMode::CopyBody(RemainderInfo {
                bytes_written: 0,
                body_length: header_info.body_length,
            });
        } else {
            self.operation_mode = OperationMode::CopyHeader(RemainderInfo {
                bytes_written: header_info.bytes_written + bytes_to_write,
                body_length: header_info.body_length,
            });
        }
        bytes_to_write
    }

    fn copy_body(&mut self, buf: &mut [u8], header_info: &RemainderInfo) -> usize {
        let remainder = header_info.body_length - header_info.bytes_written;
        let bytes_to_write = std::cmp::min(remainder, buf.len());
        buf[0..bytes_to_write].copy_from_slice(&self.body_buffer[0..bytes_to_write]);

        if bytes_to_write == remainder {
            self.operation_mode = OperationMode::Read;
        } else {
            self.operation_mode = OperationMode::CopyBody(RemainderInfo {
                bytes_written: header_info.bytes_written + bytes_to_write,
                body_length: header_info.body_length,
            });
        }
        bytes_to_write
    }
}

impl<'a> Read for DockerStreamMultiplexer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_written_total: usize = 0;
        while bytes_written_total < buf.len() {
            match self.operation_mode {
                OperationMode::Read => {
                    let frame_header = self.read_chunk()?;
                    if let Some(header) = frame_header {
                        header.serialize(&mut self.header_buffer);
                        self.operation_mode = OperationMode::CopyHeader(RemainderInfo {
                            bytes_written: 0,
                            body_length: header.length as usize,
                        });
                    } else {
                        return Ok(0);
                    }
                }
                OperationMode::CopyHeader(header_info) => {
                    bytes_written_total += self.copy_header(buf, &header_info);
                }
                OperationMode::CopyBody(remainder_info) => {
                    bytes_written_total += self.copy_body(buf, &remainder_info);
                }
            }
        }
        return Ok(bytes_written_total);
    }
}
