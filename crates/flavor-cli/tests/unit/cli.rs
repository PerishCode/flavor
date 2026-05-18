use std::path::PathBuf;

use crate::cli::{help_text, parse_args, CliCommand, LogLevel, OutputFormat, RulesOptions};

#[test]
fn parses_check_options() {
    let CliCommand::Check(options) = parse_args(vec![
        "check".into(),
        "--root".into(),
        "workspace".into(),
        "--format=json".into(),
        "--strict-warnings".into(),
        "--log-level=debug".into(),
    ])
    .unwrap() else {
        panic!("expected check command");
    };

    assert_eq!(options.root, PathBuf::from("workspace"));
    assert_eq!(options.format, OutputFormat::Json);
    assert!(options.strict_warnings);
    assert_eq!(options.log_level, LogLevel::Debug);
}

#[test]
fn command_can_be_omitted() {
    let CliCommand::Check(options) = parse_args(Vec::new()).unwrap() else {
        panic!("expected check command");
    };

    assert_eq!(options.root, PathBuf::from("."));
    assert_eq!(options.format, OutputFormat::Text);
    assert!(!options.strict_warnings);
    assert_eq!(options.log_level, LogLevel::Off);
}

#[test]
fn help_command_is_parsed() {
    assert_eq!(parse_args(vec!["help".into()]).unwrap(), CliCommand::Help);
}

#[test]
fn rules_defaults_to_text() {
    assert_eq!(
        parse_args(vec!["rules".into()]).unwrap(),
        CliCommand::Rules(RulesOptions {
            format: OutputFormat::Text,
            log_level: LogLevel::Off,
        })
    );
}

#[test]
fn rules_honours_format_flag() {
    assert_eq!(
        parse_args(vec!["rules".into(), "--format=json".into()]).unwrap(),
        CliCommand::Rules(RulesOptions {
            format: OutputFormat::Json,
            log_level: LogLevel::Off,
        })
    );
}

#[test]
fn rules_log_level_flag() {
    assert_eq!(
        parse_args(vec![
            "rules".into(),
            "--format=json".into(),
            "--log-level".into(),
            "trace".into(),
        ])
        .unwrap(),
        CliCommand::Rules(RulesOptions {
            format: OutputFormat::Json,
            log_level: LogLevel::Trace,
        })
    );
}

#[test]
fn version_command_is_parsed() {
    assert_eq!(
        parse_args(vec!["--version".into()]).unwrap(),
        CliCommand::Version
    );
}

#[test]
fn help_stays_operational() {
    let help = help_text();

    assert!(help.contains("check [--root <path>]"));
    assert!(help.contains("rules [--format text|json]"));
    assert!(help.contains("--log-level=debug"));
    assert!(help.contains("rule-level bad-flavor notes"));
    assert!(help.contains("Rust, TypeScript, TSX, Vue, and Svelte"));
    assert!(help.contains("Source:  https://github.com/PerishCode/flavor"));
    assert!(help.contains("Issues:  https://github.com/PerishCode/flavor/issues"));
    assert!(help.contains("AGENTS.md"));
    assert!(!help.contains("Preferred repair"));
    assert!(help.contains("does not format, rewrite"));
}

#[test]
fn json_format_available() {
    let CliCommand::Check(options) =
        parse_args(vec!["check".into(), "--format".into(), "json".into()]).unwrap()
    else {
        panic!("expected check command");
    };

    assert_eq!(options.format, OutputFormat::Json);
}
