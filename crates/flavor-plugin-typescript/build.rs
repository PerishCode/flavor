use flavor_shared::grammar_build::{write_workspace_syntax_enum, SyntaxEnumOptions};

fn main() {
    write_workspace_syntax_enum(SyntaxEnumOptions {
        grammar_dir: "typescript",
        lexer: "TypeScriptLexer.g4",
        parser: "TypeScriptParser.g4",
        grammar_id: "typescript",
        raw_kind_start: 100,
        enum_name: "TsSyntaxKind",
        raw_kind_path: "flavor_core::RawSyntaxKind",
        fallback_kind: "Unknown",
        output_file: "ts_syntax_kind.rs",
    });
}
