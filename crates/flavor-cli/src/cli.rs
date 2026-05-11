use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CliOptions {
    pub(crate) root: PathBuf,
    pub(crate) config: Option<PathBuf>,
    pub(crate) format: OutputFormat,
    pub(crate) strict_warnings: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum CliCommand {
    Check(CliOptions),
    Help,
    Version,
}

pub(crate) fn parse_args(args: Vec<String>) -> Result<CliCommand, String> {
    let mut root = PathBuf::from(".");
    let mut config = None;
    let mut format = OutputFormat::Text;
    let mut strict_warnings = false;
    let mut command_seen = false;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "check" if !command_seen => {
                command_seen = true;
            }
            "help" | "--help" | "-h" => {
                return Ok(CliCommand::Help);
            }
            "version" | "--version" | "-V" => {
                return Ok(CliCommand::Version);
            }
            "--root" => {
                root = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--root requires a value".to_string())?,
                );
            }
            "--config" => {
                config = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--config requires a value".to_string())?,
                ));
            }
            "--format" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--format requires text or json".to_string())?;
                format = parse_format(&value)?;
            }
            "--strict-warnings" => {
                strict_warnings = true;
            }
            other if other.starts_with("--root=") => {
                root = PathBuf::from(&other["--root=".len()..]);
            }
            other if other.starts_with("--config=") => {
                config = Some(PathBuf::from(&other["--config=".len()..]));
            }
            other if other.starts_with("--format=") => {
                format = parse_format(&other["--format=".len()..])?;
            }
            other => {
                return Err(format!(
                    "unsupported flavor argument: {other}\n\n{}",
                    help_text()
                ))
            }
        }
    }

    Ok(CliCommand::Check(CliOptions {
        root,
        config,
        format,
        strict_warnings,
    }))
}

pub(crate) fn help_text() -> &'static str {
    r#"flavor

Check-only codestyle attributes for the personal flavor boundary.
It does not format, rewrite, run services, or manage runtime state.

Commands:
  check [--root <path>] [--config <path>] [--format text|json] [--strict-warnings]
  help
  version

Config:
  --config <path>  Load this JSON config. When omitted, flavor looks for
                   flavor.json at the scan root and falls back to built-in
                   include/exclude patterns if the file is absent.

Scope:
  The check covers handwritten Rust, TypeScript, TSX, Vue, and Svelte source
  through scan.include / scan.exclude path patterns. Rule behavior is adjusted
  through ordered overrides that match files or directories and set rule
  payload, severity, or enabled state.

Reports:
  check reports include rule-level bad-flavor notes and action hints when
  issues exist. The hints are review pressure, not automatic fix instructions.

Exit codes:
  0  scan matched at least one file and produced no deny issues (and no
     warnings when --strict-warnings is set).
  1  deny issues, strict-warning failure, or scan.include matched 0 files
     (which usually means the config or --root is wrong).

Feedback:
  Report parser gaps, rule noise, and install issues at:
  https://github.com/PerishCode/flavor/issues
"#
}

fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(format!("unsupported output format: {other}")),
    }
}
