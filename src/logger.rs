use std::fs::File;
use std::error::Error;
use simplelog::{Config, WriteLogger, LogLevelFilter};

pub fn init() -> Result<(), Box<Error>> {
    let _ = WriteLogger::init(LogLevelFilter::Debug, Config::default(), File::create("/tmp/test_logger.log")?)?;
    info!("Starting");
    Ok(())
}
