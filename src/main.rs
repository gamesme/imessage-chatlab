//! `imessage-chatlab` CLI entry point.

mod avatar;
mod compatibility;
mod contacts;
mod data_source;
mod error;
mod exporter;
mod list;
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

    if let Some(("list", sub_matches)) = matches.subcommand() {
        return run_list(sub_matches);
    }

    run_export(&matches)
}

fn run_list(matches: &clap::ArgMatches) -> Result<(), RuntimeError> {
    let mut opts = Options::default_for_list();
    if matches.get_flag("quiet") {
        opts.quiet = true;
    }
    crate::logging::set_quiet(opts.quiet);
    crate::preflight::check_db_readable(&opts.get_db_path())?;
    let mut config = Config::new(opts)?;
    config.resolve_filtered_handles();
    let json = matches.get_flag("json");
    crate::list::run(&config, json)
}

fn run_export(matches: &clap::ArgMatches) -> Result<(), RuntimeError> {
    let opts = Options::from_args(matches)?;
    crate::logging::set_quiet(opts.quiet);
    crate::preflight::check_db_readable(&opts.get_db_path())?;
    let mut config = Config::new(opts)?;
    // Without this call, --conversation-filter never populates the query context
    // and `start()` rejects the filter as not matching any participants.
    config.resolve_filtered_handles();
    config.start()
}
