use std::fs::File;
use std::error::Error;
use simplelog::{Config, WriteLogger, LogLevelFilter};

#[cfg(debug_assertions)]
fn get_level() -> LogLevelFilter { LogLevelFilter::Debug }

#[cfg(not(debug_assertions))]
fn get_level() -> LogLevelFilter { LogLevelFilter::Error }

pub fn init() -> Result<(), Box<Error>> {
    let _ = WriteLogger::init(get_level(), Config::default(), File::create("/tmp/test_logger.log")?)?;
    info!("Starting");
    Ok(())
}
