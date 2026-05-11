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
use config::ConfigSource;
use model::Report;
use output::{print_report, print_rules};
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
        CliCommand::Rules(rules_options) => {
            print_rules(rules_options.format)?;
            return Ok(0);
        }
        CliCommand::Help => {
            println!("{}", help_text());
            return Ok(0);
        }
        CliCommand::Version => {
            println!("flavor {}", build_version());
            return Ok(0);
        }
    };
    let (config, source) = config::resolve(options.root, options.config)?;
    if let ConfigSource::Discovered(path) = &source {
        eprintln!("flavor: using config {}", path.display());
    }
    let allow_empty_scan = config.allow_empty_scan();
    let scan = run_scan(&config)?;
    let report = if allow_empty_scan {
        Report::with_scan_allow_empty(config.root, scan.stats, scan.issues)
    } else {
        Report::with_scan(config.root, scan.stats, scan.issues)
    };

    print_report(&report, options.format)?;

    if report.is_empty_scan() {
        eprintln!(
            "flavor: warning: scan.include matched 0 files — check the patterns in your config and --root",
        );
    }

    Ok(report.exit_code(options.strict_warnings))
}

fn build_version() -> &'static str {
    option_env!("FLAVOR_BUILD_VERSION").unwrap_or(concat!("v", env!("CARGO_PKG_VERSION")))
}
