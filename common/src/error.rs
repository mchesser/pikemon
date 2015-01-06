use std::fmt;
use std::io::IoError;
use std::sync::mpsc::{SendError, RecvError};
use std::error::{Error, FromError};

pub type NetworkResult<T> = Result<T, NetworkError>;

#[must_use]
pub enum NetworkError {
    Io(IoError),
    SendError,
    RecvError,
    DecodeError,
}

impl Error for NetworkError {
    fn description(&self) -> &str {
        match *self {
            NetworkError::Io(ref e) => e.description(),
            NetworkError::SendError => "sending on a closed channel",
            NetworkError::RecvError => "receiving on a closed channel",
            NetworkError::DecodeError => "received invalid network data",
        }
    }
}

impl fmt::Show for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl FromError<IoError> for NetworkError {
    fn from_error(err: IoError) -> NetworkError {
        NetworkError::Io(err)
    }
}

impl<T> FromError<SendError<T>> for NetworkError {
    fn from_error(_: SendError<T>) -> NetworkError {
        NetworkError::SendError
    }
}

impl FromError<RecvError> for NetworkError {
    fn from_error(_: RecvError) -> NetworkError {
        NetworkError::RecvError
    }
}
