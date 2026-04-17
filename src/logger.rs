use std::time::SystemTime;

use eyre::Result;

use crate::cli;

pub fn init(cli: &cli::Cli) -> Result<(), fern::InitError> {
    let mut log = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(cli.log_level)
        .chain(fern::log_file(&cli.log_path)?);
    if cli.log_stdout {
        log = log.chain(std::io::stdout());
    }
    log.apply()?;
    Ok(())
}
