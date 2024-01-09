use std::{error::Error, fmt};

use clap::Parser;

const FRAME_SIZE_ABS_MAX: u32 = 4092;

// @see https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html

/// output the last part of files
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// output file name, use '-' for stdout
    #[arg(short = 'O', long, default_value = "-")]
    pub output: String,

    /// Stdin stream source filename.
    #[arg(short = 'i', long)]
    pub stdin: Option<String>,

    /// Stdout stream source filename.
    #[arg(short = 'o', long)]
    pub stdout: Option<String>,

    /// Stderr stream source filename.
    #[arg(short = 'e', long)]
    pub stderr: Option<String>,

    /// Frame size max
    #[arg(short = 'm', long, default_value_t = 200)]
    pub frame_max: u32,

    /// Frame size min. Can be specified as negative value (offset from frame_max) or 0 -- equals to frame_size_max
    #[arg(short = 'm', long, default_value_t = 0)]
    pub frame_min: i32,
}

impl Args {
    pub fn parse() -> Result<Args, ArgsError> {
        let mut args = <Self as Parser>::parse();
        if args.stdout.is_none() && args.stdout.is_none() && args.stderr.is_none() {
            return Err(ArgsError::NoInputSpecified);
        }
        if args.frame_max > FRAME_SIZE_ABS_MAX {
            return Err(ArgsError::FrameSizeExceeded(args.frame_max));
        }
        if args.frame_min <= 0 {
            args.frame_min = (args.frame_max as i32 + args.frame_min) as i32
        } else if args.frame_min > args.frame_max as i32 {
            args.frame_min = args.frame_max as i32;
        }
        Ok(args)
    }

    pub fn get_sources(&self) -> SourcesIterator {
        SourcesIterator {
            args: self,
            last_checked: 0,
        }
    }
}

pub struct StreamFilename<'a> {
    pub stream_type: u8,
    pub filename: &'a String,
}

pub struct SourcesIterator<'a> {
    args: &'a Args,
    last_checked: u8,
}
impl<'a> Iterator for SourcesIterator<'a> {
    type Item = StreamFilename<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.last_checked <= 2 {
            let stream_type = self.last_checked;
            self.last_checked += 1;
            let maybe_stream = match stream_type {
                0 if self.args.stdin.is_some() => &self.args.stdin,
                1 if self.args.stdout.is_some() => &self.args.stdout,
                2 if self.args.stderr.is_some() => &self.args.stderr,
                _ => &None,
            };

            if let Some(filename) = maybe_stream {
                return Some(StreamFilename {
                    filename,
                    stream_type,
                });
            }
        }
        return None;
    }
}

#[derive(Debug)]
pub enum ArgsError {
    NoInputSpecified,
    FrameSizeExceeded(u32),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoInputSpecified => {
                write!(f, "No input files were specified, you must specify any of --stdin, --stdout or --stderr files")
            }
            Self::FrameSizeExceeded(header) => {
                write!(
                    f,
                    "Specified frame size exceeds maximum possible value: provided {}, max possible is {}",
                    header,
                    FRAME_SIZE_ABS_MAX
                )
            }
        }
    }
}

impl Error for ArgsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
