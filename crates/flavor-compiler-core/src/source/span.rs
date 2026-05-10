use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}
