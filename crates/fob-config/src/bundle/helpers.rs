use std::path::PathBuf;

// Helper defaults
pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_output_dir() -> PathBuf {
    PathBuf::from("dist")
}

pub(crate) fn default_shared_chunk_threshold() -> usize {
    20_000
}

pub(crate) fn default_static_dir() -> Option<PathBuf> {
    Some(PathBuf::from("public"))
}

pub(crate) fn default_mode() -> String {
    "development".to_string()
}

pub(crate) fn default_tailwind_output() -> String {
    "styles.css".to_string()
}

pub(crate) fn default_html_filename() -> String {
    "index.html".to_string()
}

pub(crate) fn default_lang() -> String {
    "en".to_string()
}

