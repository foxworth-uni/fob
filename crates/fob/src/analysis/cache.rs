
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheAnalysis {
    pub hit_rate: f64,
    pub hits: u64,
    pub misses: u64,
    pub total_requests: u64,
    pub time_saved_ms: u64,
}

impl CacheAnalysis {
    pub fn new(hits: u64, misses: u64, time_saved_ms: u64) -> Self {
        let total = hits + misses;
        let hit_rate = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };
        Self {
            hit_rate,
            hits,
            misses,
            total_requests: total,
            time_saved_ms,
        }
    }

    pub fn effectiveness(&self) -> CacheEffectiveness {
        match self.hit_rate {
            r if r >= 0.9 => CacheEffectiveness::Excellent,
            r if r >= 0.7 => CacheEffectiveness::Good,
            r if r >= 0.5 => CacheEffectiveness::Fair,
            r if r >= 0.3 => CacheEffectiveness::Poor,
            _ => CacheEffectiveness::VeryPoor,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.hit_rate > 0.5
    }
}

impl fmt::Display for CacheAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cache Analysis:")?;
        writeln!(f, "  Hit rate: {:.1}%", self.hit_rate * 100.0)?;
        writeln!(f, "  Hits: {}", self.hits)?;
        writeln!(f, "  Misses: {}", self.misses)?;
        writeln!(f, "  Requests: {}", self.total_requests)?;
        writeln!(f, "  Time saved: {}ms", self.time_saved_ms)?;
        writeln!(f, "  Effectiveness: {:?}", self.effectiveness())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheEffectiveness {
    Excellent,
    Good,
    Fair,
    Poor,
    VeryPoor,
}
