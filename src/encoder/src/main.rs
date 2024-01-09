use std::{
    error::Error,
    fs::File,
    io::{self, BufReader, BufWriter},
    io::{Error as IoError, Write},
};

use crate::{
    args::Args,
    docker_stream_multiplexer::{DockerStreamMultiplexer, StreamSourceInfo},
};

mod args;
mod docker_stream_multiplexer;
mod frame_header;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse()?;

    let sources: Result<Vec<StreamSourceInfo>, IoError> = args
        .get_sources()
        .map(|source_file| {
            let file = File::open(source_file.filename)?;
            Ok(StreamSourceInfo {
                stream_type: source_file.stream_type,
                source: Box::new(BufReader::new(file)),
            })
        })
        .collect();
    let sources = sources?;

    let mut output: BufWriter<Box<dyn Write>> = match args.output.as_str() {
        "-" => BufWriter::new(Box::new(std::io::stdout())),
        _ => BufWriter::new(Box::new(File::create(args.output)?)),
    };

    let mut multiplexer =
        DockerStreamMultiplexer::new(sources, args.frame_max, args.frame_min as u32);
    io::copy(&mut multiplexer, &mut output)?;
    Ok(())
}
