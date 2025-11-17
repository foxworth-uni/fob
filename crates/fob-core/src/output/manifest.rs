use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bundle manifest for runtime loading and preload optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifest {
    /// Entry point mappings: entry name -> output filename
    pub entries: HashMap<String, String>,

    /// Chunk metadata for dependency tracking
    pub chunks: HashMap<String, ChunkMetadata>,

    /// Build version/hash for cache invalidation
    pub version: String,
}

impl BundleManifest {
    /// Generate manifest from build output and analysis
    pub fn from_build_output(
        output: &super::super::builders::unified::BuildOutput,
        analysis: &crate::analysis::AnalysisResult,
    ) -> Self {
        use super::super::builders::unified::BuildOutput;
        use crate::Output;

        let mut entries = HashMap::new();
        let mut chunks = HashMap::new();

        // Helper to process a single chunk
        let mut process_chunk = |chunk: &crate::OutputChunk| {
            // Add entry mapping if this is an entry chunk
            if chunk.is_entry {
                // chunk.name is an ArcStr, not an Option
                entries.insert(chunk.name.to_string(), chunk.filename.to_string());
            }

            // Build chunk metadata - modules.keys is a field
            let modules: Vec<String> = chunk.modules.keys.iter().map(|k| k.to_string()).collect();

            chunks.insert(
                chunk.filename.to_string(),
                ChunkMetadata {
                    file: chunk.filename.to_string(),
                    imports: chunk.imports.iter().map(|s| s.to_string()).collect(),
                    dynamic_imports: chunk.dynamic_imports.iter().map(|s| s.to_string()).collect(),
                    css: vec![], // TODO: Extract CSS references
                    modules,
                },
            );
        };

        // Process chunks from single or multiple bundles
        match output {
            BuildOutput::Single(bundle) => {
                for asset in &bundle.assets {
                    if let Output::Chunk(chunk) = asset {
                        process_chunk(chunk);
                    }
                }
            }
            BuildOutput::Multiple(bundles) => {
                for bundle in bundles.values() {
                    for asset in &bundle.assets {
                        if let Output::Chunk(chunk) = asset {
                            process_chunk(chunk);
                        }
                    }
                }
            }
        }

        Self {
            entries,
            chunks,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Metadata for a single chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkMetadata {
    /// Output file name
    pub file: String,

    /// Static imports (for preloading)
    pub imports: Vec<String>,

    /// Dynamic imports (for prefetching)
    pub dynamic_imports: Vec<String>,

    /// Associated CSS files
    pub css: Vec<String>,

    /// Module IDs included in this chunk (for debugging)
    pub modules: Vec<String>,
}

/// Build statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildStats {
    pub total_modules: usize,
    pub total_chunks: usize,
    pub total_size: usize,
    pub duration_ms: u64,
    pub cache_hit_rate: f64,
}
