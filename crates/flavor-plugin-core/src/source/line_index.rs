use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LineIndex {
    line_starts: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push((index + 1) as u32);
            }
        }
        Self { line_starts }
    }

    pub fn position(&self, offset: u32) -> Position {
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts[line_index];
        Position {
            line: line_index as u32 + 1,
            column: offset.saturating_sub(line_start) + 1,
        }
    }

    pub fn line(&self, offset: u32) -> usize {
        self.position(offset).line as usize
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn line_start(&self, line: u32) -> Option<u32> {
        if line == 0 {
            return None;
        }
        self.line_starts.get(line as usize - 1).copied()
    }
}
