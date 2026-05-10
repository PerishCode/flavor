use crate::Token;

#[derive(Debug, Clone)]
pub struct TokenCursor<K> {
    tokens: Vec<Token<K>>,
    position: usize,
}

impl<K> TokenCursor<K> {
    pub fn new(tokens: Vec<Token<K>>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn peek(&self) -> Option<&Token<K>> {
        self.tokens.get(self.position)
    }

    pub fn bump(&mut self) -> Option<&Token<K>> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}
