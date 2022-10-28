pub mod cli;
pub mod path;

use self::path::{install_dir, NoHomeDir};

use std::{
    error::Error as StdError,
    fmt::{self, Debug, Display},
    io::{self, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstalledCommitMsgError {
    #[error(transparent)]
    NoHomeDir(#[from] NoHomeDir),
    #[error("Failed to read version info from {path:?}: {source}")]
    ReadFailed { path: PathBuf, source: io::Error },
}

pub fn installed_commit_msg() -> Result<Option<String>, InstalledCommitMsgError> {
    let path = install_dir()?.join("commit");
    if path.is_file() {
        std::fs::read_to_string(&path)
            .map(Some)
            .map_err(|source| InstalledCommitMsgError::ReadFailed { path, source })
    } else {
        Ok(None)
    }
}

pub fn format_commit_msg(msg: String) -> String {
    format!("Contains commits up to {:?}", msg)
}
