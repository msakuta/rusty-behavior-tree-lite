use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Yaml(serde_yaml::Error),
    Missing,
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Error::Yaml(e) => e.fmt(fmt),
            Error::Missing => write!(fmt, "Missing"),
        }
    }
}

impl std::error::Error for Error {}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err)
    }
}
