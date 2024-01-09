use std::error::Error;
use std::fmt;

use crate::frame_header::FRAME_HEADER_LENGTH;

#[derive(Debug)]
pub enum DockerDecoderError {
    IncorrectFrameType(u8),
    MalformedHeader([u8; FRAME_HEADER_LENGTH]),
}
impl fmt::Display for DockerDecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedHeader(header) => {
                write!(
                    f,
                    "Malformed docker frame header, header contents dump: {:x?}",
                    header
                )
            }
            Self::IncorrectFrameType(t) => {
                write!(f, "Incorrect DockerFrame type: {}", t)
            }
        }
    }
}

impl Error for DockerDecoderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
