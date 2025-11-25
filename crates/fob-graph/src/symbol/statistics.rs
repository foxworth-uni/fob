//! Statistics and analysis helpers for symbols.

use serde::{Deserialize, Serialize};

use super::{SymbolKind, SymbolTable};

/// Statistics about symbols across the entire graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolStatistics {
    /// Total number of symbols analyzed
    pub total_symbols: usize,
    /// Number of unused symbols detected
    pub unused_symbols: usize,
    /// Breakdown by symbol kind
    pub by_kind: Vec<(SymbolKind, usize)>,
}

impl SymbolStatistics {
    /// Create new symbol statistics.
    pub fn new(total_symbols: usize, unused_symbols: usize) -> Self {
        Self {
            total_symbols,
            unused_symbols,
            by_kind: Vec::new(),
        }
    }

    /// Create statistics from a collection of symbol tables.
    pub fn from_tables<'a, I>(tables: I) -> Self
    where
        I: Iterator<Item = &'a SymbolTable>,
    {
        let mut total_symbols = 0;
        let mut unused_symbols = 0;
        let mut kind_counts: std::collections::HashMap<SymbolKind, usize> =
            std::collections::HashMap::new();

        for table in tables {
            total_symbols += table.len();
            unused_symbols += table.unused_symbols().len();

            for symbol in &table.symbols {
                *kind_counts.entry(symbol.kind).or_insert(0) += 1;
            }
        }

        let by_kind = kind_counts.into_iter().collect();

        Self {
            total_symbols,
            unused_symbols,
            by_kind,
        }
    }

    /// Calculate the percentage of unused symbols.
    pub fn unused_percentage(&self) -> f64 {
        if self.total_symbols == 0 {
            0.0
        } else {
            (self.unused_symbols as f64 / self.total_symbols as f64) * 100.0
        }
    }
}
