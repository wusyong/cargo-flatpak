use std::{
    fmt::{self, Display},
    io,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to get user's home directory!")]
pub struct NoHomeDir;

pub fn home_dir() -> Result<PathBuf, NoHomeDir> {
    home::home_dir().ok_or(NoHomeDir)
}

pub fn expand_home(path: impl AsRef<Path>) -> Result<PathBuf, NoHomeDir> {
    let home = home_dir()?;
    let path = path.as_ref();
    if let Ok(path) = path.strip_prefix("~") {
        Ok(home.join(path))
    } else {
        Ok(path.to_owned())
    }
}

#[derive(Debug, Error)]
pub enum ContractHomeError {
    #[error(transparent)]
    NoHomeDir(#[from] NoHomeDir),
    #[error("User's home directory path wasn't valid UTF-8.")]
    HomeInvalidUtf8,
    #[error("Supplied path wasn't valid UTF-8.")]
    PathInvalidUtf8,
}

pub fn contract_home(path: impl AsRef<Path>) -> Result<String, ContractHomeError> {
    let path = path
        .as_ref()
        .to_str()
        .ok_or(ContractHomeError::PathInvalidUtf8)?;
    #[cfg(not(windows))]
    {
        let home = home_dir()?;
        let home = home.to_str().ok_or(ContractHomeError::HomeInvalidUtf8)?;
        Ok(path.replace(home, "~").to_owned())
    }
    #[cfg(windows)]
    {
        Ok(path.to_owned())
    }
}

pub fn install_dir() -> Result<PathBuf, NoHomeDir> {
    home_dir().map(|home| home.join(concat!(".", env!("CARGO_PKG_NAME"))))
}

pub fn checkouts_dir() -> Result<PathBuf, NoHomeDir> {
    install_dir().map(|install_dir| install_dir.join("checkouts"))
}

pub fn tools_dir() -> Result<PathBuf, NoHomeDir> {
    install_dir().map(|install_dir| install_dir.join("tools"))
}
