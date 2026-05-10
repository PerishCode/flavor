mod line_index;
mod span;

pub use line_index::{LineIndex, Position};
pub use span::Span;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SourceText {
    name: String,
    text: String,
}

impl SourceText {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: text.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn slice(&self, span: Span) -> &str {
        &self.text[span.start as usize..span.end as usize]
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn line_index(&self) -> LineIndex {
        LineIndex::new(&self.text)
    }
}
