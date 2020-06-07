use chrono::{Utc};
use clap::Clap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Result, Error, ErrorKind, Write};
use std::path::PathBuf;

fn log_location() -> PathBuf {
    if cfg!(target_os = "macos") {
        let home = env::var("HOME").unwrap_or("/tmp".to_string());
        [&home, "Library", "Application Support", "StreamLogger"].iter().collect()
    } else {
        PathBuf::from(".")
    }

}

fn start_new_log() -> Result<String> {
    let time = Utc::now().timestamp();
    let filename = format!("{}.csv", time);
    let mut filepath = log_location();
    let folder = filepath.as_path();

    if !folder.is_dir() {
        fs::create_dir_all(folder)?;
    }

    filepath.push(filename);
    let path_str = filepath
        .to_str()
        .ok_or(Error::new(ErrorKind::Other, "could not convert path to string"))?;
    fs::write(path_str, format!("{},", time))?;

    Ok(path_str.to_string())
}

fn find_latest_log() -> Option<PathBuf> {
    let mut log_location = log_location();
    let mut latest_file_name_value: Option<i64> = None;
    for entry in fs::read_dir(log_location.to_str()?).ok()? {
        let entry = entry.ok()?;
        let name_value = entry
            .file_name()
            .to_str()?
            .split('.')
            .collect::<Vec<&str>>()[0]
            .parse::<i64>()
            .ok()?;

        match latest_file_name_value {
            None => {
                latest_file_name_value = Some(name_value)
            },
            Some(value) => {
                if value < name_value {
                    latest_file_name_value = Some(name_value)
                }
            }
        }
    }

    latest_file_name_value.map(|base_name| {
        log_location.push(format!("{}.csv", base_name));
        log_location
    })
}

#[derive(Clap)]
#[clap(author, about, version)]
struct StreamLogger {
    log: Option<String>,
    #[clap(subcommand)]
    subcommand: Option<SubCommand>
}

impl StreamLogger {
    fn run(&self) -> Result<()> {
        match &self.log {
            None => {
                eprintln!("Provide a log message.");
                return Err(Error::new(ErrorKind::Other, "missing log message"));
            },
            Some(log_msg) => {
                if let Some(log_path) = find_latest_log() {
                    if let Ok(mut log) = OpenOptions::new()
                        .append(true)
                        .open(log_path)
                    {
                        return write!(log, "{}\n{},", log_msg, Utc::now().timestamp());
                    }
                }
                return Err(Error::new(ErrorKind::Other, "could not log message"));
            },
        }
    }
}

#[derive(Clap)]
enum SubCommand {
    Start,
    Stamp(Stamp),
}

#[derive(Clap)]
struct Stamp {
    #[clap(short, long)]
    shift: Option<String>,
    #[clap(short, long)]
    log: Option<String>,
}

impl Stamp {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

struct Start {}
impl Start {
    fn run() -> Result<()> {
        match start_new_log() {
            Err(err) => {
                Err(err)
            },
            Ok(path) => {
                println!("Created stream log at {}", path);
                Ok(())
            }
        }
    }
}

fn main() -> Result<()> {
    let logger: StreamLogger = StreamLogger::parse();

    match logger.subcommand {
        Some(SubCommand::Start) => {
            Start::run()
        },
        Some(SubCommand::Stamp(stamp)) => {
            stamp.run()
        },
        None => {
            logger.run()
        },
    }
}
