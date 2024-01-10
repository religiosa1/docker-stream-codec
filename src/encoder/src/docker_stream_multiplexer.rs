use rand::prelude::*;
use std::io::Read;

use crate::frame_header::{FrameHeader, FRAME_HEADER_LENGTH};

pub struct StreamSourceInfo {
    pub stream_type: u8,
    pub source: Box<(dyn Read)>,
}

#[derive(Clone, Copy)]
enum OperationMode {
    Read,
    CopyHeader(usize),
    CopyBody(usize),
}

pub struct DockerStreamMultiplexer {
    operation_mode: OperationMode,
    body_buffer: Vec<u8>,
    header_buffer: [u8; FRAME_HEADER_LENGTH],

    frame_min: u32,
    frame_max: u32,
    sources: Vec<StreamSourceInfo>,
    rand_rng: ThreadRng,

    bytes_written: usize,
    body_length: usize,
}

impl DockerStreamMultiplexer {
    pub fn new(sources: Vec<StreamSourceInfo>, frame_max: u32, frame_min: u32) -> Self {
        Self {
            operation_mode: OperationMode::Read,
            body_buffer: vec![0; frame_max as usize],
            header_buffer: [0u8; FRAME_HEADER_LENGTH],
            frame_max: frame_max,
            frame_min: frame_min,
            sources: sources,
            rand_rng: rand::thread_rng(),
            bytes_written: 0,
            body_length: 0,
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

    fn copy_header(&mut self, buf: &mut [u8], header_bytes_written: usize) {
        let remainder = FRAME_HEADER_LENGTH - header_bytes_written;
        let bytes_to_write = std::cmp::min(remainder, buf.len() - self.bytes_written);

        let dest_buf_rng = self.bytes_written..self.bytes_written + bytes_to_write;
        let body_buf_rng = header_bytes_written..header_bytes_written + bytes_to_write;

        buf[dest_buf_rng].copy_from_slice(&self.header_buffer[body_buf_rng]);
        self.bytes_written += bytes_to_write;

        if bytes_to_write < remainder {
            self.operation_mode = OperationMode::CopyHeader(header_bytes_written + bytes_to_write);
        } else {
            self.operation_mode = OperationMode::CopyBody(0);
        }
    }

    fn copy_body(&mut self, buf: &mut [u8], body_bytes_written: usize) {
        let remainder = self.body_length - body_bytes_written;
        let bytes_to_write = std::cmp::min(remainder, buf.len() - self.bytes_written);

        let dest_buf_rng = self.bytes_written..self.bytes_written + bytes_to_write;
        let body_buf_rng = body_bytes_written..body_bytes_written + bytes_to_write;

        buf[dest_buf_rng].copy_from_slice(&self.body_buffer[body_buf_rng]);
        self.bytes_written += bytes_to_write;

        if bytes_to_write < remainder {
            self.operation_mode = OperationMode::CopyBody(body_bytes_written + bytes_to_write);
        } else {
            self.operation_mode = OperationMode::Read;
        }
    }
}

impl Read for DockerStreamMultiplexer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        self.bytes_written = 0;

        while self.bytes_written < buf.len() {
            match self.operation_mode {
                OperationMode::Read => {
                    let frame_header = self.read_chunk()?;
                    if let Some(header) = frame_header {
                        header.serialize(&mut self.header_buffer);
                        self.body_length = header.length as usize;
                        self.operation_mode = OperationMode::CopyHeader(0);
                    } else {
                        return Ok(self.bytes_written);
                    }
                }
                OperationMode::CopyHeader(header_bytes_written) => {
                    self.copy_header(buf, header_bytes_written)
                }
                OperationMode::CopyBody(body_bytes_written) => {
                    self.copy_body(buf, body_bytes_written)
                }
            }
        }
        return Ok(self.bytes_written);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    fn make_simple_input_output() -> (Vec<StreamSourceInfo>, Vec<u8>) {
        let test_input: Cursor<[u8; 9]> =
            Cursor::new([0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]);
        let sources_list = vec![{
            StreamSourceInfo {
                stream_type: 2u8,
                source: Box::new(test_input),
            }
        }];

        let header: [u8; FRAME_HEADER_LENGTH] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03];
        let mut expected_output = Vec::<u8>::new();
        expected_output.extend_from_slice(&header);
        expected_output.extend_from_slice(&[0x01, 0x02, 0x03]);
        expected_output.extend_from_slice(&header);
        expected_output.extend_from_slice(&[0x04, 0x05, 0x06]);
        expected_output.extend_from_slice(&header);
        expected_output.extend_from_slice(&[0x07, 0x08, 0x09]);

        return (sources_list, expected_output);
    }

    #[test]
    fn breaks_single_stream_into_chunks() {
        let (test_source, expected_output) = make_simple_input_output();
        let mut mp = DockerStreamMultiplexer::new(test_source, 3, 3);

        let mut output = vec![0; expected_output.len()];
        mp.read(&mut output).unwrap();

        assert_eq!(output, expected_output);
    }

    #[test]
    fn read_to_end() {
        let (test_source, expected_output) = make_simple_input_output();
        let mut mp = DockerStreamMultiplexer::new(test_source, 3, 3);

        let mut output = Vec::<u8>::new();
        mp.read_to_end(&mut output).unwrap();

        assert_eq!(output, expected_output);
    }
}
