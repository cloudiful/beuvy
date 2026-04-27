use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum DeclarativeUiAssetLoadError {
    Io(std::io::Error),
    InvalidUtf8(String),
    Xml(roxmltree::Error),
    InvalidDsl(String),
}

impl Display for DeclarativeUiAssetLoadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to read declarative ui asset: {error}"),
            Self::InvalidUtf8(error) => {
                write!(
                    formatter,
                    "failed to decode declarative ui asset as utf-8: {error}"
                )
            }
            Self::Xml(error) => write!(formatter, "failed to parse declarative ui XML: {error}"),
            Self::InvalidDsl(error) => write!(formatter, "invalid declarative ui asset: {error}"),
        }
    }
}

impl Error for DeclarativeUiAssetLoadError {}

impl From<std::io::Error> for DeclarativeUiAssetLoadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<roxmltree::Error> for DeclarativeUiAssetLoadError {
    fn from(error: roxmltree::Error) -> Self {
        Self::Xml(error)
    }
}
