use clap::Parser;

// @see https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html

/// output the last part of files
#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about)]
pub struct Args {
    /// Input files. Omit or use '-' to read from stdin.
    pub files: Vec<String>,

    /// Stdin stream destination filename.
    #[arg(short = 'i', long)]
    pub stdin: Option<String>,

    /// Stdout stream destination filename. Defaults to stdout. Use '-' to set output to process stdout explicitely.
    #[arg(short = 'o', long, default_value = "-")]
    pub stdout: String,

    /// Stderr stream destination filename. Use '-' to output to process stderr
    #[arg(short = 'e', long)]
    pub stderr: Option<String>,

    /// Not try to recover from parsing errors and fail immediately
    #[arg(short = 'f', long, default_value_t = false)]
    pub fatal: bool,

    /// Silent -- do not print error information to stderr
    #[arg(short = 's', long, visible_alias = "silent", default_value_t = false)]
    pub silent: bool,
}

impl Args {
    pub fn parse() -> Args {
        let mut args = <Self as Parser>::parse();
        if args.files.len() == 0 {
            args.files.push("-".into());
        }
        args
    }
}
