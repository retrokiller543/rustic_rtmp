use log::error;
use {
    bytesio::bytes_errors::{BytesReadError, BytesWriteError},
    failure::{Fail},
    std::{
        fmt, {io, string},
    },
};

#[derive(Debug, Fail)]
pub enum Amf0ReadErrorValue {
    #[fail(display = "Encountered unknown marker: {}\n", marker)]
    UnknownMarker { marker: u8 },
    #[fail(display = "parser string error: {}\n", _0)]
    StringParseError(#[cause] string::FromUtf8Error),
    #[fail(display = "bytes read error :{}\n", _0)]
    BytesReadError(BytesReadError),
    #[fail(display = "wrong type")]
    WrongType,
}

#[derive(Debug)]
pub struct Amf0ReadError {
    pub value: Amf0ReadErrorValue,
}

impl From<string::FromUtf8Error> for Amf0ReadError {
    fn from(error: string::FromUtf8Error) -> Self {
        error!("string parse error: {}", error);
        Amf0ReadError {
            value: Amf0ReadErrorValue::StringParseError(error),
        }
    }
}

impl From<BytesReadError> for Amf0ReadError {
    fn from(error: BytesReadError) -> Self {
        error!("bytes read error: {}", error);
        Amf0ReadError {
            value: Amf0ReadErrorValue::BytesReadError(error),
        }
    }
}

impl std::error::Error for Amf0ReadError {}

#[derive(Debug, Fail)]
pub enum Amf0WriteErrorValue {
    #[fail(display = "normal string too long")]
    NormalStringTooLong,
    #[fail(display = "io error\n")]
    BufferWriteError(io::Error),
    #[fail(display = "bytes write error\n")]
    BytesWriteError(BytesWriteError),
}

impl std::error::Error for Amf0WriteError {}

#[derive(Debug)]
pub struct Amf0WriteError {
    pub value: Amf0WriteErrorValue,
}

impl From<io::Error> for Amf0WriteError {
    fn from(error: io::Error) -> Self {
        error!("io error: {}", error);
        Amf0WriteError {
            value: Amf0WriteErrorValue::BufferWriteError(error),
        }
    }
}

impl From<BytesWriteError> for Amf0WriteError {
    fn from(error: BytesWriteError) -> Self {
        error!("bytes write error: {}", error);
        Amf0WriteError {
            value: Amf0WriteErrorValue::BytesWriteError(error),
        }
    }
}

impl fmt::Display for Amf0ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.value, f)
    }
}

/*
impl Fail for Amf0ReadError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.value.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.value.backtrace()
    }
}
*/

impl fmt::Display for Amf0WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.value, f)
    }
}
/*
impl Fail for Amf0WriteError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.value.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.value.backtrace()
    }
}
 */
