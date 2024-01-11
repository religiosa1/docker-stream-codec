use crate::{args::Args, docker_stream_decoder::DockerDecoderChunk, frame_header::StreamType};

use std::{fs::File, io::BufWriter, io::Result, io::Write};

pub struct DockerDecoderChunkWriter {
    stdin: Option<BufWriter<File>>,
    stdout: BufWriter<Box<dyn Write>>,
    stderr: Option<BufWriter<Box<dyn Write>>>,
}

impl DockerDecoderChunkWriter {
    pub fn new(args: &Args) -> Result<Self> {
        let stdout_file: Box<dyn Write> = match args.stdout.as_str() {
            "-" => Box::new(std::io::stdout()),
            _ => Box::new(File::create(&args.stdout)?),
        };
        let stdout_writer = BufWriter::new(stdout_file);

        let stdin_writer = match &args.stdin {
            None => None,
            Some(filename) => Some(BufWriter::new(File::create(filename)?)),
        };
        let stderr_file: Option<Box<dyn Write>> = match args.stderr.as_deref() {
            None => None,
            Some("-") => Some(Box::new(std::io::stdout())),
            Some(filename) => Some(Box::new(File::create(filename)?)),
        };
        let stderr_writer = stderr_file.and_then(|file| Some(BufWriter::new(file)));

        Ok(Self {
            stdin: stdin_writer,
            stdout: stdout_writer,
            stderr: stderr_writer,
        })
    }

    pub fn write(&mut self, chunk: DockerDecoderChunk) -> Result<()> {
        if let Ok(stream_type) = StreamType::try_from(chunk.stream_type) {
            match stream_type {
                StreamType::Stdin => {
                    if let Some(stdin) = &mut self.stdin {
                        stdin.write(chunk.body)?;
                    }
                }
                StreamType::Stdout => {
                    self.stdout.write(chunk.body)?;
                }
                StreamType::Stderr => {
                    if let Some(stderr) = &mut self.stderr {
                        stderr.write(chunk.body)?;
                    }
                }
            }
        }
        Ok(())
    }
}
