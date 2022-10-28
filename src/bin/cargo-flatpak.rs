use std::path::PathBuf;

use cargo_flatpak::{
    util::cli::{self, Exec, GlobalFlags, Report, Reportable, VERSION_LONG, VERSION_SHORT},
    NAME,
};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    bin_name = cli::bin_name(NAME),
    version = VERSION_SHORT,
    long_version = VERSION_LONG.as_str(),
)]
pub struct Input {
    #[command(flatten)]
    flags: GlobalFlags,
    // #[structopt(subcommand)]
    // command: Command,
}

#[derive(Debug)]
pub enum Error {
    // InitFailed(init::Error),
    DirCreationFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    DirChangeFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    // OpenFailed(util::OpenInEditorError),
    // UpdateFailed(update::Error),
    // #[cfg(target_os = "macos")]
    // AppleFailed(cargo_mobile::apple::cli::Error),
    // AndroidFailed(cargo_mobile::android::cli::Error),
    // DoctorFailed(doctor::Unrecoverable),
}

impl Reportable for Error {
    fn report(&self) -> Report {
        match self {
            // Self::InitFailed(err) => err.report(),
            Self::DirCreationFailed { path, source } => {
                Report::error(format!("Failed to create directory {:?}", path), source)
            }
            Self::DirChangeFailed { path, source } => Report::error(
                format!("Failed to change current directory {:?}", path),
                source,
            ),
            // Self::OpenFailed(err) => {
            //     Report::error("Failed to open project in default code editor", err)
            // }
            // Self::UpdateFailed(err) => Report::error("Failed to update `cargo-mobile`", err),
            // #[cfg(target_os = "macos")]
            // Self::AppleFailed(err) => err.report(),
            // Self::AndroidFailed(err) => err.report(),
            // Self::DoctorFailed(err) => Report::error("Failed to run doctor", err),
        }
    }
}

impl Exec for Input {
    type Report = Error;

    fn global_flags(&self) -> GlobalFlags {
        self.flags
    }

    fn exec(self, option: &textwrap::Options) -> Result<(), Self::Report> {
        Ok(())
    }
}

fn main() {
    cli::exec::<Input>()
}
