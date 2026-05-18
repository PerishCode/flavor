use flavor_core::SourceText;
use flavor_plugin_rust::{run, RustNameKind, RustPluginConfig};

#[test]
fn collects_names_and_arms() {
    let output = run(
        SourceText::new(
            "sample.rs",
            r#"
fn route_value_name(input_value: i32) {
    let controller_runtime_result_value = input_value;
    match input_value {
        1 => {
            controller_runtime_result_value
        }
        _ => 0,
    };
}
"#,
        ),
        RustPluginConfig::default(),
    );

    assert!(output
        .facts
        .names
        .iter()
        .any(|name| name.kind == RustNameKind::Function && name.name == "route_value_name"));
    assert!(output
        .facts
        .names
        .iter()
        .any(|name| name.kind == RustNameKind::Binding
            && name.name == "controller_runtime_result_value"));
    assert!(output
        .facts
        .names
        .iter()
        .any(|name| name.kind == RustNameKind::Parameter && name.name == "input_value"));
    assert_eq!(output.facts.match_arms.len(), 2);
}

#[test]
fn skips_trait_impl_names() {
    let output = run(
        SourceText::new(
            "sample.rs",
            r#"
trait Repo {
    fn find_primary_by_account_id(&self);
}

impl Repo for SeaOrmRepo {
    fn find_primary_by_account_id(&self) {}
}

impl SeaOrmRepo {
    fn find_secondary_by_account_id(&self) {}
}
"#,
        ),
        RustPluginConfig::default(),
    );

    assert_eq!(
        output
            .facts
            .names
            .iter()
            .filter(|name| name.name == "find_primary_by_account_id")
            .count(),
        1
    );
    assert!(output
        .facts
        .names
        .iter()
        .any(|name| name.name == "find_secondary_by_account_id"));
}

#[test]
fn collects_test_attributes() {
    let output = run(
        SourceText::new(
            "sample.rs",
            "#[cfg(test)] mod tests { #[test] fn sample() {} }",
        ),
        RustPluginConfig::default(),
    );

    assert_eq!(output.facts.test_attributes.len(), 2);
}
