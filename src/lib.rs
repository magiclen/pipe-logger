//! # Pipe Logger
//! Stores, rotates, compresses process logs.

extern crate clap;
extern crate byte_unit;
extern crate path_absolutize;
extern crate chrono;
extern crate regex;
extern crate xz2;

use std::env;
use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions, File};
use std::io::{self, Read, Write};
use std::thread;

use regex::Regex;

use byte_unit::Byte;
use byte_unit::ByteError;

use path_absolutize::*;

use chrono::prelude::*;

use xz2::write::XzEncoder;

use clap::{App, Arg};

// TODO -----Config START-----

const APP_NAME: &str = "Pipe Logger";
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const CARGO_PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DEFAULT_LOG_NAME: &str = "logfile.log";
const BUFFER_SIZE: usize = 4096 * 4;

#[derive(Debug)]
pub enum RotateMethod {
    FileSize(u64)
}

#[derive(Debug)]
pub struct Config {
    pub rotate: Option<RotateMethod>,
    pub count: Option<usize>,
    pub log_path: String,
    pub compress: bool,
    pub err: bool,
}

impl Config {
    pub fn from_cli() -> Result<Config, String> {
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

        let rotate = match matches.value_of("ROTATE") {
            Some(r) => {
                match Byte::from_string(r) {
                    Ok(byte) => {
                        let b = byte.get_bytes();
                        if b > <u64>::max_value() as u128 {
                            return Err("The file size of rotation is too large.".to_string());
                        } else if b < 2 {
                            return Err("The file size of rotation is too small.".to_string());
                        }
                        Some(RotateMethod::FileSize(b as u64))
                    }
                    Err(err) => {
                        match err {
                            ByteError::ParseError => {
                                // TODO other methods
                                return Err("You need to input a file size for log rotation.".to_string());
                            }
                            ByteError::ValueIncorrect => {
                                return Err("The file size of rotation is incorrect.".to_string());
                            }
                            ByteError::UnitIncorrect => {
                                return Err("The unit of the file size is incorrect.".to_string());
                            }
                        }
                    }
                }
            }
            None => {
                None
            }
        };

        let count = if rotate.is_some() {
            match matches.value_of("COUNT") {
                Some(c) => {
                    match c.parse::<usize>() {
                        Ok(c) => {
                            if c < 1 {
                                return Err("The count of log files is too small.".to_string());
                            }
                            Some(c)
                        }
                        Err(err) => {
                            return Err(err.to_string());
                        }
                    }
                }
                None => {
                    None
                }
            }
        } else {
            None
        };

        let compress = rotate.is_some() && matches.is_present("COMPRESS");

        let err = matches.is_present("ERR");

        let log_path = match Path::new(matches.value_of("LOG_PATH").unwrap()).absolutize() {
            Ok(p) => {
                p
            }
            Err(err) => {
                return Err(err.to_string());
            }
        };

        if log_path.exists() {
            if log_path.is_dir() {
                return Err(format!("`{}` is a directory.", log_path.to_str().unwrap()));
            }
            match fs::metadata(&log_path) {
                Ok(m) => {
                    let p = m.permissions();
                    if p.readonly() {
                        return Err(format!("`{}` is readonly.", log_path.to_str().unwrap()));
                    }
                }
                Err(err) => {
                    return Err(err.to_string());
                }
            }

            if rotate.is_some() {
                match log_path.parent() {
                    Some(parent) => {
                        match fs::metadata(&parent) {
                            Ok(m) => {
                                let p = m.permissions();
                                if p.readonly() {
                                    return Err(format!("`{}` is readonly.", parent.to_str().unwrap()));
                                }
                            }
                            Err(err) => {
                                return Err(err.to_string());
                            }
                        }
                    }
                    None => {
                        panic!("impossible");
                    }
                }
            }
        } else {
            match log_path.parent() {
                Some(parent) => {
                    match fs::metadata(&parent) {
                        Ok(m) => {
                            let p = m.permissions();
                            if p.readonly() {
                                return Err(format!("`{}` is readonly.", parent.to_str().unwrap()));
                            }
                        }
                        Err(err) => {
                            return Err(err.to_string());
                        }
                    }
                }
                None => {
                    return Err(format!("`{}`'s parent does not exist.", log_path.to_str().unwrap()));
                }
            };
        }

        let log_path = log_path.to_str().unwrap().to_string();

        Ok(
            Config {
                rotate,
                count,
                log_path,
                compress,
                err,
            }
        )
    }
}

// TODO -----Config END-----

pub fn run(config: Config) -> Result<i32, String> {
    let file_path = Path::new(&config.log_path);

    let file_parent = file_path.parent().unwrap();

    let mut file_size = match fs::metadata(&file_path) {
        Ok(m) => {
            m.len()
        }
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
            0
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let file_name = Path::new(&file_path).file_name().unwrap().to_str().unwrap();

    let file_name_point_index = match file_name.rfind(".") {
        Some(index) => {
            index
        }
        None => {
            file_name.len()
        }
    };

    let file_name_without_extension = Path::new(&file_path).file_stem().unwrap().to_str().unwrap();

    let mut rotated_log_file_names = {
        let mut rotated_log_file_names = Vec::new();

        let re = Regex::new("^-[1-2][0-9]{3}(-[0-5][0-9]){5}-[0-9]{3}$").unwrap(); // -%Y-%m-%d-%H-%M-%S + $.3f

        for entry in file_parent.read_dir().unwrap().filter_map(|entry| entry.ok()) {
            let rotated_log_file_path = entry.path();

            if !rotated_log_file_path.is_file() {
                continue;
            }

            let rotated_log_file_name = Path::new(&rotated_log_file_path).file_name().unwrap().to_str().unwrap();

            if !rotated_log_file_name.starts_with(file_name_without_extension) {
                continue;
            }

            let rotated_log_file_name_point_index = match rotated_log_file_name.rfind(".") {
                Some(index) => {
                    index
                }
                None => {
                    rotated_log_file_name.len()
                }
            };

            if rotated_log_file_name_point_index < file_name_point_index + 24 { // -%Y-%m-%d-%H-%M-%S + $.3f
                continue;
            }

            let file_name_without_extension_len = file_name_without_extension.len();

            if !re.is_match(&rotated_log_file_name[file_name_without_extension_len..file_name_without_extension_len + 24]) {  // -%Y-%m-%d-%H-%M-%S + $.3f
                continue;
            }

            let ext = &rotated_log_file_name[rotated_log_file_name_point_index..];

            if ext.eq(&file_name[file_name_point_index..]) {
                rotated_log_file_names.push(rotated_log_file_name.to_string());
            } else if ext.eq(".xz") && rotated_log_file_name[..rotated_log_file_name_point_index].ends_with(&file_name[file_name_point_index..]) {
                rotated_log_file_names.push(rotated_log_file_name[..rotated_log_file_name_point_index].to_string());
            }
        }

        rotated_log_file_names.sort();

        rotated_log_file_names
    };

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&file_path) {
        Ok(f) => {
            f
        }
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut input = String::new();
    let stdin = &io::stdin();
    loop {
        match stdin.read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                if config.err {
                    eprint!("{}", input);
                } else {
                    print!("{}", input);
                }

                match file.write(input.as_bytes()) {
                    Ok(n) => {
                        if n != input.len() {
                            return Err("The space is not enough.".to_string());
                        }
                    }
                    Err(err) => {
                        return Err(err.to_string());
                    }
                }

                file_size += input.len() as u64;

                if let Some(rotate) = &config.rotate {
                    match rotate {
                        RotateMethod::FileSize(size) => {
                            if file_size >= *size {
                                let utc: DateTime<Utc> = Utc::now();
                                let timestamp = utc.format("%Y-%m-%d-%H-%M-%S").to_string();
                                let millisecond = utc.format("%.3f").to_string();

                                if let Err(err) = file.flush() {
                                    return Err(err.to_string());
                                }
                                if let Err(err) = file.sync_all() {
                                    return Err(err.to_string());
                                }
                                drop(file);

                                let rotated_log_file_name = format!("{}-{}-{}{}", file_name_without_extension, timestamp, &millisecond[1..], &file_name[file_name_point_index..]);

                                let rotated_log_file = Path::join(file_parent, Path::new(&rotated_log_file_name));

                                if let Err(err) = fs::copy(&file_path, &rotated_log_file) {
                                    return Err(err.to_string());
                                }

                                if config.compress {
                                    let rotated_log_file_name_compressed = format!("{}.xz", rotated_log_file_name);
                                    let rotated_log_file_compressed = Path::join(file_parent, Path::new(&rotated_log_file_name_compressed));
                                    let config_err = config.err;

                                    thread::spawn(move || {
                                        match File::create(&rotated_log_file_compressed) {
                                            Ok(file_w) => {
                                                match File::open(&rotated_log_file) {
                                                    Ok(mut file_r) => {
                                                        let mut compressor = XzEncoder::new(file_w, 9);
                                                        let mut buffer = [0u8; BUFFER_SIZE];
                                                        loop {
                                                            match file_r.read(&mut buffer) {
                                                                Ok(c) => {
                                                                    if c == 0 {
                                                                        if let Err(_) = fs::remove_file(&rotated_log_file) {}
                                                                        break;
                                                                    }
                                                                    match compressor.write(&buffer[..c]) {
                                                                        Ok(cc) => {
                                                                            if c != cc {
                                                                                if config_err {
                                                                                    println!("The space is not enough.");
                                                                                } else {
                                                                                    eprintln!("The space is not enough.");
                                                                                }
                                                                                break;
                                                                            }
                                                                        }
                                                                        Err(err) => {
                                                                            if config_err {
                                                                                println!("{}", err.to_string());
                                                                            } else {
                                                                                eprintln!("{}", err.to_string());
                                                                            }
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                                Err(err) => {
                                                                    if config_err {
                                                                        println!("{}", err.to_string());
                                                                    } else {
                                                                        eprintln!("{}", err.to_string());
                                                                    }
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        if config_err {
                                                            println!("{}", err.to_string());
                                                        } else {
                                                            eprintln!("{}", err.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                if config_err {
                                                    println!("{}", err.to_string());
                                                } else {
                                                    eprintln!("{}", err.to_string());
                                                }
                                            }
                                        };
                                    });
                                }

                                rotated_log_file_names.push(rotated_log_file_name);

                                if let Some(count) = config.count {
                                    while rotated_log_file_names.len() >= count {
                                        let rotated_log_file_name = rotated_log_file_names.remove(0);
                                        if let Err(_) = fs::remove_file(Path::join(file_parent, Path::new(&rotated_log_file_name))) {}

                                        let p_compressed_name = format!("{}.xz", rotated_log_file_name);

                                        let p_compressed = Path::join(file_parent, Path::new(&p_compressed_name));
                                        if let Err(_) = fs::remove_file(&p_compressed) {}
                                    }
                                }

                                file = match OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .open(&file_path) {
                                    Ok(f) => {
                                        f
                                    }
                                    Err(err) => {
                                        return Err(err.to_string());
                                    }
                                };
                            }
                        }
                    }
                }
            }
            Err(err) => {
                return Err(err.to_string());
            }
        }
        input.clear();
    }

    Ok(0)
}