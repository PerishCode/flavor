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
pub(crate) struct RulesOptions {
    pub(crate) format: OutputFormat,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum CliCommand {
    Check(CliOptions),
    Rules(RulesOptions),
    Help,
    Version,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum CommandMode {
    Check,
    Rules,
}

pub(crate) fn parse_args(args: Vec<String>) -> Result<CliCommand, String> {
    let mut root = PathBuf::from(".");
    let mut config = None;
    let mut format = OutputFormat::Text;
    let mut strict_warnings = false;
    let mut command_seen = false;
    let mut mode = CommandMode::Check;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "check" if !command_seen => {
                command_seen = true;
                mode = CommandMode::Check;
            }
            "rules" if !command_seen => {
                command_seen = true;
                mode = CommandMode::Rules;
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

    Ok(match mode {
        CommandMode::Check => CliCommand::Check(CliOptions {
            root,
            config,
            format,
            strict_warnings,
        }),
        CommandMode::Rules => CliCommand::Rules(RulesOptions { format }),
    })
}

pub(crate) fn help_text() -> &'static str {
    r#"flavor

Check-only codestyle attributes for the personal flavor boundary.
It does not format, rewrite, run services, or manage runtime state.

Commands:
  check [--root <path>] [--config <path>] [--format text|json] [--strict-warnings]
  rules [--format text|json]
  help
  version

Config:
  --config <path>  Load this JSON config. The file's directory becomes the
                   project root for scan patterns.
  (no --config)    Walk ancestors of --root (default: cwd) looking for a
                   flavor.json. The nearest match wins; its directory
                   becomes the project root. Falls back to built-in
                   include/exclude patterns if none is found before the
                   filesystem root.

  Optional flavor.json field:
    allowEmptyScan  Suppress the "0 files matched" warning + exit 1.
                    Reserved for workspace-root configs that intentionally
                    exclude every submodule and delegate to per-submodule
                    flavor.json files via walk-up.

Scope:
  The check covers handwritten Rust, TypeScript, TSX, Vue, and Svelte source
  through scan.include / scan.exclude path patterns. Rule behavior is adjusted
  through named preferences and ordered overrides that match files or
  directories and set rule payload, severity, or enabled state. Preferences
  expand first; explicit overrides remain the final adjustment layer.

Reports:
  check reports include rule-level bad-flavor notes and action hints when
  issues exist. The hints are review pressure, not automatic fix instructions.

Exit codes:
  0  scan matched at least one file and produced no deny issues (and no
     warnings when --strict-warnings is set), or scan.include matched 0
     files and the active config set allowEmptyScan: true.
  1  deny issues, strict-warning failure, or scan.include matched 0 files
     without allowEmptyScan (which usually means the config or --root is
     wrong).

Project:
  Source:  https://github.com/PerishCode/flavor
  Issues:  https://github.com/PerishCode/flavor/issues
  Contributing: see AGENTS.md in the source tree for branch / commit / PR
                shape conventions, including how agents land changes.
"#
}

fn parse_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        other => Err(format!("unsupported output format: {other}")),
    }
}
