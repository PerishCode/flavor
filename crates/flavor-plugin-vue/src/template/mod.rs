mod ast;
mod expressions;
pub mod kind;
mod names;
mod parser;

pub use ast::TemplateAst;
pub use expressions::validate_expressions;
pub use parser::parse_template;
