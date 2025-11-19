use napi_derive::napi;
use std::collections::HashMap;

/// Result of a bundle operation
#[napi(object)]
pub struct BundleResult {
    /// Generated chunks
    pub chunks: Vec<ChunkInfo>,

    /// Bundle manifest
    pub manifest: ManifestInfo,

    /// Build statistics
    pub stats: BuildStatsInfo,

    /// Static assets
    pub assets: Vec<AssetInfo>,
}

/// Detailed chunk information
#[napi(object)]
pub struct ChunkInfo {
    /// Chunk identifier
    pub id: String,

    /// Chunk type: "entry" | "async" | "shared"
    pub kind: String,

    /// Output file name
    pub file_name: String,

    /// Generated code
    pub code: String,

    /// Source map (optional)
    pub source_map: Option<String>,

    /// Modules in this chunk
    pub modules: Vec<ModuleInfo>,

    /// Static imports
    pub imports: Vec<String>,

    /// Dynamic imports
    pub dynamic_imports: Vec<String>,

    /// Size in bytes
    pub size: u32,
}

/// Module information
#[napi(object)]
pub struct ModuleInfo {
    /// Module path
    pub path: String,

    /// Module size
    pub size: u32,

    /// Has side effects
    pub has_side_effects: bool,
}

/// Bundle manifest
#[napi(object)]
pub struct ManifestInfo {
    /// Entry mappings
    pub entries: HashMap<String, String>,

    /// Chunk metadata
    pub chunks: HashMap<String, ChunkMetadata>,

    /// Version
    pub version: String,
}

/// Chunk metadata
#[napi(object)]
pub struct ChunkMetadata {
    pub file: String,
    pub imports: Vec<String>,
    pub dynamic_imports: Vec<String>,
    pub css: Vec<String>,
}

/// Build statistics
#[napi(object)]
pub struct BuildStatsInfo {
    pub total_modules: u32,
    pub total_chunks: u32,
    pub total_size: u32,
    pub duration_ms: u32,
    pub cache_hit_rate: f64,
}

/// Asset information
#[napi(object)]
pub struct AssetInfo {
    pub public_path: String,
    pub relative_path: String,
    pub size: u32,
    pub format: Option<String>,
}

/// Convert from fob_bundler types to NAPI types
impl From<fob_bundler::BuildResult> for BundleResult {
    fn from(result: fob_bundler::BuildResult) -> Self {
        let manifest = result.manifest();
        let stats = result.build_stats();

        let chunks = result
            .chunks()
            .map(|chunk| ChunkInfo {
                id: chunk.filename.to_string(),
                kind: if chunk.is_entry {
                    "entry".to_string()
                } else if chunk.is_dynamic_entry {
                    "async".to_string()
                } else {
                    "shared".to_string()
                },
                file_name: chunk.filename.to_string(),
                code: chunk.code.to_string(),
                source_map: chunk.map.as_ref().map(|m| m.to_json_string()),
                // Use module keys as paths - RenderedModule doesn't expose detailed info
                modules: chunk
                    .modules
                    .keys
                    .iter()
                    .map(|path| ModuleInfo {
                        path: path.to_string(),
                        size: 0, // RenderedModule doesn't expose size
                        has_side_effects: false, // RenderedModule doesn't expose side effects
                    })
                    .collect(),
                imports: chunk.imports.iter().map(|s| s.to_string()).collect(),
                dynamic_imports: chunk
                    .dynamic_imports
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                size: chunk.code.len() as u32,
            })
            .collect();

        let assets = result
            .assets()
            .map(|asset| AssetInfo {
                public_path: format!("/{}", asset.filename),
                relative_path: asset.filename.to_string(),
                size: asset.source.as_bytes().len() as u32,
                format: None, // Could infer from extension
            })
            .collect();

        Self {
            chunks,
            manifest: ManifestInfo {
                entries: manifest.entries,
                chunks: manifest
                    .chunks
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            k,
                            ChunkMetadata {
                                file: v.file,
                                imports: v.imports,
                                dynamic_imports: v.dynamic_imports,
                                css: v.css,
                            },
                        )
                    })
                    .collect(),
                version: manifest.version,
            },
            stats: BuildStatsInfo {
                total_modules: stats.total_modules as u32,
                total_chunks: stats.total_chunks as u32,
                total_size: stats.total_size as u32,
                duration_ms: stats.duration_ms as u32,
                cache_hit_rate: stats.cache_hit_rate,
            },
            assets,
        }
    }
}
