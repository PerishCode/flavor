#![allow(dead_code)]

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct StyleFacts {
    pub scoped: bool,
    pub module: bool,
    pub uses_slotted: bool,
}
