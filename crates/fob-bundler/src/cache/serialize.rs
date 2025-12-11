//! Serialization layer for build cache.
//!
//! Rolldown types (`OutputChunk`, `OutputAsset`, etc.) don't implement serde
//! traits. This module provides serializable wrapper types and conversions.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::BundleOutput;
use crate::builders::asset_registry::{AssetInfo, AssetRegistry};
use fob_graph::{CacheAnalysis, TransformationTrace};

/// Current cache format version. Increment when format changes.
pub const CACHE_FORMAT_VERSION: u32 = 1;

/// Cached build data - serializable representation of `AnalyzedBundle`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBuild {
    /// Cache metadata for validation.
    pub metadata: CacheMetadata,

    /// Serialized bundle outputs (chunks and assets).
    pub outputs: Vec<SerializedOutput>,

    /// Serialized module graph (JSON).
    pub graph_json: String,

    /// Entry point module IDs.
    pub entry_points: Vec<String>,

    /// Warnings from analysis.
    pub warnings: Vec<String>,

    /// Errors from analysis.
    pub errors: Vec<String>,

    /// Cache metrics.
    pub cache: CacheAnalysis,

    /// Optional transformation trace.
    pub trace: Option<TransformationTrace>,

    /// Serialized asset registry.
    pub assets: Vec<SerializedAssetInfo>,
}

/// Cache metadata for validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// fob-bundler version that created this cache.
    pub fob_version: String,

    /// Cache format version.
    pub format_version: u32,

    /// Unix timestamp when cache was created.
    pub created_at: u64,
}

impl CacheMetadata {
    /// Create new metadata with current version and timestamp.
    pub fn new() -> Self {
        Self {
            fob_version: env!("CARGO_PKG_VERSION").to_string(),
            format_version: CACHE_FORMAT_VERSION,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Check if this metadata is compatible with current version.
    pub fn is_compatible(&self) -> bool {
        self.format_version == CACHE_FORMAT_VERSION
    }
}

impl Default for CacheMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialized output (chunk or asset).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedOutput {
    Chunk(SerializedChunk),
    Asset(SerializedAsset),
}

/// Serialized chunk from Rolldown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedChunk {
    pub name: String,
    pub filename: String,
    pub code: String,
    pub map_json: Option<String>,
    pub sourcemap_filename: Option<String>,
    pub preliminary_filename: String,
    pub is_entry: bool,
    pub is_dynamic_entry: bool,
    pub facade_module_id: Option<String>,
    pub module_ids: Vec<String>,
    pub imports: Vec<String>,
    pub dynamic_imports: Vec<String>,
    pub exports: Vec<String>,
    pub modules: Vec<SerializedRenderedModule>,
}

/// Serialized rendered module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedRenderedModule {
    pub id: String,
    pub code: Option<String>,
    pub rendered_exports: Vec<String>,
    pub exec_order: u32,
}

/// Serialized asset from Rolldown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAsset {
    pub names: Vec<String>,
    pub original_file_names: Vec<String>,
    pub filename: String,
    pub source: SerializedStrOrBytes,
}

/// Serialized string or bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializedStrOrBytes {
    Str(String),
    Bytes(Vec<u8>),
}

/// Serialized asset info from registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAssetInfo {
    pub source_path: String,
    pub referrer: String,
    pub specifier: String,
    pub content_type: String,
    pub size: Option<u64>,
    pub url_path: Option<String>,
    pub content_hash: Option<String>,
}

// ============================================================================
// Conversion: Rolldown types -> Serialized types
// ============================================================================

impl SerializedChunk {
    /// Convert from Rolldown OutputChunk.
    pub fn from_rolldown(chunk: &rolldown_common::OutputChunk) -> Self {
        Self {
            name: chunk.name.to_string(),
            filename: chunk.filename.to_string(),
            code: chunk.code.clone(),
            map_json: chunk.map.as_ref().map(|m| m.to_json_string()),
            sourcemap_filename: chunk.sourcemap_filename.clone(),
            preliminary_filename: chunk.preliminary_filename.clone(),
            is_entry: chunk.is_entry,
            is_dynamic_entry: chunk.is_dynamic_entry,
            facade_module_id: chunk.facade_module_id.as_ref().map(|id| id.to_string()),
            module_ids: chunk.module_ids.iter().map(|id| id.to_string()).collect(),
            imports: chunk.imports.iter().map(|s| s.to_string()).collect(),
            dynamic_imports: chunk
                .dynamic_imports
                .iter()
                .map(|s| s.to_string())
                .collect(),
            exports: chunk.exports.iter().map(|s| s.to_string()).collect(),
            modules: chunk
                .modules
                .keys
                .iter()
                .zip(chunk.modules.values.iter())
                .map(|(id, module)| SerializedRenderedModule {
                    id: id.to_string(),
                    code: module.code(),
                    rendered_exports: module
                        .rendered_exports
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    exec_order: module.exec_order,
                })
                .collect(),
        }
    }
}

impl SerializedAsset {
    /// Convert from Rolldown OutputAsset.
    pub fn from_rolldown(asset: &rolldown_common::OutputAsset) -> Self {
        Self {
            names: asset.names.clone(),
            original_file_names: asset.original_file_names.clone(),
            filename: asset.filename.to_string(),
            source: match &asset.source {
                rolldown_common::StrOrBytes::Str(s) => SerializedStrOrBytes::Str(s.clone()),
                rolldown_common::StrOrBytes::Bytes(b) => SerializedStrOrBytes::Bytes(b.clone()),
            },
        }
    }
}

impl SerializedOutput {
    /// Convert from Rolldown Output.
    pub fn from_rolldown(output: &rolldown_common::Output) -> Self {
        match output {
            rolldown_common::Output::Chunk(chunk) => {
                SerializedOutput::Chunk(SerializedChunk::from_rolldown(chunk))
            }
            rolldown_common::Output::Asset(asset) => {
                SerializedOutput::Asset(SerializedAsset::from_rolldown(asset))
            }
        }
    }
}

impl SerializedAssetInfo {
    /// Convert from AssetInfo.
    pub fn from_asset_info(info: &AssetInfo) -> Self {
        Self {
            source_path: info.source_path.to_string_lossy().to_string(),
            referrer: info.referrer.clone(),
            specifier: info.specifier.clone(),
            content_type: info.content_type.clone(),
            size: info.size,
            url_path: info.url_path.clone(),
            content_hash: info.content_hash.clone(),
        }
    }
}

// ============================================================================
// Conversion: Serialized types -> Rolldown types
// ============================================================================

impl SerializedChunk {
    /// Convert back to Rolldown OutputChunk.
    pub fn into_rolldown(self) -> rolldown_common::OutputChunk {
        use arcstr::ArcStr;

        // Build Modules struct
        let mut keys = Vec::with_capacity(self.modules.len());
        let mut values = Vec::with_capacity(self.modules.len());

        for module in self.modules {
            keys.push(rolldown_common::ModuleId::new(module.id));
            // Convert via String to avoid oxc_span version mismatch issues
            let rendered_exports = module
                .rendered_exports
                .into_iter()
                .map(|s| s.into())
                .collect();
            values.push(Arc::new(rolldown_common::RenderedModule::new(
                None, // We can't reconstruct the inner_code Sources
                rendered_exports,
                module.exec_order,
            )));
        }

        rolldown_common::OutputChunk {
            name: ArcStr::from(self.name),
            filename: ArcStr::from(self.filename),
            code: self.code,
            // Note: We store the sourcemap as JSON and return None here.
            // The JSON is preserved in case we need to reconstruct it later.
            map: None,
            sourcemap_filename: self.sourcemap_filename,
            preliminary_filename: self.preliminary_filename,
            is_entry: self.is_entry,
            is_dynamic_entry: self.is_dynamic_entry,
            facade_module_id: self.facade_module_id.map(rolldown_common::ModuleId::new),
            module_ids: self
                .module_ids
                .into_iter()
                .map(rolldown_common::ModuleId::new)
                .collect(),
            imports: self.imports.into_iter().map(ArcStr::from).collect(),
            dynamic_imports: self.dynamic_imports.into_iter().map(ArcStr::from).collect(),
            exports: self.exports.into_iter().map(|s| s.into()).collect(),
            modules: rolldown_common::Modules { keys, values },
        }
    }
}

impl SerializedAsset {
    /// Convert back to Rolldown OutputAsset.
    pub fn into_rolldown(self) -> rolldown_common::OutputAsset {
        use arcstr::ArcStr;

        rolldown_common::OutputAsset {
            names: self.names,
            original_file_names: self.original_file_names,
            filename: ArcStr::from(self.filename),
            source: match self.source {
                SerializedStrOrBytes::Str(s) => rolldown_common::StrOrBytes::Str(s),
                SerializedStrOrBytes::Bytes(b) => rolldown_common::StrOrBytes::Bytes(b),
            },
        }
    }
}

impl SerializedOutput {
    /// Convert back to Rolldown Output.
    pub fn into_rolldown(self) -> rolldown_common::Output {
        match self {
            SerializedOutput::Chunk(chunk) => {
                rolldown_common::Output::Chunk(Arc::new(chunk.into_rolldown()))
            }
            SerializedOutput::Asset(asset) => {
                rolldown_common::Output::Asset(Arc::new(asset.into_rolldown()))
            }
        }
    }
}

// ============================================================================
// CachedBuild conversion
// ============================================================================

/// Components needed to create a cached build.
pub struct BuildComponents<'a> {
    pub bundle: &'a BundleOutput,
    pub graph_json: String,
    pub entry_points: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub cache: &'a CacheAnalysis,
    pub trace: Option<&'a TransformationTrace>,
    pub asset_registry: Option<&'a Arc<AssetRegistry>>,
}

impl CachedBuild {
    /// Create from build components.
    pub fn from_components(components: BuildComponents<'_>) -> Self {
        // Serialize outputs
        let outputs: Vec<SerializedOutput> = components
            .bundle
            .assets
            .iter()
            .map(SerializedOutput::from_rolldown)
            .collect();

        // Serialize asset registry
        let assets = components
            .asset_registry
            .map(|registry| {
                registry
                    .all_assets()
                    .iter()
                    .map(SerializedAssetInfo::from_asset_info)
                    .collect()
            })
            .unwrap_or_default();

        Self {
            metadata: CacheMetadata::new(),
            outputs,
            graph_json: components.graph_json,
            entry_points: components.entry_points,
            warnings: components.warnings,
            errors: components.errors,
            cache: components.cache.clone(),
            trace: components.trace.cloned(),
            assets,
        }
    }

    /// Convert to BundleOutput.
    pub fn into_bundle_output(self) -> BundleOutput {
        BundleOutput {
            assets: self
                .outputs
                .into_iter()
                .map(|o| o.into_rolldown())
                .collect(),
            warnings: vec![], // Warnings are stored separately
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_metadata_compatibility() {
        let meta = CacheMetadata::new();
        assert!(meta.is_compatible());
        assert_eq!(meta.format_version, CACHE_FORMAT_VERSION);
    }

    #[test]
    fn test_serialized_str_or_bytes_roundtrip() {
        let str_val = SerializedStrOrBytes::Str("hello".to_string());
        let bytes_val = SerializedStrOrBytes::Bytes(vec![1, 2, 3]);

        // Test bincode serialization roundtrip
        let str_bytes = bincode::serialize(&str_val).unwrap();
        let str_back: SerializedStrOrBytes = bincode::deserialize(&str_bytes).unwrap();
        assert!(matches!(str_back, SerializedStrOrBytes::Str(s) if s == "hello"));

        let bytes_bytes = bincode::serialize(&bytes_val).unwrap();
        let bytes_back: SerializedStrOrBytes = bincode::deserialize(&bytes_bytes).unwrap();
        assert!(matches!(bytes_back, SerializedStrOrBytes::Bytes(b) if b == vec![1, 2, 3]));
    }
}
