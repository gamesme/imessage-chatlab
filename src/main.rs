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
mod wizard;

use std::io::IsTerminal;

use crate::error::RuntimeError;
use crate::options::Options;
use crate::runtime::Config;

fn run() -> Result<(), RuntimeError> {
    let matches = options::cli().get_matches();

    // Subcommand dispatch
    if let Some((sub, sub_matches)) = matches.subcommand() {
        return match sub {
            "list" => run_list(sub_matches),
            "export" => run_export(sub_matches),
            _ => unreachable!("unknown subcommand: {sub}"),
        };
    }

    // No subcommand. Decide wizard vs flag-driven.
    let no_flags = is_no_flags_invocation(&matches);
    let is_tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();

    if no_flags && is_tty {
        let lang = matches
            .get_one::<String>(options::OPTION_LANG)
            .map(|s| s.as_str());
        let opts = wizard::run(lang)?;
        crate::logging::set_quiet(opts.quiet);
        crate::preflight::check_db_readable(&opts.get_db_path())?;
        let mut config = Config::new(opts)?;
        config.resolve_filtered_handles();
        return config.start();
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
    config.resolve_filtered_handles();
    config.start()
}

/// True when the user invoked the program with no flags or only --lang
/// (which can sit alongside the wizard).
fn is_no_flags_invocation(matches: &clap::ArgMatches) -> bool {
    matches
        .ids()
        .all(|id| id.as_str() == options::OPTION_LANG)
}

fn main() {
    if let Err(err) = run() {
        match err {
            RuntimeError::WizardCancelled => std::process::exit(130),
            other => {
                eprintln!("{other}");
                std::process::exit(1);
            }
        }
    }
}
