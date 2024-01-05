use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DockerDecoderError {
    MalformedHeader,
}
impl fmt::Display for DockerDecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DockerDecoderError::MalformedHeader => {
                write!(f, "Value too large for defined data type")
            }
        }
    }
}

impl Error for DockerDecoderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DockerDecoderError::MalformedHeader => Some(self),
        }
    }
}

// impl From<std::io::Error> for DockerDecoderError {
//     fn from(err: std::io::Error) -> Self {
//         DockerDecoderError::Io(err)
//     }
// }
