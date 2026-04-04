use std::{
    fs::{create_dir_all, remove_file, OpenOptions},
    io,
    sync::atomic::{AtomicBool, Ordering},
};

use log::LevelFilter;
use simplelog::{CombinedLogger, ConfigBuilder, WriteLogger};

const LOG_FILE_PATH: &str = "data/sedidata-tui.log";
static LOGGER_INITIALISED: AtomicBool = AtomicBool::new(false);

pub fn init() -> io::Result<()> {
    let _ = remove_file(LOG_FILE_PATH);
    log::set_max_level(LevelFilter::Off);
    Ok(())
}

pub fn set_enabled(enabled: bool) {
    if enabled {
        if !LOGGER_INITIALISED.load(Ordering::Acquire) && init_logger().is_err() {
            return;
        }
        log::set_max_level(LevelFilter::Debug);
        log::info!("Debug logging enabled (path: {})", LOG_FILE_PATH);
    } else {
        log::info!("Debug logging disabled");
        log::set_max_level(LevelFilter::Off);
    }
}

fn init_logger() -> io::Result<()> {
    create_dir_all("data")?;
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(LOG_FILE_PATH)?;

    let config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_thread_level(LevelFilter::Debug)
        .build();

    CombinedLogger::init(vec![WriteLogger::new(LevelFilter::Debug, config, log_file)])
        .map_err(|error| io::Error::other(error.to_string()))?;

    LOGGER_INITIALISED.store(true, Ordering::Release);
    Ok(())
}
