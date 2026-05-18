#![allow(dead_code)]

#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/config/mod.rs"]
mod config;
#[path = "../src/model.rs"]
mod model;
#[path = "../src/naming/mod.rs"]
mod naming;
#[path = "../src/output.rs"]
mod output;
#[path = "../src/path_match.rs"]
mod path_match;
#[path = "../src/plugins/mod.rs"]
mod plugins;
#[path = "../src/rules.rs"]
mod rules;
#[path = "../src/scan/mod.rs"]
mod scan;

#[path = "unit/cli.rs"]
mod cli_cases;
#[path = "unit/config.rs"]
mod config_cases;
#[path = "unit/model.rs"]
mod model_cases;
#[path = "unit/naming.rs"]
mod naming_cases;
#[path = "unit/output.rs"]
mod output_cases;
#[path = "unit/path_match.rs"]
mod path_cases;
#[path = "unit/plugins.rs"]
mod plugin_cases;
#[path = "unit/renderer_boundary.rs"]
mod renderer_boundary_cases;
#[path = "unit/scan.rs"]
mod scan_cases;
#[path = "unit/shape.rs"]
mod shape_cases;
