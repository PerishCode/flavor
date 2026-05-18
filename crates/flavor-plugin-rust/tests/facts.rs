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

#[test]
fn collects_repeated_token_patterns() {
    let source = repeated_handlers(10);
    let output = run(
        SourceText::new("repeated.rs", &source),
        RustPluginConfig::default(),
    );

    assert!(output
        .facts
        .repeated_token_patterns
        .iter()
        .any(|fact| fact.occurrences >= 10 && fact.total_lines >= 200));
}

fn repeated_handlers(count: usize) -> String {
    let mut source = String::new();
    for index in 0..count {
        source.push_str(&format!(
            "fn handle_{index}(input_{index}: i32) -> i32 {{\n\
                let local_{index} = input_{index};\n\
                match local_{index} {{\n\
                    0 => {{\n\
                        let next_{index}_a = local_{index};\n\
                        next_{index}_a\n\
                    }}\n\
                    1 => {{\n\
                        let next_{index}_b = local_{index};\n\
                        next_{index}_b\n\
                    }}\n\
                    2 => {{\n\
                        let next_{index}_c = local_{index};\n\
                        next_{index}_c\n\
                    }}\n\
                    _ => {{\n\
                        local_{index}\n\
                    }}\n\
                }}\n\
             }}\n\n"
        ));
    }
    source
}
