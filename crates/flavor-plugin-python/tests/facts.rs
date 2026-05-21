use flavor_core::SourceText;
use flavor_plugin_python::{run, PythonNameKind};

#[test]
fn collects_python_shape_facts() {
    let output = run(SourceText::new(
        "sample.py",
        r#"class Runner:
    def run(self, first_value, second_value=None):
        if first_value:
            return second_value
        return None

def helper(input_value):
    return input_value

result_value = helper(1)
"#,
    ));

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert!(output
        .facts
        .names
        .iter()
        .any(|fact| fact.kind == PythonNameKind::Method && fact.name == "run"));
    assert!(output
        .facts
        .names
        .iter()
        .any(|fact| fact.kind == PythonNameKind::Function && fact.name == "helper"));
    assert!(output
        .facts
        .names
        .iter()
        .any(|fact| fact.kind == PythonNameKind::Parameter && fact.name == "first_value"));
    assert!(output
        .facts
        .names
        .iter()
        .any(|fact| fact.kind == PythonNameKind::Binding && fact.name == "result_value"));

    let method = output
        .facts
        .function_bodies
        .iter()
        .find(|fact| fact.name == "run")
        .unwrap();
    assert_eq!(method.lines, 4);

    let branch = output.facts.dispatch_branches.first().unwrap();
    assert_eq!(branch.lines, 2);
}

#[test]
fn reports_missing_indented_body() {
    let output = run(SourceText::new("broken.py", "def empty():\nvalue = 1\n"));

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("no indented body")));
}
