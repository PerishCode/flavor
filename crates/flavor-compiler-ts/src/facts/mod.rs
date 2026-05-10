use crate::{ast::TsSourceFile, syntax_kind::TsSyntaxKind};

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct TsFacts {
    pub import_count: usize,
    pub export_count: usize,
}

pub fn collect(source_file: &TsSourceFile) -> TsFacts {
    let mut facts = TsFacts::default();
    for token in source_file.tokens() {
        match token.kind {
            TsSyntaxKind::KeywordImport => facts.import_count += 1,
            TsSyntaxKind::KeywordExport => facts.export_count += 1,
            _ => {}
        }
    }
    facts
}
