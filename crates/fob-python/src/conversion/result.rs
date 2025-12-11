//! Bundle result conversion

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Convert fob_bundler::BuildResult to Python dict
pub fn build_result_to_py_dict(
    py: Python,
    result: &fob_bundler::BuildResult,
) -> PyResult<Py<PyAny>> {
    let manifest = result.manifest();
    let stats = result.build_stats();

    // Build chunks list
    let chunks = PyList::empty(py);
    for chunk in result.chunks() {
        let chunk_dict = PyDict::new(py);

        let kind = if chunk.is_entry {
            "entry"
        } else if chunk.is_dynamic_entry {
            "async"
        } else {
            "shared"
        };

        chunk_dict.set_item("id", chunk.filename.to_string())?;
        chunk_dict.set_item("kind", kind)?;
        chunk_dict.set_item("file_name", chunk.filename.to_string())?;
        chunk_dict.set_item("code", chunk.code.to_string())?;

        if let Some(map) = &chunk.map {
            chunk_dict.set_item("source_map", map.to_json_string())?;
        } else {
            chunk_dict.set_item("source_map", py.None())?;
        }

        // Build modules list
        let modules = PyList::empty(py);
        for path in chunk.modules.keys.iter() {
            let module_dict = PyDict::new(py);
            module_dict.set_item("path", path.to_string())?;
            module_dict.set_item("size", py.None())?;
            module_dict.set_item("has_side_effects", py.None())?;
            modules.append(module_dict)?;
        }
        chunk_dict.set_item("modules", modules)?;

        // Build imports lists
        let imports: Vec<String> = chunk.imports.iter().map(|s| s.to_string()).collect();
        chunk_dict.set_item("imports", imports)?;

        let dynamic_imports: Vec<String> = chunk
            .dynamic_imports
            .iter()
            .map(|s| s.to_string())
            .collect();
        chunk_dict.set_item("dynamic_imports", dynamic_imports)?;

        chunk_dict.set_item("size", chunk.code.len() as u32)?;

        chunks.append(chunk_dict)?;
    }

    // Build assets list
    let assets = PyList::empty(py);
    for asset in result.assets() {
        let asset_dict = PyDict::new(py);
        asset_dict.set_item("public_path", format!("/{}", asset.filename))?;
        asset_dict.set_item("relative_path", asset.filename.to_string())?;
        asset_dict.set_item("size", asset.source.as_bytes().len() as u32)?;
        asset_dict.set_item("format", py.None())?;
        assets.append(asset_dict)?;
    }

    // Build manifest dict
    let manifest_dict = PyDict::new(py);
    let entries_dict = PyDict::new(py);
    for (k, v) in &manifest.entries {
        entries_dict.set_item(k, v)?;
    }
    manifest_dict.set_item("entries", entries_dict)?;

    let chunks_meta_dict = PyDict::new(py);
    for (k, v) in &manifest.chunks {
        let chunk_meta_dict = PyDict::new(py);
        chunk_meta_dict.set_item("file", &v.file)?;
        chunk_meta_dict.set_item("imports", &v.imports)?;
        chunk_meta_dict.set_item("dynamic_imports", &v.dynamic_imports)?;
        chunk_meta_dict.set_item("css", &v.css)?;
        chunks_meta_dict.set_item(k, chunk_meta_dict)?;
    }
    manifest_dict.set_item("chunks", chunks_meta_dict)?;
    manifest_dict.set_item("version", &manifest.version)?;

    // Build stats dict
    let stats_dict = PyDict::new(py);
    stats_dict.set_item("total_modules", stats.total_modules as u32)?;
    stats_dict.set_item("total_chunks", stats.total_chunks as u32)?;
    stats_dict.set_item("total_size", stats.total_size as u32)?;
    stats_dict.set_item("duration_ms", stats.duration_ms as u32)?;
    stats_dict.set_item("cache_hit_rate", stats.cache_hit_rate)?;

    // Build final result dict
    let result_dict = PyDict::new(py);
    result_dict.set_item("chunks", chunks)?;
    result_dict.set_item("manifest", manifest_dict)?;
    result_dict.set_item("stats", stats_dict)?;
    result_dict.set_item("assets", assets)?;
    result_dict.set_item("module_count", stats.total_modules as u32)?;

    Ok(result_dict.unbind().into())
}
