use chrono::{Utc, Duration};
use clap::Clap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Result, Error, ErrorKind, Write};
use std::path::PathBuf;

fn log_location() -> PathBuf {
    if cfg!(target_os = "macos") {
        let home = env::var("HOME").unwrap_or("/tmp".to_string());
        [&home, "Library", "Application Support", "StreamLogger"].iter().collect()
    } else if cfg!(target_os = "windows") {
        let home = env::var("LOCALAPPDATA")
            .or(env::var("TEMP"))
            .unwrap_or(".".to_string());
        [&home, "StreamLogger"].iter().collect()
    } else if cfg!(target_os = "linux") {
        env::var("XDG_DATA_HOME")
            .map(|s| PathBuf::from(s))
            .ok()
            .unwrap_or({
                let home = env::var("HOME").unwrap_or("/tmp".to_string());
                [&home, ".local", "share", "StreamLogger"].iter().collect()
            })
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
#[clap(name = "slog", author, about, version)]
struct StreamLogger {
    /// The content of the log.
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
    /// Start logging for a new stream.
    Start,
    /// Print out timestamp for a stream archive.
    Stamp(Stamp),
}

#[derive(Clap)]
struct Stamp {
    /// Shift the timestamps in log forward by this much. It should be in the format "HH:MM:SS".
    /// Hours and minutes can be skipped if not applicable.
    #[clap(short, long)]
    shift: Option<String>,
//    #[clap(short, long)]
//    log: Option<String>,
}


impl Stamp {
    /// Take "HH:MM:SS" string and turn it into seconds.
    /// "SS" and "MM:SS" can be valid input as well and will be interpreted as such.
    fn shift_in_secs(&self) -> Option<i64> {
        let segs: Vec<&str> = self.shift.as_ref()?.split(":").collect();
        match segs.len() {
            1 => segs[0].parse::<i64>().ok(),
            2 => {
                if let Ok(m) = segs[0].parse::<i64>() {
                    if let Ok(s) = segs[1].parse::<i64>() {
                        return Some(m * 60 + s);
                    }
                }

                None

            },
            3 => {
                if let Ok(h) = segs[0].parse::<i64>() {
                    if let Ok(m) = segs[1].parse::<i64>() {
                        if let Ok(s) = segs[2].parse::<i64>() {
                            return Some(h * 3600 + m * 60 + s);
                        }
                    }
                }

                None
            },
            _ => None,
        }

    }

    fn run(&self) -> Result<()> {
        let shift = self.shift_in_secs().unwrap_or(0);
        if let Some(log_path) = find_latest_log() {
            let bytes = fs::read(log_path)?;
            let all_log = String::from_utf8_lossy(&bytes);
            let mut first_timestamp: Option<i64> = None;
            for line in all_log.lines() {
                let mut segs = line.split(',');
                if let Some(time) = segs.next() {
                    if let Ok(timestamp) = time.parse::<i64>() {
                        let reference_timestamp: i64;
                        match first_timestamp {
                            None => {
                                reference_timestamp = timestamp;
                                first_timestamp = Some(timestamp);
                            },
                            Some(first) => {
                                reference_timestamp = first;
                            },
                        }

                        if let Some(log) = segs.next() {
                            let duration = Duration::seconds(timestamp - reference_timestamp + shift);
                            println!(
                                "{}:{:02}:{:02} {}",
                                duration.num_hours(),
                                duration.num_minutes() % 60,
                                duration.num_seconds() % 60,
                                log
                            );
                        }
                    }
                }
            }

            return Ok(());
        }

        return Err(Error::new(ErrorKind::Other, "could not log message"));
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

#[cfg(test)]
mod test {
    use super::{Stamp};

    #[test]
    fn shift_to_secs_parsing_secs_only() {
        let cmd = Stamp {
            shift: Some("44".to_string()),
        };

        assert_eq!(cmd.shift_in_secs(), Some(44));
    }

    #[test]
    fn shift_to_secs_parsing_mins_secs() {
        let cmd = Stamp {
            shift: Some("02:44".to_string()),
        };

        assert_eq!(cmd.shift_in_secs(), Some(164));
    }

    #[test]
    fn shift_to_secs_parsing_hrs_mins_secs() {
        let cmd = Stamp {
            shift: Some("01:02:44".to_string()),
        };

        assert_eq!(cmd.shift_in_secs(), Some(3764));
    }
}
