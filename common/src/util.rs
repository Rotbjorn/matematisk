use std::{collections::HashMap, fmt::Display};

#[cfg(target_arch = "wasm32")]
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Serialize))]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("(row {}, col {})", self.row, self.col))
    }
}

pub type SymbolTable<T> = HashMap<String, T>;
