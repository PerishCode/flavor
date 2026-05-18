mod ast;
mod attribute;
mod cursor;
mod expressions;
mod kind;
mod names;
mod parser;

pub use ast::SvelteMarkupAst;
pub use expressions::validate_expressions;
pub use kind::SvelteMarkupKind;
pub use parser::parse_markup;
