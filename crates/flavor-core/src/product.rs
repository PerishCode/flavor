use std::collections::BTreeMap;

use crate::{Diagnostic, LineIndex, Span};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ProductId(usize);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarProduct {
    pub id: ProductId,
    pub grammar_id: &'static str,
    pub entrypoint: &'static str,
    pub diagnostics: Vec<ProductDiagnostic>,
    pub facts: Vec<Fact>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProductDiagnostic {
    pub product_id: ProductId,
    pub message: String,
    pub span: Option<Span>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Fact {
    pub product_id: ProductId,
    pub key: &'static str,
    pub span: Option<Span>,
    pub line: Option<usize>,
    payload: FactPayload,
}

impl Fact {
    pub fn text(&self, key: &'static str) -> Option<&str> {
        self.payload.get_text(key)
    }

    pub fn bool(&self, key: &'static str) -> Option<bool> {
        self.payload.get_bool(key)
    }

    pub fn usize(&self, key: &'static str) -> Option<usize> {
        self.payload.get_usize(key)
    }

    pub fn texts(&self, key: &'static str) -> Option<&[String]> {
        self.payload.get_texts(key)
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct FactPayload {
    values: BTreeMap<&'static str, FactValue>,
}

impl FactPayload {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.values.insert(key, FactValue::Text(value.into()));
        self
    }

    pub fn bool(mut self, key: &'static str, value: bool) -> Self {
        self.values.insert(key, FactValue::Bool(value));
        self
    }

    pub fn usize(mut self, key: &'static str, value: usize) -> Self {
        self.values.insert(key, FactValue::Usize(value));
        self
    }

    pub fn texts(mut self, key: &'static str, values: Vec<String>) -> Self {
        self.values.insert(key, FactValue::Texts(values));
        self
    }

    fn get_text(&self, key: &'static str) -> Option<&str> {
        match self.values.get(key)? {
            FactValue::Text(value) => Some(value),
            _ => None,
        }
    }

    fn get_bool(&self, key: &'static str) -> Option<bool> {
        match self.values.get(key)? {
            FactValue::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn get_usize(&self, key: &'static str) -> Option<usize> {
        match self.values.get(key)? {
            FactValue::Usize(value) => Some(*value),
            _ => None,
        }
    }

    fn get_texts(&self, key: &'static str) -> Option<&[String]> {
        match self.values.get(key)? {
            FactValue::Texts(values) => Some(values),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum FactValue {
    Bool(bool),
    Text(String),
    Texts(Vec<String>),
    Usize(usize),
}

pub fn product(
    products: &mut Vec<GrammarProduct>,
    grammar_id: &'static str,
    entrypoint: &'static str,
    diagnostics: Vec<PendingDiagnostic>,
    facts: Vec<PendingFact>,
) {
    let product_id = ProductId(products.len());
    products.push(GrammarProduct {
        id: product_id,
        grammar_id,
        entrypoint,
        diagnostics: diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.finish(product_id))
            .collect(),
        facts: facts
            .into_iter()
            .map(|fact| fact.finish(product_id))
            .collect(),
    });
}

pub fn diagnostics(
    diagnostics: Vec<Diagnostic>,
    line_index: &LineIndex,
    line_offset: usize,
) -> Vec<PendingDiagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| PendingDiagnostic {
            line: diagnostic
                .span
                .map(|span| line_index.position(span.start).line as usize + line_offset),
            span: diagnostic.span,
            message: diagnostic.message,
        })
        .collect()
}

pub struct PendingDiagnostic {
    pub message: String,
    pub span: Option<Span>,
    pub line: Option<usize>,
}

impl PendingDiagnostic {
    pub fn finish(self, product_id: ProductId) -> ProductDiagnostic {
        ProductDiagnostic {
            product_id,
            message: self.message,
            span: self.span,
            line: self.line,
        }
    }
}

pub struct PendingFact {
    key: &'static str,
    span: Option<Span>,
    line: Option<usize>,
    payload: FactPayload,
}

impl PendingFact {
    pub fn new(key: &'static str, payload: FactPayload) -> Self {
        Self {
            key,
            span: None,
            line: None,
            payload,
        }
    }

    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    fn finish(self, product_id: ProductId) -> Fact {
        Fact {
            product_id,
            key: self.key,
            span: self.span,
            line: self.line,
            payload: self.payload,
        }
    }
}
