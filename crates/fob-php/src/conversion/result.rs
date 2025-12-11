//! Bundle result conversion

use ext_php_rs::convert::IntoZval;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendHashTable, Zval};

/// Convert fob_bundler::BuildResult to PHP array
pub fn build_result_to_php_array(result: &fob_bundler::BuildResult) -> PhpResult<Zval> {
    let manifest = result.manifest();
    let stats = result.build_stats();

    // Build chunks array
    let mut chunks = ZendHashTable::new();
    for chunk in result.chunks() {
        let mut chunk_arr = ZendHashTable::new();

        let kind = if chunk.is_entry {
            "entry"
        } else if chunk.is_dynamic_entry {
            "async"
        } else {
            "shared"
        };

        let _ = chunk_arr.insert("id", chunk.filename.to_string());
        let _ = chunk_arr.insert("kind", kind);
        let _ = chunk_arr.insert("file_name", chunk.filename.to_string());
        let _ = chunk_arr.insert("code", chunk.code.to_string());

        if let Some(map) = &chunk.map {
            let _ = chunk_arr.insert("source_map", map.to_json_string());
        } else {
            let _ = chunk_arr.insert("source_map", None::<()>);
        }

        // Build modules array
        let mut modules = ZendHashTable::new();
        for path in chunk.modules.keys.iter() {
            let mut module_arr = ZendHashTable::new();
            let _ = module_arr.insert("path", path.to_string());
            let _ = module_arr.insert("size", None::<()>);
            let _ = module_arr.insert("has_side_effects", None::<()>);
            let _ = modules.push(module_arr);
        }
        let _ = chunk_arr.insert("modules", modules);

        // Build imports arrays
        let imports: Vec<String> = chunk.imports.iter().map(|s| s.to_string()).collect();
        let _ = chunk_arr.insert("imports", imports);

        let dynamic_imports: Vec<String> = chunk
            .dynamic_imports
            .iter()
            .map(|s| s.to_string())
            .collect();
        let _ = chunk_arr.insert("dynamic_imports", dynamic_imports);

        let _ = chunk_arr.insert("size", chunk.code.len() as u32);

        let _ = chunks.push(chunk_arr);
    }

    // Build assets array
    let mut assets = ZendHashTable::new();
    for asset in result.assets() {
        let mut asset_arr = ZendHashTable::new();
        let _ = asset_arr.insert("public_path", format!("/{}", asset.filename));
        let _ = asset_arr.insert("relative_path", asset.filename.to_string());
        let _ = asset_arr.insert("size", asset.source.as_bytes().len() as u32);
        let _ = asset_arr.insert("format", None::<()>);
        let _ = assets.push(asset_arr);
    }

    // Build manifest array
    let mut manifest_arr = ZendHashTable::new();
    let mut entries_arr = ZendHashTable::new();
    for (k, v) in &manifest.entries {
        let _ = entries_arr.insert(k.clone(), v.clone());
    }
    let _ = manifest_arr.insert("entries", entries_arr);

    let mut chunks_meta_arr = ZendHashTable::new();
    for (k, v) in &manifest.chunks {
        let mut chunk_meta_arr = ZendHashTable::new();
        let _ = chunk_meta_arr.insert("file", v.file.clone());
        let _ = chunk_meta_arr.insert("imports", v.imports.clone());
        let _ = chunk_meta_arr.insert("dynamic_imports", v.dynamic_imports.clone());
        let _ = chunk_meta_arr.insert("css", v.css.clone());
        let _ = chunks_meta_arr.insert(k.clone(), chunk_meta_arr);
    }
    let _ = manifest_arr.insert("chunks", chunks_meta_arr);
    let _ = manifest_arr.insert("version", manifest.version.clone());

    // Build stats array
    let mut stats_arr = ZendHashTable::new();
    let _ = stats_arr.insert("total_modules", stats.total_modules as u32);
    let _ = stats_arr.insert("total_chunks", stats.total_chunks as u32);
    let _ = stats_arr.insert("total_size", stats.total_size as u32);
    let _ = stats_arr.insert("duration_ms", stats.duration_ms as u32);
    let _ = stats_arr.insert("cache_hit_rate", stats.cache_hit_rate);

    // Build final result array
    let mut result_arr = ZendHashTable::new();
    let _ = result_arr.insert("chunks", chunks);
    let _ = result_arr.insert("manifest", manifest_arr);
    let _ = result_arr.insert("stats", stats_arr);
    let _ = result_arr.insert("assets", assets);
    let _ = result_arr.insert("module_count", stats.total_modules as u32);

    let mut zval = Zval::new();
    result_arr.set_zval(&mut zval, false)?;
    Ok(zval)
}
