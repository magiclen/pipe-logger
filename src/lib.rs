//! # Pipe Logger
//! Stores, rotates, compresses process logs.

extern crate clap;
extern crate byte_unit;
extern crate pipe_logger_lib;

use std::env;
use std::path::{Path, PathBuf};
use std::io;

use byte_unit::Byte;
use byte_unit::ByteError;

use pipe_logger_lib::*;

use clap::{App, Arg};

// TODO -----Config START-----

const APP_NAME: &str = "Pipe Logger";
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DEFAULT_LOG_NAME: &str = "logfile.log";

pub fn from_cli() -> Result<PipeLogger, String> {
    let arg0 = env::args().next().unwrap();
    let arg0 = Path::new(&arg0).file_stem().unwrap().to_str().unwrap();

    let default_log_path = {
        let path = Path::new(&arg0);
        match path.parent() {
            Some(p) => {
                Path::join(p, DEFAULT_LOG_NAME)
            }
            None => {
                PathBuf::from(DEFAULT_LOG_NAME)
            }
        }
    };

    let examples = vec![
        "/path/to/out.log                        # Stores log into /path/to/out.log",
        "/path/to/out.log -r 10M                 # The same as above, plus if its size is over than 10MB, it will be rotated and renamed.",
        "/path/to/out.log -r 10M -c 4            # The same as above, plus the max count of log files is 4. The oldest ones will be removed when the quota is exhausted.",
        "/path/to/out.log -r 10M -c 4 --compress # The same as above, plus the rotated log files are compressed by xz.",
    ];

    let matches = App::new(APP_NAME)
        .version(CARGO_PKG_VERSION)
        .author(CARGO_PKG_AUTHORS)
        .about(format!("Stores, rotates, compresses process logs.\n\nEXAMPLES:\n{}", examples.iter()
            .map(|e| format!("  {} {}\n", arg0, e))
            .collect::<Vec<String>>()
            .concat()
        ).as_str()
        )
        .arg(Arg::with_name("ROTATE")
            .long("rotate")
            .short("r")
            .help("Rotates the log file.")
            .takes_value(true)
        )
        .arg(Arg::with_name("COUNT")
            .long("count")
            .short("c")
            .help("Assigns the max count of log files.")
            .takes_value(true)
        )
        .arg(Arg::with_name("COMPRESS")
            .long("--compress")
            .help("Compresses the rotated log files.")
        )
        .arg(Arg::with_name("ERR")
            .long("--err")
            .help("Re-outputs logs through stderr.")
        )
        .arg(Arg::with_name("LOG_PATH")
            .required(true)
            .help("The path that you want to store your logs.")
            .takes_value(true)
            .default_value(default_log_path.to_str().unwrap())
        )
        .after_help("Enjoy it! https://magiclen.org")
        .get_matches();

    let log_path = matches.value_of("LOG_PATH").unwrap();

    let mut builder = PipeLoggerBuilder::new(log_path);

    if let Some(r) = matches.value_of("ROTATE") {
        match Byte::from_str(r) {
            Ok(byte) => {
                let b = byte.get_bytes();
                if b > <u64>::max_value() as u128 {
                    return Err("The file size of rotation is too large.".to_string());
                }
                builder.set_rotate(Some(RotateMethod::FileSize(b as u64)));
            }
            Err(err) => {
                match err {
                    ByteError::ValueIncorrect(s) => {
                        return Err(s);
                    }
                    ByteError::UnitIncorrect(s) => {
                        return Err(s);
                    }
                }
            }
        }
    }

    if builder.rotate().is_some() {
        builder.set_compress(matches.is_present("COMPRESS"));

        if let Some(c) = matches.value_of("COUNT") {
            match c.parse::<usize>() {
                Ok(c) => {
                    builder.set_count(Some(c));
                }
                Err(err) => {
                    return Err(err.to_string());
                }
            }
        }
    }

    if matches.is_present("ERR") {
        builder.set_tee(Some(Tee::Stderr));
    } else {
        builder.set_tee(Some(Tee::Stdout));
    }

    match builder.build() {
        Ok(logger) => Ok(logger),
        Err(err) => {
            match err {
                PipeLoggerBuilderError::RotateFileSizeTooSmall => {
                    return Err("The file size of rotation is too small.".to_string());
                }
                PipeLoggerBuilderError::CountTooSmall => {
                    return Err("The count of log files is too small.".to_string());
                }
                PipeLoggerBuilderError::IOError(err) => {
                    return Err(err.to_string());
                }
                PipeLoggerBuilderError::FileIsDirectory(path) => {
                    return Err(format!("`{}` is a directory.", path.to_str().unwrap()));
                }
            }
        }
    }
}

// TODO -----Config END-----

pub fn run(mut logger: PipeLogger) -> Result<i32, String> {
    let mut input = String::new();
    let stdin = &io::stdin();
    loop {
        match stdin.read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                logger.write(&input).map_err(|err| err.to_string())?;
            }
            Err(err) => {
                return Err(err.to_string());
            }
        }
        input.clear();
    }

    Ok(0)
}