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

fn ts_output(source: &str) -> TsAnalysisOutput {
    run(
        SourceText::new("sample.ts", source),
        TsPluginConfig::default(),
    )
}

fn tsx_output(source: &str) -> TsAnalysisOutput {
    run(
        SourceText::new("sample.tsx", source),
        TsPluginConfig {
            source_mode: SourceMode::Tsx,
            ..Default::default()
        },
    )
}

mod source_file {
    use super::*;

    #[test]
    fn cst() {
        let output = ts_output("const value = 1;");

        assert_eq!(output.syntax.text().to_string(), "const value = 1;");
        assert!(has_node(&output, kind::VARIABLE_STATEMENT));
        assert!(has_node(&output, kind::VARIABLE_DECLARATION));
    }

    #[test]
    fn leading_trivia() {
        let output = ts_output("// leading\nconst value = 1;");

        let text = output.syntax.text().to_string();
        assert_eq!(text, "// leading\nconst value = 1;");
    }
}

mod tsx {
    use super::*;

    #[test]
    fn cst() {
        let output = tsx_output("const node = <div />;");

        assert!(output.syntax.text().to_string().contains("<div />"));
        assert!(has_node(&output, kind::INITIALIZER));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn element_tree() {
        let output = tsx_output("const node = <Panel title={name}><span>ok</span></Panel>;");

        assert!(has_node(&output, kind::JSX_ELEMENT));
        assert!(has_node(&output, kind::JSX_OPENING_ELEMENT));
        assert!(has_node(&output, kind::JSX_CLOSING_ELEMENT));
        assert!(has_node(&output, kind::JSX_ATTRIBUTE));
        assert!(has_node(&output, kind::JSX_EXPRESSION));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn edge_tree() {
        let output = tsx_output(
            r#"const node = (<UI.Panel data-id="card" svg:path="p" {...props}>hello <span>{title}</span></UI.Panel>);"#,
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
    fn comparison_stays_expression() {
        let output = tsx_output(
            "function inspectionLabelsOverlap(a: InspectionLabelBox, b: InspectionLabelBox) {
  return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
}",
        );

        assert!(has_node(&output, kind::BINARY_EXPRESSION));
        assert!(has_node(&output, kind::MEMBER_EXPRESSION));
        assert!(!has_node(&output, kind::JSX_ELEMENT));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn satisfies_generics_stay_type() {
        let output = tsx_output(
            r#"type InspectDomain = "session" | "message";
type InspectDomainPanel = (props: InspectDomainPanelProps) => ReactNode;
const inspectDomainRegistry = {
  message: MessageInspectDomain,
  session: SessionInspectDomain,
} satisfies Record<InspectDomain, InspectDomainPanel>;"#,
        );

        assert!(has_node(&output, kind::BINARY_EXPRESSION));
        assert!(has_node(&output, kind::TYPE_REFERENCE));
        assert!(!has_node(&output, kind::JSX_ELEMENT));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }
}

mod declarations {
    use super::*;

    #[test]
    fn function_signature() {
        let output = ts_output("function greet(name: string): string { return name; }");

        assert!(has_node(&output, kind::FUNCTION_DECLARATION));
        assert!(has_node(&output, kind::PARAMETER_LIST));
        assert!(has_node(&output, kind::RETURN_TYPE));
        assert!(has_node(&output, kind::BLOCK));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn decorated_class() {
        let output = ts_output("@sealed() class Example<T> extends Base { value!: string; }");

        assert!(has_node(&output, kind::CLASS_DECLARATION));
        assert!(has_node(&output, kind::DECORATOR_LIST));
        assert!(has_node(&output, kind::TYPE_PARAMETERS));
        assert!(has_node(&output, kind::HERITAGE_CLAUSE));
        assert!(has_node(&output, kind::CLASS_BODY));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn missing_class_body() {
        let output = ts_output("class Broken");

        assert!(output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == "expected class body"));
    }

    #[test]
    fn enum_and_namespace() {
        let output = ts_output(
            "export enum State { Idle = 'idle', Ready = 'ready' } declare module \"virtual:api\" { export const ready: boolean; } namespace App.Core { export type Id = string; }",
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
    fn interface_shape() {
        let output = ts_output(
            "interface User<T> extends Entity { id: string; save(next: Result<T>): void; }",
        );

        assert!(has_node(&output, kind::INTERFACE_DECLARATION));
        assert!(has_node(&output, kind::INTERFACE_BODY));
        assert!(has_node(&output, kind::PROPERTY_SIGNATURE));
        assert!(has_node(&output, kind::METHOD_SIGNATURE));
        assert!(has_node(&output, kind::HERITAGE_CLAUSE));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn type_alias() {
        let output = ts_output("export type UserMap<T> = Record<string, T> | null;");

        assert!(has_node(&output, kind::EXPORT_DECLARATION));
        assert!(has_node(&output, kind::TYPE_ALIAS_DECLARATION));
        assert!(has_node(&output, kind::UNION_TYPE));
        assert!(has_node(&output, kind::TYPE_REFERENCE));
        assert!(output.diagnostics.is_empty());
    }
}

mod modules {
    use super::*;

    #[test]
    fn import_export_surface() {
        let output = ts_output(
            "import value, * as api from './x'; import { default as base, type Ref as ViewRef } from './types'; import legacy = require('./legacy'); export { value as out }; export type * as models from './models'; export = legacy; export as namespace Legacy;",
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
    fn contextual_names() {
        let output = ts_output(
            r#"const type = stringField(value, "type"); const namespace = stringField(value, "namespace"); type Source = { type: "workspace" } | { namespace?: string; module?: string; type: "bundle" };"#,
        );

        assert!(has_node(&output, kind::VARIABLE_DECLARATION));
        assert!(has_node(&output, kind::UNION_TYPE));
        assert!(has_node(&output, kind::OBJECT_TYPE));
        assert!(has_node(&output, kind::TYPE_MEMBER));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }
}

mod control_flow {
    use super::*;

    #[test]
    fn statements() {
        let output = ts_output(
            "function resolve(items: string[]): string { for (const item of items) { if (item === 'skip') continue; switch (item) { case 'done': return item; default: break; } } try { throw new Error('missing'); } catch ({ message }) { return message; } finally { cleanup(); } }",
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
}

mod expressions {
    use super::*;

    #[test]
    fn shape() {
        let output = ts_output(
            "const value = makeUser(); const name = profile?.name; const ns = agentsRuntime?.namespace ?? \"unknown\"; const label = profile.name.trim(); const sum = left + right; const meta = { active: true }; const tags = [\"a\"];",
        );

        assert!(has_node(&output, kind::CALL_EXPRESSION));
        assert!(has_node(&output, kind::MEMBER_EXPRESSION));
        assert!(has_node(&output, kind::BINARY_EXPRESSION));
        assert!(has_node(&output, kind::OBJECT_EXPRESSION));
        assert!(has_node(&output, kind::ARRAY_EXPRESSION));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn literal_tokens() {
        let source = "const count = 123n; const pattern = /foo\\/[a-z]+/gi; const ratio = left / right; const label = `user:${profile.name}`;";
        let output = ts_output(source);

        assert_eq!(output.syntax.text().to_string(), source);
        assert!(has_token(&output, kind::BIG_INT_LITERAL));
        assert!(has_token(&output, kind::REGEX_LITERAL));
        assert!(has_token(&output, kind::TEMPLATE_LITERAL));
        assert!(has_token(&output, kind::SLASH));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn keywords() {
        let source = "const ok = value instanceof View && value satisfies View; const kind = typeof value; const empty = void value; const removed = delete target.key; const self = this.value;";
        let output = ts_output(source);

        assert!(has_node(&output, kind::BINARY_EXPRESSION));
        assert!(has_node(&output, kind::UNARY_EXPRESSION));
        assert!(has_node(&output, kind::MEMBER_EXPRESSION));
        assert!(has_token(&output, kind::KEYWORD_SATISFIES));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn optional_chains() {
        let source = "const value = loader?.(); const item = records?.[key]; const nested = records?.[key]?.render();";
        let output = ts_output(source);

        assert!(has_node(&output, kind::CALL_EXPRESSION));
        assert!(has_node(&output, kind::MEMBER_EXPRESSION));
        assert!(has_node(&output, kind::ELEMENT_ACCESS_EXPRESSION));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn rich_shapes() {
        let output = ts_output(
            "const mapper = (item: Item) => item.name; const label = ok ? 'yes' : 'no'; const created = new UserStore<Item>(); const ok = !disabled; const loaded = await load<Item>();",
        );

        assert!(has_node(&output, kind::ARROW_FUNCTION));
        assert!(has_node(&output, kind::CONDITIONAL_EXPRESSION));
        assert!(has_node(&output, kind::NEW_EXPRESSION));
        assert!(has_node(&output, kind::UNARY_EXPRESSION));
        assert!(has_node(&output, kind::AWAIT_EXPRESSION));
        assert!(has_node(&output, kind::TYPE_PARAMETERS));
        assert!(output.diagnostics.is_empty());
    }
}

mod types {
    use super::*;

    #[test]
    fn shape() {
        let output = ts_output(
            "function map<T>(value: string | number, next: Array<string>): Result<T> { return value; } const names: string[] = []; type Shape<T> = { readonly id?: string; save(next: T): Result<T>; [key: string]: unknown; };",
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
    fn operators() {
        let output = ts_output(
            "type Keys<T> = keyof T; type Value<T, K extends keyof T> = T[K]; type Box<T> = { readonly [K in keyof T]?: T[K]; }; type Result<T> = T extends infer U ? U : T; type Brand = unique symbol;",
        );

        assert!(has_node(&output, kind::TYPE_OPERATOR));
        assert!(has_node(&output, kind::INDEXED_ACCESS_TYPE));
        assert!(has_node(&output, kind::MAPPED_TYPE));
        assert!(has_node(&output, kind::CONDITIONAL_TYPE));
        assert!(has_node(&output, kind::TYPE_MEMBER));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }
}

mod class_members {
    use super::*;

    #[test]
    fn property_and_method() {
        let output =
            ts_output("class User { @trace id!: string; save(next: User): void { return next; } }");

        assert!(has_node(&output, kind::CLASS_MEMBER));
        assert!(has_node(&output, kind::PROPERTY_DECLARATION));
        assert!(has_node(&output, kind::METHOD_DECLARATION));
        assert!(has_node(&output, kind::DECORATOR_LIST));
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn special_methods() {
        let output = ts_output("class Store { delete(id: string): void { this.records.delete(id); } \"delete\"(id: string): void { this.records.delete(id); } }");

        assert!(has_node(&output, kind::METHOD_DECLARATION));
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    }

    #[test]
    fn recovery() {
        let output = ts_output("class Store { ?(): void {} ok(): void {} }");

        assert!(has_node(&output, kind::CLASS_DECLARATION));
        assert!(!output.diagnostics.is_empty());
    }

    #[test]
    fn modifiers() {
        let output = ts_output(
            "export default abstract class User { private readonly id!: string; static create(): User { return new User(); } } declare const version: string;",
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
    fn accessors() {
        let output =
            ts_output("class User { get name(): string { return value; } set name(value: string) { this.value = value; } }");

        assert!(has_node(&output, kind::METHOD_DECLARATION));
        assert!(output.diagnostics.is_empty());
    }
}

mod bindings {
    use super::*;

    #[test]
    fn patterns() {
        let output = ts_output(
            "const { id, profile: { name = 'anon' }, ...rest } = user; const [first, , second = 'b', ...tail] = items; function render({ title }: Props, [head]: string[]) { return title; }",
        );

        assert!(has_node(&output, kind::OBJECT_BINDING_PATTERN));
        assert!(has_node(&output, kind::ARRAY_BINDING_PATTERN));
        assert!(has_node(&output, kind::BINDING_ELEMENT));
        assert!(has_node(&output, kind::REST_ELEMENT));
        assert!(has_node(&output, kind::INITIALIZER));
        assert!(has_node(&output, kind::PARAMETER));
        assert!(output.diagnostics.is_empty());
    }
}
