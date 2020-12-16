use failure::Fail;
use serde::{Deserialize, Serialize};
use std::io;
use serde_json::Error;

#[derive(Debug, Fail)]
pub enum CyKvError {
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "serde json: {}", _0)]
    SerdeJson(#[cause] serde_json::Error),

    #[fail(display = "serde bson ser error: {}", _0)]
    Serialize(#[cause] bson::ser::Error),

    #[fail(display = "serde bson de error: {}", _0)]
    Deserialize(#[cause] bson::de::Error),

    #[fail(display = "internal error")]
    Internal,

    #[fail(display = "key not found:{}", _0)]
    KeyNotFound(String),
}

impl From<io::Error> for CyKvError {
    fn from(err: io::Error) -> CyKvError {
        CyKvError::Io(err)
    }
}

impl From<serde_json::Error> for CyKvError {
    fn from(err: serde_json::Error) -> Self {
        CyKvError::SerdeJson(err)
    }
}

impl From<bson::ser::Error> for CyKvError {
    fn from(err: bson::ser::Error) -> CyKvError {
        CyKvError::Serialize(err)
    }
}

impl From<bson::de::Error> for CyKvError {
    fn from(err: bson::de::Error) -> CyKvError {
        CyKvError::Deserialize(err)
    }
}

pub type Result<T> = std::result::Result<T, CyKvError>;
