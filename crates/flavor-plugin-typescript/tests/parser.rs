use flavor_core::SourceText;
use flavor_plugin_typescript::{run, SourceMode, TsAnalysisOutput, TsPluginConfig};

#[path = "../src/internal/grammar.rs"]
mod kind;

use kind::Kind;

fn has_node(output: &TsAnalysisOutput, kind: Kind) -> bool {
    output
        .syntax
        .descendants()
        .any(|node| node.kind() == kind::schema().raw_kind(kind))
}

fn has_token(output: &TsAnalysisOutput, kind: Kind) -> bool {
    output.tokens.iter().any(|token| token.kind == kind)
}

#[test]
fn builds_source_file_cst() {
    let output = run(
        SourceText::new("sample.ts", "const value = 1;"),
        TsPluginConfig::default(),
    );

    assert_eq!(output.syntax.text().to_string(), "const value = 1;");
    assert!(has_node(&output, kind::VARIABLE_STATEMENT));
    assert!(has_node(&output, kind::VARIABLE_DECLARATION));
}

#[test]
fn keeps_trivia_in_cst() {
    let output = run(
        SourceText::new("sample.ts", "// leading\nconst value = 1;"),
        TsPluginConfig::default(),
    );

    let text = output.syntax.text().to_string();
    assert_eq!(text, "// leading\nconst value = 1;");
}

#[test]
fn parses_tsx_cst() {
    let config = TsPluginConfig {
        source_mode: SourceMode::Tsx,
        ..Default::default()
    };
    let output = run(
        SourceText::new("sample.tsx", "const node = <div />;"),
        config,
    );

    assert!(output.syntax.text().to_string().contains("<div />"));
    assert!(has_node(&output, kind::INITIALIZER));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_tsx_element_nodes() {
    let config = TsPluginConfig {
        source_mode: SourceMode::Tsx,
        ..Default::default()
    };
    let output = run(
        SourceText::new(
            "sample.tsx",
            "const node = <Panel title={name}><span>ok</span></Panel>;",
        ),
        config,
    );

    assert!(has_node(&output, kind::JSX_ELEMENT));
    assert!(has_node(&output, kind::JSX_OPENING_ELEMENT));
    assert!(has_node(&output, kind::JSX_CLOSING_ELEMENT));
    assert!(has_node(&output, kind::JSX_ATTRIBUTE));
    assert!(has_node(&output, kind::JSX_EXPRESSION));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_tsx_edge_nodes() {
    let config = TsPluginConfig {
        source_mode: SourceMode::Tsx,
        ..Default::default()
    };
    let output = run(
        SourceText::new(
            "sample.tsx",
            r#"const node = (<UI.Panel data-id="card" svg:path="p" {...props}>hello <span>{title}</span></UI.Panel>);"#,
        ),
        config,
    );

    assert!(has_node(&output, kind::JSX_ELEMENT));
    assert!(has_node(&output, kind::PARENTHESIZED_EXPRESSION));
    assert!(has_node(&output, kind::JSX_ATTRIBUTE));
    assert!(has_node(&output, kind::JSX_SPREAD_ATTRIBUTE));
    assert!(has_node(&output, kind::JSX_TEXT));
    assert!(has_node(&output, kind::JSX_EXPRESSION));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn keeps_ts_comparison() {
    let output = run(
        SourceText::new("sample.ts", "const ok = left < right;"),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::BINARY_EXPRESSION));
    assert!(!has_node(&output, kind::JSX_ELEMENT));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_function_declaration_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "function greet(name: string): string { return name; }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::FUNCTION_DECLARATION));
    assert!(has_node(&output, kind::PARAMETER_LIST));
    assert!(has_node(&output, kind::RETURN_TYPE));
    assert!(has_node(&output, kind::BLOCK));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_binding_patterns() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "const { id, profile: { name = 'anon' }, ...rest } = user; const [first, , second = 'b', ...tail] = items; function render({ title }: Props, [head]: string[]) { return title; }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::OBJECT_BINDING_PATTERN));
    assert!(has_node(&output, kind::ARRAY_BINDING_PATTERN));
    assert!(has_node(&output, kind::BINDING_ELEMENT));
    assert!(has_node(&output, kind::REST_ELEMENT));
    assert!(has_node(&output, kind::INITIALIZER));
    assert!(has_node(&output, kind::PARAMETER));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_contextual_names() {
    let output = run(
        SourceText::new(
            "sample.ts",
            r#"const type = stringField(value, "type"); const namespace = stringField(value, "namespace"); type Source = { type: "workspace" } | { namespace?: string; module?: string; type: "bundle" };"#,
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::VARIABLE_DECLARATION));
    assert!(has_node(&output, kind::UNION_TYPE));
    assert!(has_node(&output, kind::OBJECT_TYPE));
    assert!(has_node(&output, kind::TYPE_MEMBER));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_decorated_class_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "@sealed() class Example<T> extends Base { value!: string; }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::CLASS_DECLARATION));
    assert!(has_node(&output, kind::DECORATOR_LIST));
    assert!(has_node(&output, kind::TYPE_PARAMETERS));
    assert!(has_node(&output, kind::HERITAGE_CLAUSE));
    assert!(has_node(&output, kind::CLASS_BODY));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn reports_missing_class_body() {
    let output = run(
        SourceText::new("sample.ts", "class Broken"),
        TsPluginConfig::default(),
    );

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message == "expected class body"));
}

#[test]
fn parses_module_declaration_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "import value, * as api from './x'; import { default as base, type Ref as ViewRef } from './types'; import legacy = require('./legacy'); export { value as out }; export type * as models from './models'; export = legacy; export as namespace Legacy;",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::IMPORT_DECLARATION));
    assert!(has_node(&output, kind::IMPORT_CLAUSE));
    assert!(has_node(&output, kind::IMPORT_EQUALS_DECLARATION));
    assert!(has_node(&output, kind::EXTERNAL_MODULE_REFERENCE));
    assert!(has_node(&output, kind::NAMESPACE_IMPORT));
    assert!(has_node(&output, kind::NAMED_IMPORTS));
    assert!(has_node(&output, kind::IMPORT_SPECIFIER));
    assert!(has_node(&output, kind::EXPORT_DECLARATION));
    assert!(has_node(&output, kind::EXPORT_CLAUSE));
    assert!(has_node(&output, kind::NAMED_EXPORTS));
    assert!(has_node(&output, kind::EXPORT_SPECIFIER));
    assert!(has_node(&output, kind::EXPORT_ASSIGNMENT));
    assert!(has_node(&output, kind::NAMESPACE_EXPORT_DECLARATION));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_enum_namespace_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "export enum State { Idle = 'idle', Ready = 'ready' } declare module \"virtual:api\" { export const ready: boolean; } namespace App.Core { export type Id = string; }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::ENUM_DECLARATION));
    assert!(has_node(&output, kind::ENUM_BODY));
    assert!(has_node(&output, kind::ENUM_MEMBER));
    assert!(has_node(&output, kind::NAMESPACE_DECLARATION));
    assert!(has_node(&output, kind::NAMESPACE_BODY));
    assert!(has_node(&output, kind::VARIABLE_STATEMENT));
    assert!(has_node(&output, kind::TYPE_ALIAS_DECLARATION));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_control_flow_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "function resolve(items: string[]): string { for (const item of items) { if (item === 'skip') continue; switch (item) { case 'done': return item; default: break; } } try { throw new Error('missing'); } catch ({ message }) { return message; } finally { cleanup(); } }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::FOR_STATEMENT));
    assert!(has_node(&output, kind::IF_STATEMENT));
    assert!(has_node(&output, kind::SWITCH_STATEMENT));
    assert!(has_node(&output, kind::SWITCH_BODY));
    assert!(has_node(&output, kind::SWITCH_CASE));
    assert!(has_node(&output, kind::RETURN_STATEMENT));
    assert!(has_node(&output, kind::TRY_STATEMENT));
    assert!(has_node(&output, kind::CATCH_CLAUSE));
    assert!(has_node(&output, kind::CATCH_BINDING));
    assert!(has_node(&output, kind::OBJECT_BINDING_PATTERN));
    assert!(has_node(&output, kind::FINALLY_CLAUSE));
    assert!(has_node(&output, kind::THROW_STATEMENT));
    assert!(has_node(&output, kind::BREAK_STATEMENT));
    assert!(has_node(&output, kind::CONTINUE_STATEMENT));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_expression_shape_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "const value = makeUser(); const name = profile?.name; const ns = agentsRuntime?.namespace ?? \"unknown\"; const label = profile.name.trim(); const sum = left + right; const meta = { active: true }; const tags = [\"a\"];",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::CALL_EXPRESSION));
    assert!(has_node(&output, kind::MEMBER_EXPRESSION));
    assert!(has_node(&output, kind::BINARY_EXPRESSION));
    assert!(has_node(&output, kind::OBJECT_EXPRESSION));
    assert!(has_node(&output, kind::ARRAY_EXPRESSION));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_literal_tokens() {
    let source = "const count = 123n; const pattern = /foo\\/[a-z]+/gi; const ratio = left / right; const label = `user:${profile.name}`;";
    let output = run(
        SourceText::new("sample.ts", source),
        TsPluginConfig::default(),
    );

    assert_eq!(output.syntax.text().to_string(), source);
    assert!(has_token(&output, kind::BIG_INT_LITERAL));
    assert!(has_token(&output, kind::REGEX_LITERAL));
    assert!(has_token(&output, kind::TEMPLATE_LITERAL));
    assert!(has_token(&output, kind::SLASH));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_keyword_exprs() {
    let source = "const ok = value instanceof View && value satisfies View; const kind = typeof value; const empty = void value; const removed = delete target.key; const self = this.value;";
    let output = run(
        SourceText::new("sample.ts", source),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::BINARY_EXPRESSION));
    assert!(has_node(&output, kind::UNARY_EXPRESSION));
    assert!(has_node(&output, kind::MEMBER_EXPRESSION));
    assert!(has_token(&output, kind::KEYWORD_SATISFIES));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_optional_chains() {
    let source = "const value = loader?.(); const item = records?.[key]; const nested = records?.[key]?.render();";
    let output = run(
        SourceText::new("sample.ts", source),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::CALL_EXPRESSION));
    assert!(has_node(&output, kind::MEMBER_EXPRESSION));
    assert!(has_node(&output, kind::ELEMENT_ACCESS_EXPRESSION));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_rich_expr_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "const mapper = (item: Item) => item.name; const label = ok ? 'yes' : 'no'; const created = new UserStore<Item>(); const ok = !disabled; const loaded = await load<Item>();",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::ARROW_FUNCTION));
    assert!(has_node(&output, kind::CONDITIONAL_EXPRESSION));
    assert!(has_node(&output, kind::NEW_EXPRESSION));
    assert!(has_node(&output, kind::UNARY_EXPRESSION));
    assert!(has_node(&output, kind::AWAIT_EXPRESSION));
    assert!(has_node(&output, kind::TYPE_PARAMETERS));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_type_shape_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "function map<T>(value: string | number, next: Array<string>): Result<T> { return value; } const names: string[] = []; type Shape<T> = { readonly id?: string; save(next: T): Result<T>; [key: string]: unknown; };",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::UNION_TYPE));
    assert!(has_node(&output, kind::TYPE_REFERENCE));
    assert!(has_node(&output, kind::ARRAY_TYPE));
    assert!(has_node(&output, kind::OBJECT_TYPE));
    assert!(has_node(&output, kind::TYPE_MEMBER));
    assert!(has_node(&output, kind::PARAMETER_LIST));
    assert!(has_node(&output, kind::RETURN_TYPE));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_type_operator_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "type Keys<T> = keyof T; type Value<T, K extends keyof T> = T[K]; type Box<T> = { readonly [K in keyof T]?: T[K]; }; type Result<T> = T extends infer U ? U : T; type Brand = unique symbol;",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::TYPE_OPERATOR));
    assert!(has_node(&output, kind::INDEXED_ACCESS_TYPE));
    assert!(has_node(&output, kind::MAPPED_TYPE));
    assert!(has_node(&output, kind::CONDITIONAL_TYPE));
    assert!(has_node(&output, kind::TYPE_MEMBER));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_interface_declaration_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "interface User<T> extends Entity { id: string; save(next: Result<T>): void; }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::INTERFACE_DECLARATION));
    assert!(has_node(&output, kind::INTERFACE_BODY));
    assert!(has_node(&output, kind::PROPERTY_SIGNATURE));
    assert!(has_node(&output, kind::METHOD_SIGNATURE));
    assert!(has_node(&output, kind::HERITAGE_CLAUSE));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_type_alias_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "export type UserMap<T> = Record<string, T> | null;",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::EXPORT_DECLARATION));
    assert!(has_node(&output, kind::TYPE_ALIAS_DECLARATION));
    assert!(has_node(&output, kind::UNION_TYPE));
    assert!(has_node(&output, kind::TYPE_REFERENCE));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_class_member_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "class User { @trace id!: string; save(next: User): void { return next; } }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::CLASS_MEMBER));
    assert!(has_node(&output, kind::PROPERTY_DECLARATION));
    assert!(has_node(&output, kind::METHOD_DECLARATION));
    assert!(has_node(&output, kind::DECORATOR_LIST));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_modifier_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "export default abstract class User { private readonly id!: string; static create(): User { return new User(); } } declare const version: string;",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::EXPORT_DECLARATION));
    assert!(has_node(&output, kind::MODIFIER_LIST));
    assert!(has_node(&output, kind::CLASS_DECLARATION));
    assert!(has_node(&output, kind::PROPERTY_DECLARATION));
    assert!(has_node(&output, kind::METHOD_DECLARATION));
    assert!(has_node(&output, kind::VARIABLE_STATEMENT));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_accessor_members() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "class User { get name(): string { return value; } set name(value: string) { this.value = value; } }",
        ),
        TsPluginConfig::default(),
    );

    assert!(has_node(&output, kind::METHOD_DECLARATION));
    assert!(output.diagnostics.is_empty());
}
