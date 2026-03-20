use std::{
    fs::{create_dir_all, OpenOptions},
    io,
};

use log::LevelFilter;
use simplelog::{CombinedLogger, ConfigBuilder, WriteLogger};

const LOG_FILE_PATH: &str = "data/sedidata-tui.log";

pub fn init() -> io::Result<()> {
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

    // Start with logging disabled. The runtime toggle follows the "store responses" state.
    log::set_max_level(LevelFilter::Off);
    Ok(())
}

pub fn set_enabled(enabled: bool) {
    if enabled {
        log::set_max_level(LevelFilter::Debug);
        log::info!("Debug logging enabled (path: {})", LOG_FILE_PATH);
    } else {
        log::info!("Debug logging disabled");
        log::set_max_level(LevelFilter::Off);
    }
}
