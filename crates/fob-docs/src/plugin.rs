use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustc_hash::FxHashSet;

use crate::{
    extractor::{DocsExtractor, ExtractOptions},
    model::Documentation,
};

#[cfg(feature = "json")]
use crate::generators::json::render_json;
#[cfg(feature = "markdown")]
use crate::generators::markdown::render_markdown;

use rolldown_common::{Output, OutputAsset};
use rolldown_plugin::{HookGenerateBundleArgs, HookNoopReturn, HookUsage, Plugin, PluginContext};

/// Output formats supported by the documentation plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocsPluginOutputFormat {
    Markdown,
    Json,
    Both,
}

/// Configuration for [`DocsEmitPlugin`].
#[derive(Debug, Clone)]
pub struct DocsEmitPluginOptions {
    pub output_format: DocsPluginOutputFormat,
    pub output_dir: Option<PathBuf>,
    pub include_internal: bool,

    /// Optional LLM configuration for documentation enhancement
    #[cfg(feature = "llm")]
    pub llm_config: Option<crate::llm::LlmConfig>,
}

impl Default for DocsEmitPluginOptions {
    fn default() -> Self {
        Self {
            output_format: DocsPluginOutputFormat::Markdown,
            output_dir: Some(PathBuf::from("docs")),
            include_internal: false,
            #[cfg(feature = "llm")]
            llm_config: None,
        }
    }
}

/// Rolldown plugin that emits documentation in Markdown / JSON formats.
#[derive(Debug, Clone)]
pub struct DocsEmitPlugin {
    options: DocsEmitPluginOptions,
}

impl DocsEmitPlugin {
    pub fn new(options: DocsEmitPluginOptions) -> Self {
        Self { options }
    }
}

impl Default for DocsEmitPlugin {
    fn default() -> Self {
        Self::new(DocsEmitPluginOptions::default())
    }
}

impl Plugin for DocsEmitPlugin {
    fn name(&self) -> Cow<'static, str> {
        "fob-docs-emit".into()
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::GenerateBundle
    }

    fn generate_bundle(
        &self,
        _ctx: &PluginContext,
        args: &mut HookGenerateBundleArgs<'_>,
    ) -> impl std::future::Future<Output = HookNoopReturn> + Send {
        let options = self.options.clone();

        async move {
            let mut documentation = Documentation::default();
            let mut processed = FxHashSet::default();

            let extractor = DocsExtractor::new(ExtractOptions {
                include_internal: options.include_internal,
            });

            for output in args.bundle.iter() {
                if let Output::Chunk(chunk) = output {
                    for module_id in chunk.modules.keys.iter() {
                        let path_str = module_id.as_ref();
                        if !is_supported_module(path_str) {
                            continue;
                        }
                        if !processed.insert(path_str.to_string()) {
                            continue;
                        }

                        match std::fs::read_to_string(path_str) {
                            Ok(source) => {
                                match extractor.extract_from_source(Path::new(path_str), &source) {
                                    Ok(module_doc) => {
                                        if !module_doc.symbols.is_empty() {
                                            documentation.add_module(module_doc);
                                        }
                                    }
                                    Err(error) => warn(&format!(
                                        "Failed to extract docs for {}: {error}",
                                        path_str
                                    )),
                                }
                            }
                            Err(error) => {
                                warn(&format!("Failed to read module {}: {error}", path_str))
                            }
                        }
                    }
                }
            }

            if documentation.modules.is_empty() {
                return Ok(());
            }

            // Apply LLM enhancement if configured
            #[cfg(feature = "llm")]
            let documentation = if let Some(ref llm_config) = options.llm_config {
                // Try to initialize enhancer first, before moving documentation
                match crate::llm::LlmEnhancer::new(llm_config.clone()).await {
                    Ok(enhancer) => {
                        let total_symbols: usize =
                            documentation.modules.iter().map(|m| m.symbols.len()).sum();

                        eprintln!("[LLM] Enhancing {} symbols...", total_symbols);

                        // Now we can safely move documentation into enhance_documentation
                        match enhancer
                            .enhance_documentation(documentation, |current, total| {
                                if current % 5 == 0 || current == total {
                                    eprintln!("[LLM] Progress: {}/{}", current, total);
                                }
                            })
                            .await
                        {
                            Ok(enhanced) => {
                                eprintln!("[LLM] Enhancement complete!");
                                enhanced
                            }
                            Err(e) => {
                                warn(&format!(
                                    "LLM enhancement failed (original docs cannot be recovered): {}",
                                    e
                                ));
                                // Return empty documentation since we moved the original
                                Documentation::default()
                            }
                        }
                    }
                    Err(e) => {
                        warn(&format!(
                            "Failed to initialize LLM enhancer (continuing without enhancement): {}",
                            e
                        ));
                        // Enhancer init failed, so we still have documentation
                        documentation
                    }
                }
            } else {
                documentation
            };

            let original_files: Vec<String> = documentation
                .modules
                .iter()
                .map(|module| module.path.clone())
                .collect();

            let mut assets = Vec::new();
            match options.output_format {
                DocsPluginOutputFormat::Markdown => {
                    if let Some(asset) = emit_markdown_asset(
                        &documentation,
                        options.output_dir.as_ref(),
                        original_files.clone(),
                    ) {
                        assets.push(asset);
                    }
                }
                DocsPluginOutputFormat::Json => {
                    if let Some(asset) = emit_json_asset(
                        &documentation,
                        options.output_dir.as_ref(),
                        original_files.clone(),
                    ) {
                        assets.push(asset);
                    }
                }
                DocsPluginOutputFormat::Both => {
                    if let Some(asset) = emit_markdown_asset(
                        &documentation,
                        options.output_dir.as_ref(),
                        original_files.clone(),
                    ) {
                        assets.push(asset);
                    }
                    if let Some(asset) =
                        emit_json_asset(&documentation, options.output_dir.as_ref(), original_files)
                    {
                        assets.push(asset);
                    }
                }
            }

            args.bundle.extend(assets);

            Ok(())
        }
    }
}

#[cfg(feature = "markdown")]
fn emit_markdown_asset(
    documentation: &Documentation,
    output_dir: Option<&PathBuf>,
    original_files: Vec<String>,
) -> Option<Output> {
    let filename = make_path(output_dir, "documentation.md");
    let content = render_markdown(documentation);
    Some(make_asset(filename, content, original_files))
}

#[cfg(not(feature = "markdown"))]
fn emit_markdown_asset(
    _documentation: &Documentation,
    _output_dir: Option<&PathBuf>,
    _original_files: Vec<String>,
) -> Option<Output> {
    warn("Markdown output requested but the 'markdown' feature is disabled.");
    None
}

#[cfg(feature = "json")]
fn emit_json_asset(
    documentation: &Documentation,
    output_dir: Option<&PathBuf>,
    original_files: Vec<String>,
) -> Option<Output> {
    match render_json(documentation) {
        Ok(content) => {
            let filename = make_path(output_dir, "documentation.json");
            Some(make_asset(filename, content, original_files))
        }
        Err(error) => {
            warn(&format!("Failed to render JSON documentation: {error}"));
            None
        }
    }
}

#[cfg(not(feature = "json"))]
fn emit_json_asset(
    _documentation: &Documentation,
    _output_dir: Option<&PathBuf>,
    _original_files: Vec<String>,
) -> Option<Output> {
    warn("JSON output requested but the 'json' feature is disabled.");
    None
}

fn make_asset(filename: String, content: String, original_files: Vec<String>) -> Output {
    let asset = OutputAsset {
        names: vec![],
        original_file_names: original_files,
        filename: filename.into(),
        source: content.into(),
    };
    Output::Asset(Arc::new(asset))
}

fn make_path(output_dir: Option<&PathBuf>, file: &str) -> String {
    match output_dir {
        Some(dir) => dir.join(file).to_string_lossy().replace('\\', "/"),
        None => file.to_string(),
    }
}

fn is_supported_module(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext,
                "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "mts" | "cts"
            )
        })
        .unwrap_or(false)
}

fn warn(message: &str) {
    eprintln!("[fob-docs] {message}");
}
