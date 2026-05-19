#![allow(dead_code)]

use crate::ast::TsSourceFile;

pub trait TsVisitor {
    fn visit_source_file(&mut self, _source_file: &TsSourceFile) {}
}
