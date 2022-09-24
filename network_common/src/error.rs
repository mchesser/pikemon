use std::{error::Error, fmt, io};

pub type NetworkResult<T> = Result<T, NetworkError>;

#[derive(Debug)]
#[must_use]
pub enum NetworkError {
    Io(io::Error),
    SendError,
    RecvError,
    DecodeError,
    EncodeError,
}

impl Error for NetworkError {}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetworkError::Io(e) => e.fmt(f),
            NetworkError::SendError => f.write_str("sending on a closed channel"),
            NetworkError::RecvError => f.write_str("receiving on a closed channel"),
            NetworkError::DecodeError => f.write_str("received invalid network data"),
            NetworkError::EncodeError => f.write_str("failed to encode network data"),
        }
    }
}

impl From<io::Error> for NetworkError {
    fn from(err: io::Error) -> NetworkError {
        NetworkError::Io(err)
    }
}
