use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum NNTPError {
    IO(std::io::Error),
    UnexpectedCode(String),
    FromUTF8Error,
    ReadLineFailed,
    TLSFailed,
    NoneError,
    Other,
}

impl From<std::io::Error> for NNTPError {
    fn from(err: std::io::Error) -> Self {
        NNTPError::IO(err)
    }
}

impl From<std::option::NoneError> for NNTPError {
    fn from(_: std::option::NoneError) -> Self {
        NNTPError::NoneError
    }
}

impl From<std::string::FromUtf8Error> for NNTPError {
    fn from(_: FromUtf8Error) -> Self {
        NNTPError::FromUTF8Error
    }
}
