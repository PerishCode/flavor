mod ast;
mod attribute;
mod expressions;
pub mod kind;
mod names;
mod parser;

pub use ast::SvelteMarkupAst;
pub use expressions::validate_expressions;
pub use parser::parse_markup;
