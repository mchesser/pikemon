use std::fmt;
use std::io;
use std::sync::mpsc::{SendError, RecvError};
use std::error::Error;

pub type NetworkResult<T> = Result<T, NetworkError>;

#[must_use]
pub enum NetworkError {
    Io(io::Error),
    SendError,
    RecvError,
    DecodeError,
    EncodeError,
}

impl Error for NetworkError {
    fn description(&self) -> &str {
        match *self {
            NetworkError::Io(ref e) => e.description(),
            NetworkError::SendError => "sending on a closed channel",
            NetworkError::RecvError => "receiving on a closed channel",
            NetworkError::DecodeError => "received invalid network data",
            NetworkError::EncodeError => "failed to encode network data",
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
impl fmt::Debug for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<io::Error> for NetworkError {
    fn from(err: io::Error) -> NetworkError {
        NetworkError::Io(err)
    }
}

impl<T> From<SendError<T>> for NetworkError {
    fn from(_: SendError<T>) -> NetworkError {
        NetworkError::SendError
    }
}

impl From<RecvError> for NetworkError {
    fn from(_: RecvError) -> NetworkError {
        NetworkError::RecvError
    }
}
