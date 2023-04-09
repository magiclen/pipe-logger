use std::{
    env,
    error::Error,
    io::{self, Write},
};

use byte_unit::Byte;
use clap::{Arg, Command};
use concat_with::concat_line;
use pipe_logger_lib::*;
use terminal_size::terminal_size;

const APP_NAME: &str = "Pipe Logger";
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

const DEFAULT_LOG_NAME: &str = "logfile.log";

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new(APP_NAME)
        .term_width( terminal_size().map(|(width, _)| width.0 as usize).unwrap_or(0))
        .version(CARGO_PKG_VERSION)
        .author(CARGO_PKG_AUTHORS)
        .about(concat!("Stores, rotates, compresses process logs.\n\nEXAMPLES:\n", concat_line!(prefix "pipe-logger ",
                "/path/to/out.log                        # Store log into /path/to/out.log",
                "/path/to/out.log -r 10M                 # The same as above, plus if its size is over than 10MB, it will be rotated and renamed.",
                "/path/to/out.log -r 10M -c 4            # The same as above, plus the max count of log files is 4. The oldest ones will be removed when the quota is exhausted.",
                "/path/to/out.log -r 10M -c 4 --compress # The same as above, plus the rotated log files are compressed by xz.",
            )))
        .arg(
            Arg::new("ROTATE")
                .long("rotate")
                .short('r')
                .help("Rotate the log file.")
                .takes_value(true),
        )
        .arg(
            Arg::new("COUNT")
                .long("count")
                .short('c')
                .help("Assign the max count of log files.")
                .takes_value(true),
        )
        .arg(
            Arg::new("COMPRESS").long("--compress").help("Compress the rotated log files."),
        )
        .arg(Arg::new("ERR").long("--err").help("Re-output logs through stderr."))
        .arg(
            Arg::new("LOG_PATH")
                .help("The path that you want to store your logs.")
                .takes_value(true)
                .default_value(DEFAULT_LOG_NAME),
        )
        .after_help("Enjoy it! https://magiclen.org")
        .get_matches();

    let log_path = matches.value_of("LOG_PATH").unwrap();

    let mut builder = PipeLoggerBuilder::new(log_path);

    if let Some(r) = matches.value_of("ROTATE") {
        let byte = Byte::from_str(r)?;

        builder.set_rotate(Some(RotateMethod::FileSize(byte.get_bytes())));

        builder.set_compress(matches.is_present("COMPRESS"));

        if let Some(c) = matches.value_of("COUNT") {
            builder.set_count(Some(c.parse::<usize>()?));
        }
    }

    if matches.is_present("ERR") {
        builder.set_tee(Some(Tee::Stderr));
    } else {
        builder.set_tee(Some(Tee::Stdout));
    }

    let mut logger = builder.build()?;

    let mut input = String::new();

    let stdin = io::stdin();

    loop {
        let c = stdin.read_line(&mut input)?;

        if c == 0 {
            break;
        }

        logger.write_all(&input.as_bytes()[..c])?;
    }

    Ok(())
}
