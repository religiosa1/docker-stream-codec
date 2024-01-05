use crate::{args::Args, docker_stream_decoder::DockerDecoderChunk};

use std::{fs::File, io::BufWriter, io::Result, io::Write};

pub struct DockerDecoderChunkWriter {
    stdin: Option<BufWriter<File>>,
    stdout: BufWriter<Box<dyn Write>>,
    stderr: Option<BufWriter<Box<dyn Write>>>,
    silent: bool,
}

impl DockerDecoderChunkWriter {
    pub fn new(args: &Args) -> Result<Self> {
        let stdout_file: Box<dyn Write> = match args.stdout.as_str() {
            "-" => Box::new(std::io::stdout()),
            _ => Box::new(File::open(&args.stdout)?),
        };
        let stdout_writer = BufWriter::new(stdout_file);

        let stdin_writer = match &args.stdin {
            None => None,
            Some(filename) => Some(BufWriter::new(File::open(filename)?)),
        };
        let stderr_file: Option<Box<dyn Write>> = match args.stderr.as_deref() {
            None => None,
            Some("-") => Some(Box::new(std::io::stderr())),
            Some(filename) => Some(Box::new(File::open(filename)?)),
        };
        let stderr_writer = stderr_file.and_then(|file| Some(BufWriter::new(file)));

        Ok(DockerDecoderChunkWriter {
            stdin: stdin_writer,
            stdout: stdout_writer,
            stderr: stderr_writer,
            silent: args.silent,
        })
    }

    pub fn write(&mut self, chunk: DockerDecoderChunk) -> Result<()> {
        match chunk.stream_type {
            // FIXME use StreamType instead
            0u8 => {
                if let Some(stdin) = &mut self.stdin {
                    stdin.write(chunk.body)?;
                }
            }
            1u8 => {
                self.stdout.write(chunk.body)?;
            }
            2u8 => {
                if let Some(stderr) = &mut self.stderr {
                    stderr.write(chunk.body)?;
                }
            }
            _ => {
                // TODO check args.fatal and maybe throw here too
                if !self.silent {
                    eprintln!("Unexpected stream type {}", chunk.stream_type);
                }
            }
        };
        Ok(())
    }
}
