mod ast;
mod expressions;
pub mod kind;

pub use ast::TemplateAst;
pub use expressions::validate_expressions;

pub(super) fn parse(source: &str) -> TemplateAst {
    let output = flavor_grammar::parse_vue_template(
        kind::bundle(),
        flavor_core::SourceText::new("vue-template", source),
    );
    TemplateAst::new(output.syntax, output.diagnostics)
}
