#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RecoverySet<K> {
    tokens: Vec<K>,
}

impl<K> RecoverySet<K> {
    pub fn new(tokens: Vec<K>) -> Self {
        Self { tokens }
    }

    pub fn tokens(&self) -> &[K] {
        &self.tokens
    }
}

impl<K: PartialEq> RecoverySet<K> {
    pub fn contains(&self, token: &K) -> bool {
        self.tokens.contains(token)
    }
}
