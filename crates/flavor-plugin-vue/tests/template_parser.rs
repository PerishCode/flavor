use flavor_core::{RawSyntaxKind, SourceText};
use flavor_plugin_vue::{
    run,
    template::{parse_template, VueTemplateKind},
    VuePluginConfig,
};

fn has_node(ast: &flavor_plugin_vue::TemplateAst, kind: VueTemplateKind) -> bool {
    ast.syntax()
        .descendants()
        .any(|node| node.kind() == RawSyntaxKind::from(kind))
}

fn has_token(ast: &flavor_plugin_vue::TemplateAst, kind: VueTemplateKind) -> bool {
    ast.syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| token.kind() == RawSyntaxKind::from(kind))
}

fn is_core_trivia(kind: RawSyntaxKind) -> bool {
    matches!(kind.0, 1..=4)
}

fn assert_cst_matches_schema(ast: &flavor_plugin_vue::TemplateAst) {
    for node in ast.syntax().descendants() {
        assert!(
            VueTemplateKind::raw_is_node(node.kind()),
            "node kind {:?} is not declared as a G4 node",
            node.kind()
        );
    }
    for token in ast
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        assert!(
            VueTemplateKind::raw_is_token(token.kind()) || is_core_trivia(token.kind()),
            "token kind {:?} is not declared as a G4 token",
            token.kind()
        );
    }
}

#[test]
fn parses_template_text() {
    let ast = parse_template("<div>{{ message }}</div>");

    assert_eq!(ast.syntax().text().to_string(), "<div>{{ message }}</div>");
    assert!(has_node(&ast, VueTemplateKind::StartTag));
    assert!(has_node(&ast, VueTemplateKind::EndTag));
}

#[test]
fn cst_matches_schema() {
    let ast = parse_template(r#"<button v-if="ok" :class="klass">Save {{ label }}</button>"#);

    assert_cst_matches_schema(&ast);
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn run_builds_template_ast() {
    let output = run(
        SourceText::new(
            "Sample.vue",
            "<template><main>{{ value }}</main></template>",
        ),
        VuePluginConfig::default(),
    );

    assert_eq!(
        output
            .template
            .expect("template ast")
            .syntax()
            .text()
            .to_string(),
        "<main>{{ value }}</main>"
    );
}

#[test]
fn preserves_attributes_and_directives() {
    let source = r#"<button v-if="ok" :class="klass" @click.stop="save">Save</button>"#;
    let ast = parse_template(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert!(has_token(&ast, VueTemplateKind::AttributeValue));
    assert!(has_node(&ast, VueTemplateKind::DirectiveName));
    assert!(has_token(&ast, VueTemplateKind::DirectiveBase));
    assert!(has_token(&ast, VueTemplateKind::DirectiveArgument));
    assert!(has_token(&ast, VueTemplateKind::DirectiveModifier));
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == RawSyntaxKind::from(VueTemplateKind::DirectiveExpression))
            .count(),
        3
    );
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == RawSyntaxKind::from(VueTemplateKind::Directive))
            .count(),
        3
    );
}

#[test]
fn parses_dynamic_directives() {
    let source = r#"<slot v-bind:[user.name].camel="value" @[event.name].stop="save" #default />"#;
    let ast = parse_template(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert_eq!(
        ast.syntax()
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| token.kind() == RawSyntaxKind::from(VueTemplateKind::DirectiveArgument))
            .count(),
        3
    );
    assert_eq!(
        ast.syntax()
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| token.kind() == RawSyntaxKind::from(VueTemplateKind::DirectiveModifier))
            .count(),
        2
    );
}

#[test]
fn v_pre_keeps_raw() {
    let ast = parse_template("<div v-pre>{{ broken( }}</div>");

    assert_eq!(
        ast.syntax().text().to_string(),
        "<div v-pre>{{ broken( }}</div>"
    );
    assert!(!has_node(&ast, VueTemplateKind::Interpolation));
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn builds_nested_elements() {
    let source = "<main><section>{{ title }}</section><img></main>";
    let ast = parse_template(source);

    assert_eq!(ast.syntax().text().to_string(), source);
    assert_eq!(
        ast.syntax()
            .descendants()
            .filter(|node| node.kind() == RawSyntaxKind::from(VueTemplateKind::Element))
            .count(),
        3
    );
    assert!(ast.diagnostics().is_empty());
}

#[test]
fn reports_missing_nested_close() {
    let ast = parse_template("<div><span></div>");

    assert_eq!(ast.syntax().text().to_string(), "<div><span></div>");
    assert!(ast
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message == "missing closing </span> tag"));
}

#[test]
fn reports_unclosed_interpolation() {
    let ast = parse_template("<div>{{ value</div>");

    assert!(ast
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.message == "missing interpolation close delimiter"));
}
