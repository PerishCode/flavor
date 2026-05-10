use std::{env, process::exit};

mod cli;
mod config;
mod model;
mod naming;
mod output;
mod path_match;
mod rules;
mod rust_tests;
mod scan;

use cli::{help_text, parse_args, CliCommand};
use config::GuardConfig;
use model::Report;
use output::print_report;
use scan::run_scan;

fn main() {
    match run() {
        Ok(exit_code) => exit(exit_code),
        Err(error) => {
            eprintln!("flavor: {error}");
            exit(1);
        }
    }
}

fn run() -> Result<i32, String> {
    let options = parse_args(env::args().skip(1).collect())?;
    let options = match options {
        CliCommand::Check(options) => options,
        CliCommand::Help => {
            println!("{}", help_text());
            return Ok(0);
        }
        CliCommand::Version => {
            println!("flavor {}", build_version());
            return Ok(0);
        }
    };
    let config = match options.config {
        Some(config_path) => GuardConfig::from_file(options.root, &config_path)?,
        None => GuardConfig::core(options.root),
    };
    let scan = run_scan(&config)?;
    let report = Report::with_scan(config.root, scan.stats, scan.issues);
    let deny_count = report.deny_count();
    let warning_count = report.warning_count();

    print_report(&report, options.format)?;

    if deny_count > 0 || (options.strict_warnings && warning_count > 0) {
        return Ok(1);
    }
    Ok(0)
}

fn build_version() -> &'static str {
    option_env!("FLAVOR_BUILD_VERSION").unwrap_or(concat!("v", env!("CARGO_PKG_VERSION")))
}
