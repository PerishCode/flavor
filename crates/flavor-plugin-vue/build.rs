use flavor_shared::grammar_build::{write_workspace_syntax_enum, SyntaxEnumOptions};

fn main() {
    write_workspace_syntax_enum(SyntaxEnumOptions {
        grammar_dir: "vue",
        lexer: "VueTemplateLexer.g4",
        parser: "VueTemplateParser.g4",
        grammar_id: "vue-template",
        raw_kind_start: 1000,
        enum_name: "VueTemplateKind",
        raw_kind_path: "flavor_core::RawSyntaxKind",
        fallback_kind: "Error",
        output_file: "vue_template_kind.rs",
    });
}
