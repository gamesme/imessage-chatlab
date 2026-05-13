//! `imessage-chatlab` CLI entry point.

mod avatar;
mod compatibility;
mod contacts;
mod data_source;
mod error;
mod exporter;
mod logging;
mod options;
mod preflight;
mod preview;
mod progress;
mod query;
mod runtime;

use std::process::exit;

use crate::error::RuntimeError;
use crate::options::Options;
use crate::runtime::Config;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        exit(1);
    }
}

fn run() -> Result<(), RuntimeError> {
    let matches = options::cli().get_matches();
    let opts = Options::from_args(&matches)?;
    crate::logging::set_quiet(opts.quiet);
    crate::preflight::check_db_readable(&opts.get_db_path())?;
    let mut config = Config::new(opts)?;
    // Without this call, --conversation-filter never populates the query context
    // and `start()` rejects the filter as not matching any participants.
    config.resolve_filtered_handles();
    config.start()
}
