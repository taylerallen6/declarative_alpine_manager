use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeclarativeAlpineError {
    #[error("APK world not found")]
    ApkWorldFileError,
    #[error("Failed to run 'apk upgrade': {0}")]
    ApkUpgradeError(String),

    #[error("io Error")]
    IoError(#[from] io::Error),
    #[error("toml deserialize Error")]
    TomlDeserializeError(#[from] toml::de::Error),
    #[error("toml serialize Error")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("database: Empty_slots vector is empty")]
    EmptySlotsVectorIsEmpty,
    #[error("database: Connection {0} is not set. No data here.")]
    ConnectionNotSet(usize),

    #[error("unknown database error")]
    Unknown,
}
