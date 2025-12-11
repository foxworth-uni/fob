//! Bundle result conversion

use magnus::{RHash, Ruby};

/// Convert fob_bundler::BuildResult to Ruby Hash
pub fn build_result_to_ruby_hash(
    ruby: &Ruby,
    result: &fob_bundler::BuildResult,
) -> Result<RHash, magnus::Error> {
    let manifest = result.manifest();
    let stats = result.build_stats();

    // Build chunks array
    let chunks = ruby.ary_new();
    for chunk in result.chunks() {
        let chunk_hash = ruby.hash_new();

        let kind = if chunk.is_entry {
            "entry"
        } else if chunk.is_dynamic_entry {
            "async"
        } else {
            "shared"
        };

        chunk_hash.aset(ruby.sym_new("id"), chunk.filename.to_string())?;
        chunk_hash.aset(ruby.sym_new("kind"), kind)?;
        chunk_hash.aset(ruby.sym_new("file_name"), chunk.filename.to_string())?;
        chunk_hash.aset(ruby.sym_new("code"), chunk.code.to_string())?;

        if let Some(map) = &chunk.map {
            chunk_hash.aset(ruby.sym_new("source_map"), map.to_json_string())?;
        } else {
            chunk_hash.aset(ruby.sym_new("source_map"), ruby.qnil())?;
        }

        // Build modules array
        let modules = ruby.ary_new();
        for path in chunk.modules.keys.iter() {
            let module_hash = ruby.hash_new();
            module_hash.aset(ruby.sym_new("path"), path.to_string())?;
            module_hash.aset(ruby.sym_new("size"), ruby.qnil())?;
            module_hash.aset(ruby.sym_new("has_side_effects"), ruby.qnil())?;
            modules.push(module_hash)?;
        }
        chunk_hash.aset(ruby.sym_new("modules"), modules)?;

        // Build imports arrays
        let imports: Vec<String> = chunk.imports.iter().map(|s| s.to_string()).collect();
        chunk_hash.aset(ruby.sym_new("imports"), imports)?;

        let dynamic_imports: Vec<String> = chunk
            .dynamic_imports
            .iter()
            .map(|s| s.to_string())
            .collect();
        chunk_hash.aset(ruby.sym_new("dynamic_imports"), dynamic_imports)?;

        chunk_hash.aset(ruby.sym_new("size"), chunk.code.len() as u32)?;

        chunks.push(chunk_hash)?;
    }

    // Build assets array
    let assets = ruby.ary_new();
    for asset in result.assets() {
        let asset_hash = ruby.hash_new();
        asset_hash.aset(ruby.sym_new("public_path"), format!("/{}", asset.filename))?;
        asset_hash.aset(ruby.sym_new("relative_path"), asset.filename.to_string())?;
        asset_hash.aset(ruby.sym_new("size"), asset.source.as_bytes().len() as u32)?;
        asset_hash.aset(ruby.sym_new("format"), ruby.qnil())?;
        assets.push(asset_hash)?;
    }

    // Build manifest hash
    let manifest_hash = ruby.hash_new();
    let entries_hash = ruby.hash_new();
    for (k, v) in &manifest.entries {
        entries_hash.aset(k.clone(), v.clone())?;
    }
    manifest_hash.aset(ruby.sym_new("entries"), entries_hash)?;

    let chunks_meta_hash = ruby.hash_new();
    for (k, v) in &manifest.chunks {
        let chunk_meta_hash = ruby.hash_new();
        chunk_meta_hash.aset(ruby.sym_new("file"), v.file.clone())?;
        chunk_meta_hash.aset(ruby.sym_new("imports"), v.imports.clone())?;
        chunk_meta_hash.aset(ruby.sym_new("dynamic_imports"), v.dynamic_imports.clone())?;
        chunk_meta_hash.aset(ruby.sym_new("css"), v.css.clone())?;
        chunks_meta_hash.aset(k.clone(), chunk_meta_hash)?;
    }
    manifest_hash.aset(ruby.sym_new("chunks"), chunks_meta_hash)?;
    manifest_hash.aset(ruby.sym_new("version"), manifest.version.clone())?;

    // Build stats hash
    let stats_hash = ruby.hash_new();
    stats_hash.aset(ruby.sym_new("total_modules"), stats.total_modules as u32)?;
    stats_hash.aset(ruby.sym_new("total_chunks"), stats.total_chunks as u32)?;
    stats_hash.aset(ruby.sym_new("total_size"), stats.total_size as u32)?;
    stats_hash.aset(ruby.sym_new("duration_ms"), stats.duration_ms as u32)?;
    stats_hash.aset(ruby.sym_new("cache_hit_rate"), stats.cache_hit_rate)?;

    // Build final result hash
    let result_hash = ruby.hash_new();
    result_hash.aset(ruby.sym_new("chunks"), chunks)?;
    result_hash.aset(ruby.sym_new("manifest"), manifest_hash)?;
    result_hash.aset(ruby.sym_new("stats"), stats_hash)?;
    result_hash.aset(ruby.sym_new("assets"), assets)?;
    result_hash.aset(ruby.sym_new("module_count"), stats.total_modules as u32)?;

    Ok(result_hash)
}
