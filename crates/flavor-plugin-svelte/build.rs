use flavor_shared::grammar_build::{write_workspace_syntax_enum, SyntaxEnumOptions};

fn main() {
    write_workspace_syntax_enum(SyntaxEnumOptions {
        grammar_dir: "svelte",
        lexer: "SvelteMarkupLexer.g4",
        parser: "SvelteMarkupParser.g4",
        grammar_id: "svelte-markup",
        raw_kind_start: 2000,
        enum_name: "SvelteMarkupKind",
        raw_kind_path: "flavor_core::RawSyntaxKind",
        fallback_kind: "Error",
        output_file: "svelte_markup_kind.rs",
    });
}
