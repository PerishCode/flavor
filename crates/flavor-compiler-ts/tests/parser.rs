use flavor_compiler_core::{RawSyntaxKind, SourceText};
use flavor_compiler_ts::{run, syntax_kind::TsSyntaxKind, SourceMode, TsCompilerConfig};

fn has_node(output: &flavor_compiler_ts::TsCompileOutput, kind: TsSyntaxKind) -> bool {
    output
        .source_file
        .syntax()
        .descendants()
        .any(|node| node.kind() == RawSyntaxKind::from(kind))
}

fn has_token(output: &flavor_compiler_ts::TsCompileOutput, kind: TsSyntaxKind) -> bool {
    output
        .source_file
        .tokens()
        .iter()
        .any(|token| token.kind == kind)
}

#[test]
fn builds_source_file_cst() {
    let output = run(
        SourceText::new("sample.ts", "const value = 1;"),
        TsCompilerConfig::default(),
    );

    assert_eq!(
        output.source_file.syntax().text().to_string(),
        "const value = 1;"
    );
    assert!(has_node(&output, TsSyntaxKind::VariableStatement));
    assert!(has_node(&output, TsSyntaxKind::VariableDeclaration));
}

#[test]
fn keeps_trivia_in_cst() {
    let output = run(
        SourceText::new("sample.ts", "// leading\nconst value = 1;"),
        TsCompilerConfig::default(),
    );

    let text = output.source_file.syntax().text().to_string();
    assert_eq!(text, "// leading\nconst value = 1;");
}

#[test]
fn parses_tsx_cst() {
    let config = TsCompilerConfig {
        source_mode: SourceMode::Tsx,
        ..Default::default()
    };
    let output = run(
        SourceText::new("sample.tsx", "const node = <div />;"),
        config,
    );

    assert!(output
        .source_file
        .syntax()
        .text()
        .to_string()
        .contains("<div />"));
    assert!(has_node(&output, TsSyntaxKind::Initializer));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_tsx_element_nodes() {
    let config = TsCompilerConfig {
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

    assert!(has_node(&output, TsSyntaxKind::JsxElement));
    assert!(has_node(&output, TsSyntaxKind::JsxOpeningElement));
    assert!(has_node(&output, TsSyntaxKind::JsxClosingElement));
    assert!(has_node(&output, TsSyntaxKind::JsxAttribute));
    assert!(has_node(&output, TsSyntaxKind::JsxExpression));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_tsx_edge_nodes() {
    let config = TsCompilerConfig {
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

    assert!(has_node(&output, TsSyntaxKind::JsxElement));
    assert!(has_node(&output, TsSyntaxKind::ParenthesizedExpression));
    assert!(has_node(&output, TsSyntaxKind::JsxAttribute));
    assert!(has_node(&output, TsSyntaxKind::JsxSpreadAttribute));
    assert!(has_node(&output, TsSyntaxKind::JsxText));
    assert!(has_node(&output, TsSyntaxKind::JsxExpression));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn keeps_ts_comparison() {
    let output = run(
        SourceText::new("sample.ts", "const ok = left < right;"),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::BinaryExpression));
    assert!(!has_node(&output, TsSyntaxKind::JsxElement));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_function_declaration_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "function greet(name: string): string { return name; }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::FunctionDeclaration));
    assert!(has_node(&output, TsSyntaxKind::ParameterList));
    assert!(has_node(&output, TsSyntaxKind::ReturnType));
    assert!(has_node(&output, TsSyntaxKind::Block));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_binding_patterns() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "const { id, profile: { name = 'anon' }, ...rest } = user; const [first, , second = 'b', ...tail] = items; function render({ title }: Props, [head]: string[]) { return title; }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ObjectBindingPattern));
    assert!(has_node(&output, TsSyntaxKind::ArrayBindingPattern));
    assert!(has_node(&output, TsSyntaxKind::BindingElement));
    assert!(has_node(&output, TsSyntaxKind::RestElement));
    assert!(has_node(&output, TsSyntaxKind::Initializer));
    assert!(has_node(&output, TsSyntaxKind::Parameter));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_decorated_class_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "@sealed() class Example<T> extends Base { value!: string; }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ClassDeclaration));
    assert!(has_node(&output, TsSyntaxKind::DecoratorList));
    assert!(has_node(&output, TsSyntaxKind::TypeParameters));
    assert!(has_node(&output, TsSyntaxKind::HeritageClause));
    assert!(has_node(&output, TsSyntaxKind::ClassBody));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn reports_missing_class_body() {
    let output = run(
        SourceText::new("sample.ts", "class Broken"),
        TsCompilerConfig::default(),
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
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ImportDeclaration));
    assert!(has_node(&output, TsSyntaxKind::ImportClause));
    assert!(has_node(&output, TsSyntaxKind::ImportEqualsDeclaration));
    assert!(has_node(&output, TsSyntaxKind::ExternalModuleReference));
    assert!(has_node(&output, TsSyntaxKind::NamespaceImport));
    assert!(has_node(&output, TsSyntaxKind::NamedImports));
    assert!(has_node(&output, TsSyntaxKind::ImportSpecifier));
    assert!(has_node(&output, TsSyntaxKind::ExportDeclaration));
    assert!(has_node(&output, TsSyntaxKind::ExportClause));
    assert!(has_node(&output, TsSyntaxKind::NamedExports));
    assert!(has_node(&output, TsSyntaxKind::ExportSpecifier));
    assert!(has_node(&output, TsSyntaxKind::ExportAssignment));
    assert!(has_node(&output, TsSyntaxKind::NamespaceExportDeclaration));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_enum_namespace_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "export enum State { Idle = 'idle', Ready = 'ready' } declare module \"virtual:api\" { export const ready: boolean; } namespace App.Core { export type Id = string; }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::EnumDeclaration));
    assert!(has_node(&output, TsSyntaxKind::EnumBody));
    assert!(has_node(&output, TsSyntaxKind::EnumMember));
    assert!(has_node(&output, TsSyntaxKind::NamespaceDeclaration));
    assert!(has_node(&output, TsSyntaxKind::NamespaceBody));
    assert!(has_node(&output, TsSyntaxKind::VariableStatement));
    assert!(has_node(&output, TsSyntaxKind::TypeAliasDeclaration));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_control_flow_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "function resolve(items: string[]): string { for (const item of items) { if (item === 'skip') continue; switch (item) { case 'done': return item; default: break; } } try { throw new Error('missing'); } catch ({ message }) { return message; } finally { cleanup(); } }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ForStatement));
    assert!(has_node(&output, TsSyntaxKind::IfStatement));
    assert!(has_node(&output, TsSyntaxKind::SwitchStatement));
    assert!(has_node(&output, TsSyntaxKind::SwitchBody));
    assert!(has_node(&output, TsSyntaxKind::SwitchCase));
    assert!(has_node(&output, TsSyntaxKind::ReturnStatement));
    assert!(has_node(&output, TsSyntaxKind::TryStatement));
    assert!(has_node(&output, TsSyntaxKind::CatchClause));
    assert!(has_node(&output, TsSyntaxKind::CatchBinding));
    assert!(has_node(&output, TsSyntaxKind::ObjectBindingPattern));
    assert!(has_node(&output, TsSyntaxKind::FinallyClause));
    assert!(has_node(&output, TsSyntaxKind::ThrowStatement));
    assert!(has_node(&output, TsSyntaxKind::BreakStatement));
    assert!(has_node(&output, TsSyntaxKind::ContinueStatement));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_expression_shape_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "const value = makeUser(); const name = profile?.name; const ns = agentsRuntime?.namespace ?? \"unknown\"; const label = profile.name.trim(); const sum = left + right; const meta = { active: true }; const tags = [\"a\"];",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::CallExpression));
    assert!(has_node(&output, TsSyntaxKind::MemberExpression));
    assert!(has_node(&output, TsSyntaxKind::BinaryExpression));
    assert!(has_node(&output, TsSyntaxKind::ObjectExpression));
    assert!(has_node(&output, TsSyntaxKind::ArrayExpression));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_literal_tokens() {
    let source = "const count = 123n; const pattern = /foo\\/[a-z]+/gi; const ratio = left / right; const label = `user:${profile.name}`;";
    let output = run(
        SourceText::new("sample.ts", source),
        TsCompilerConfig::default(),
    );

    assert_eq!(output.source_file.syntax().text().to_string(), source);
    assert!(has_token(&output, TsSyntaxKind::BigIntLiteral));
    assert!(has_token(&output, TsSyntaxKind::RegexLiteral));
    assert!(has_token(&output, TsSyntaxKind::TemplateLiteral));
    assert!(has_token(&output, TsSyntaxKind::Slash));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_keyword_exprs() {
    let source = "const ok = value instanceof View && value satisfies View; const kind = typeof value; const empty = void value; const removed = delete target.key; const self = this.value;";
    let output = run(
        SourceText::new("sample.ts", source),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::BinaryExpression));
    assert!(has_node(&output, TsSyntaxKind::UnaryExpression));
    assert!(has_node(&output, TsSyntaxKind::MemberExpression));
    assert!(has_token(&output, TsSyntaxKind::KeywordSatisfies));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_optional_chains() {
    let source = "const value = loader?.(); const item = records?.[key]; const nested = records?.[key]?.render();";
    let output = run(
        SourceText::new("sample.ts", source),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::CallExpression));
    assert!(has_node(&output, TsSyntaxKind::MemberExpression));
    assert!(has_node(&output, TsSyntaxKind::ElementAccessExpression));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_rich_expr_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "const mapper = (item: Item) => item.name; const label = ok ? 'yes' : 'no'; const created = new UserStore<Item>(); const ok = !disabled; const loaded = await load<Item>();",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ArrowFunction));
    assert!(has_node(&output, TsSyntaxKind::ConditionalExpression));
    assert!(has_node(&output, TsSyntaxKind::NewExpression));
    assert!(has_node(&output, TsSyntaxKind::UnaryExpression));
    assert!(has_node(&output, TsSyntaxKind::AwaitExpression));
    assert!(has_node(&output, TsSyntaxKind::TypeParameters));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_type_shape_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "function map<T>(value: string | number, next: Array<string>): Result<T> { return value; } const names: string[] = []; type Shape<T> = { readonly id?: string; save(next: T): Result<T>; [key: string]: unknown; };",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::UnionType));
    assert!(has_node(&output, TsSyntaxKind::TypeReference));
    assert!(has_node(&output, TsSyntaxKind::ArrayType));
    assert!(has_node(&output, TsSyntaxKind::ObjectType));
    assert!(has_node(&output, TsSyntaxKind::TypeMember));
    assert!(has_node(&output, TsSyntaxKind::ParameterList));
    assert!(has_node(&output, TsSyntaxKind::ReturnType));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_type_operator_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "type Keys<T> = keyof T; type Value<T, K extends keyof T> = T[K]; type Box<T> = { readonly [K in keyof T]?: T[K]; }; type Result<T> = T extends infer U ? U : T; type Brand = unique symbol;",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::TypeOperator));
    assert!(has_node(&output, TsSyntaxKind::IndexedAccessType));
    assert!(has_node(&output, TsSyntaxKind::MappedType));
    assert!(has_node(&output, TsSyntaxKind::ConditionalType));
    assert!(has_node(&output, TsSyntaxKind::TypeMember));
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
}

#[test]
fn parses_interface_declaration_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "interface User<T> extends Entity { id: string; save(next: Result<T>): void; }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::InterfaceDeclaration));
    assert!(has_node(&output, TsSyntaxKind::InterfaceBody));
    assert!(has_node(&output, TsSyntaxKind::PropertySignature));
    assert!(has_node(&output, TsSyntaxKind::MethodSignature));
    assert!(has_node(&output, TsSyntaxKind::HeritageClause));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_type_alias_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "export type UserMap<T> = Record<string, T> | null;",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ExportDeclaration));
    assert!(has_node(&output, TsSyntaxKind::TypeAliasDeclaration));
    assert!(has_node(&output, TsSyntaxKind::UnionType));
    assert!(has_node(&output, TsSyntaxKind::TypeReference));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_class_member_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "class User { @trace id!: string; save(next: User): void { return next; } }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ClassMember));
    assert!(has_node(&output, TsSyntaxKind::PropertyDeclaration));
    assert!(has_node(&output, TsSyntaxKind::MethodDeclaration));
    assert!(has_node(&output, TsSyntaxKind::DecoratorList));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_modifier_nodes() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "export default abstract class User { private readonly id!: string; static create(): User { return new User(); } } declare const version: string;",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::ExportDeclaration));
    assert!(has_node(&output, TsSyntaxKind::ModifierList));
    assert!(has_node(&output, TsSyntaxKind::ClassDeclaration));
    assert!(has_node(&output, TsSyntaxKind::PropertyDeclaration));
    assert!(has_node(&output, TsSyntaxKind::MethodDeclaration));
    assert!(has_node(&output, TsSyntaxKind::VariableStatement));
    assert!(output.diagnostics.is_empty());
}

#[test]
fn parses_accessor_members() {
    let output = run(
        SourceText::new(
            "sample.ts",
            "class User { get name(): string { return value; } set name(value: string) { this.value = value; } }",
        ),
        TsCompilerConfig::default(),
    );

    assert!(has_node(&output, TsSyntaxKind::MethodDeclaration));
    assert!(output.diagnostics.is_empty());
}
