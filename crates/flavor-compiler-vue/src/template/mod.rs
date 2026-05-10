mod ast;
mod expressions;
mod kind;
mod names;
mod parser;

pub use ast::TemplateAst;
pub use expressions::validate_expressions;
pub use kind::VueTemplateKind;
pub use parser::parse_template;
