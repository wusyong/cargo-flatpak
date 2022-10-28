use crate::util;
use clap::Parser;
use colored::Colorize;
use once_cell::sync::Lazy;
use std::fmt::{Debug, Display};

pub fn bin_name(name: &str) -> String {
    format!("cargo {}", name)
}

pub static VERSION_SHORT: &str = concat!("v", env!("CARGO_PKG_VERSION"));

pub static VERSION_LONG: Lazy<String> = Lazy::new(|| match util::installed_commit_msg() {
    Ok(Some(msg)) => format!("{}\n{}", VERSION_SHORT, util::format_commit_msg(msg)),
    Ok(None) => VERSION_SHORT.to_owned(),
    Err(err) => {
        log::error!("failed to get current commit msg: {}", err);
        VERSION_SHORT.to_owned()
    }
});

#[derive(Clone, Copy, Debug, Parser)]
pub struct GlobalFlags {
    #[arg(
        short,
        long,
        help = "Vomit out extensive logging (-vv for more)",
        global = true,
        action = clap::ArgAction::Count
    )]
    pub verbose: u8,
    #[structopt(short = 'y', long, help = "Never prompt for input", global = true)]
    pub non_interactive: bool,
}

#[derive(Clone, Copy, Debug, Parser)]
pub struct SkipDevTools {
    #[arg(long, help = "Skip optional tools that help when writing code")]
    pub skip_dev_tools: bool,
}

#[derive(Clone, Copy, Debug, Parser)]
pub struct ReinstallDeps {
    #[structopt(long, help = "Reinstall dependencies")]
    pub reinstall_deps: bool,
}

// #[derive(Clone, Copy, Debug, Parser)]
// pub struct Profile {
//     #[structopt(
//         long = "release",
//         help = "Build with release optimizations",
//         parse(from_flag = opts::Profile::from_flag),
//     )]
//     pub profile: opts::Profile,
// }
//
// #[derive(Clone, Copy, Debug, Parser)]
// pub struct Filter {
//     #[structopt(
//         short = "f",
//         long = "filter",
//         help = "Filter logs by level",
//         possible_values = &opts::FilterLevel::variants(),
//         case_insensitive = true,
//     )]
//     pub filter: Option<opts::FilterLevel>,
// }
//

pub mod colors {
    use colored::Color::{self, *};

    pub const ERROR: Color = BrightRed;
    pub const WARNING: Color = BrightYellow;
    pub const ACTION_REQUEST: Color = BrightMagenta;
    pub const VICTORY: Color = BrightGreen;
}

#[derive(Clone, Copy, Debug)]
pub enum Label {
    Error,
    ActionRequest,
    Victory,
}

impl Label {
    pub fn color(&self) -> colored::Color {
        match self {
            Self::Error => colors::ERROR,
            Self::ActionRequest => colors::ACTION_REQUEST,
            Self::Victory => colors::VICTORY,
        }
    }

    pub fn exit_code(&self) -> i8 {
        match self {
            Self::Victory => 0,
            _ => 1,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::ActionRequest => "action request",
            Self::Victory => "victory",
        }
    }
}

#[derive(Debug)]
pub struct Report {
    label: Label,
    msg: String,
    details: String,
}

impl Report {
    pub fn new(label: Label, msg: impl Display, details: impl Display) -> Self {
        Self {
            label,
            msg: format!("{}", msg),
            details: format!("{}", details),
        }
    }

    pub fn error(msg: impl Display, details: impl Display) -> Self {
        Self::new(Label::Error, msg, details)
    }

    pub fn action_request(msg: impl Display, details: impl Display) -> Self {
        Self::new(Label::ActionRequest, msg, details)
    }

    pub fn victory(msg: impl Display, details: impl Display) -> Self {
        Self::new(Label::Victory, msg, details)
    }

    pub fn exit_code(&self) -> i8 {
        self.label.exit_code()
    }

    fn format(&self, option: &textwrap::Options) -> String {
        static INDENT: &str = "    ";
        let head = if colored::control::SHOULD_COLORIZE.should_colorize() {
            textwrap::fill(
                &format!(
                    "{} {}",
                    format!("{}:", self.label.as_str())
                        .color(self.label.color())
                        .bold(),
                    self.msg.color(self.label.color())
                ),
                option,
            )
        } else {
            textwrap::fill(&format!("{}: {}", self.label.as_str(), &self.msg), option)
        };
        let option = option
            .clone()
            .initial_indent(INDENT)
            .subsequent_indent(INDENT);
        format!("{}\n{}\n", head, textwrap::fill(&self.details, option))
    }

    pub fn print(&self, option: &textwrap::Options) {
        let s = self.format(option);
        if matches!(self.label, Label::Error) {
            eprint!("{}", s)
        } else {
            print!("{}", s)
        }
    }
}

pub trait Reportable: Debug {
    fn report(&self) -> Report;
}

pub trait Exec: Debug + Parser {
    type Report: Reportable;

    fn global_flags(&self) -> GlobalFlags;

    fn exec(self, option: &textwrap::Options) -> Result<(), Self::Report>;
}

fn init_logging(noise_level: u8) {
    use env_logger::{Builder, Env};
    let default_level = match noise_level {
        0 => "warn",
        1 => "cargo_mobile=info,cargo_android=info,cargo_apple=info,bossy=info,hit=info",
        _ => "info,cargo_mobile=debug,cargo_android=debug,cargo_apple=debug,bossy=debug,hit=debug",
    };
    let env = Env::default().default_filter_or(default_level);
    Builder::from_env(env).init();
}

#[derive(Debug)]
pub enum Exit {
    Report(Report),
    Clap(clap::Error),
}

impl Exit {
    fn report(reportable: impl Reportable) -> Self {
        log::info!("exiting with {:#?}", reportable);
        Self::Report(reportable.report())
    }

    fn do_the_thing(self, option: textwrap::Options) -> ! {
        match self {
            Self::Report(report) => {
                report.print(&option);
                std::process::exit(report.label.exit_code().into())
            }
            Self::Clap(err) => err.exit(),
        }
    }

    pub fn main(inner: impl FnOnce(&textwrap::Options) -> Result<(), Self>) {
        let option = textwrap::Options::with_termwidth()
            .word_splitter(textwrap::WordSplitter::NoHyphenation);
        if let Err(exit) = inner(&option) {
            exit.do_the_thing(option)
        }
    }
}

pub fn exec<E: Exec>() {
    Exit::main(|wrapper| {
        let input = E::parse();
        init_logging(input.global_flags().verbose);
        log::debug!("raw args: {:#?}", input);
        input.exec(wrapper).map_err(Exit::report)
    })
}
