use std::path::PathBuf;

use clap::{CommandFactory, FromArgMatches, Parser};
use concat_with::concat_line;
use terminal_size::terminal_size;

const APP_NAME: &str = "Pipe Logger";
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

const AFTER_HELP: &str = "Enjoy it! https://magiclen.org";

const APP_ABOUT: &str = concat!(
    "Stores, rotates, compresses process logs.\n\nEXAMPLES:\n",
    concat_line!(prefix "pipe-logger ",
        "/path/to/out.log                          # Store log into /path/to/out.log",
        "/path/to/out.log -r 10M                   # The same as above, plus if its size is over than 10MB, it will be rotated and renamed",
        "/path/to/out.log -r 10M -c 4              # The same as above, plus the max count of log files is 4. The oldest ones will be removed when the quota is exhausted",
        "/path/to/out.log -r 10M -c 4 --compress   # The same as above, plus the rotated log files are compressed by xz",
    )
);

#[derive(Debug, Parser)]
#[command(name = APP_NAME)]
#[command(term_width = terminal_size().map(|(width, _)| width.0 as usize).unwrap_or(0))]
#[command(version = CARGO_PKG_VERSION)]
#[command(author = CARGO_PKG_AUTHORS)]
#[command(after_help = AFTER_HELP)]
pub struct CLIArgs {
    #[arg(short, long)]
    #[arg(help = "Rotate the log file")]
    pub rotate: Option<String>,

    #[arg(short, long)]
    #[arg(help = "Assign the max count of log files")]
    pub count: Option<usize>,

    #[arg(long)]
    #[arg(help = "Compress the rotated log files")]
    pub compress: bool,

    #[arg(long)]
    #[arg(help = "Re-output logs through stderr")]
    pub err: bool,

    #[arg(long)]
    #[arg(default_value = "logfile.log")]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path that you want to store your logs")]
    pub log_path: PathBuf,
}

pub fn get_args() -> CLIArgs {
    let args = CLIArgs::command();

    let about = format!("{APP_NAME} {CARGO_PKG_VERSION}\n{CARGO_PKG_AUTHORS}\n{APP_ABOUT}");

    let args = args.about(about);

    let matches = args.get_matches();

    match CLIArgs::from_arg_matches(&matches) {
        Ok(args) => args,
        Err(err) => {
            err.exit();
        },
    }
}
