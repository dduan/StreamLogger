use chrono::{Utc};
use std::env;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::io::{Result, Error, ErrorKind, Write};

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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("USAGE: {} LOG_MESSAGE", args[0]);
        return Ok(())
    }

    let log_msg = &args[1];

    if log_msg == "start" {
        match start_new_log() {
            Err(err) => {
                return Err(err);
            },
            Ok(path) => {
                println!("Created stream log at {}", path);
                return Ok(());
            }
        }
    } else {
        if let Some(log_path) = find_latest_log() {
            if let Ok(mut log) = OpenOptions::new()
                .append(true)
                .open(log_path)
            {
                return write!(log, "{}\n{},", args[1], Utc::now().timestamp());
            }
        }

        return Err(Error::new(ErrorKind::Other, "could not log message"));
    }
}
