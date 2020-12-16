use std::io;

#[derive(Debug)]
pub enum CyKvError {
	Io(io::Error),
	Serialize(bson::ser::Error),
	Deserialize(bson::de::Error),
	Internal
}

impl From<io::Error> for CyKvError {
	fn from(err: io::Error) -> CyKvError {
		CyKvError::Io(err)
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
