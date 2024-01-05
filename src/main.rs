mod args;
mod docker_decoder_chunk_writer;
mod docker_decoder_error;
mod docker_stream_decoder;
mod frame_header;

use std::error::Error;
use std::{
    fs::File,
    io::{BufReader, Read},
};

use args::Args;

const BUFFER_SIZE: usize = 8192;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut buffer = [0u8; BUFFER_SIZE];

    for filename in &args.files {
        let mut decoder = docker_stream_decoder::DockerStreamDecoder::new();
        let file: Box<dyn Read> = match filename.as_str() {
            "-" => Box::new(std::io::stdin()),
            _ => {
                let file = File::open(filename)?;
                Box::new(file)
            }
        };
        let mut reader = BufReader::new(file);

        let mut chunk_writer = docker_decoder_chunk_writer::DockerDecoderChunkWriter::new(&args)?;

        let bytes_read = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(err) => {
                // TODO Should we just throw here, as it's not a decoding error but an actual IO error?
                if !args.silent {
                    eprintln!("Error reading from file {}: {}", filename, err);
                }
                continue;
            }
        };

        for chunk_result in decoder.decode(&buffer[0..bytes_read]) {
            match chunk_result {
                Ok(chunk) => {
                    chunk_writer.write(chunk)?;
                }
                Err(err) => {
                    if !args.silent {
                        eprintln!("Error processing docker stream {}", err);
                    }
                    if args.fatal {
                        return Err(Box::new(err));
                    }
                }
            }
        }
    }
    Ok(())
}
