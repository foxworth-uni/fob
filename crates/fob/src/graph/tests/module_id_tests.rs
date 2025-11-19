use std::path::PathBuf;

use serde_json::{from_str, to_string};
use tempfile::tempdir;

use super::super::{Module, ModuleId, ModuleIdError, SourceSpan, SourceType};

#[test]
fn canonicalizes_relative_path() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("entry.ts");
    std::fs::write(&file_path, "export const value = 1;").unwrap();

    let guard = DirGuard::new(temp.path());
    let module_id = ModuleId::new("./entry.ts").unwrap();
    let canonical = ModuleId::from_canonical_path(file_path.canonicalize().unwrap());

    assert_eq!(module_id.as_path(), &file_path.canonicalize().unwrap());
    assert_eq!(module_id, canonical);
    drop(guard);
}

#[test]
fn rejects_empty_path() {
    let err = ModuleId::new(PathBuf::new()).unwrap_err();
    assert!(matches!(err, ModuleIdError::EmptyPath));
}

#[test]
fn supports_virtual_modules() {
    let virtual_id = ModuleId::new_virtual("react-refresh:runtime");
    assert!(virtual_id.is_virtual());
    assert_eq!(virtual_id.path_string(), "virtual:react-refresh:runtime");
}

#[test]
fn serde_roundtrip() {
    let real_id = ModuleId::new_virtual("in-memory");
    let json = to_string(&real_id).unwrap();
    let restored: ModuleId = from_str(&json).unwrap();
    assert_eq!(restored, real_id);
}

#[test]
fn span_to_line_column() {
    let span = SourceSpan::new("/tmp/foo.ts", 13, 20);
    let source = "const x = 1;\nexport const y = x;";
    let (line, col) = span.to_line_col(source);
    assert_eq!(line, 2);
    assert_eq!(col, 1);
}

#[test]
fn span_merge_same_file() {
    let a = SourceSpan::new("/tmp/foo.ts", 0, 5);
    let b = SourceSpan::new("/tmp/foo.ts", 10, 20);
    let merged = a.merge(&b).unwrap();
    assert_eq!(merged.start, 0);
    assert_eq!(merged.end, 20);
}

#[test]
fn module_builder_applies_flags() {
    let id = ModuleId::new_virtual("component");
    let path = PathBuf::from(id.path_string().to_string());
    let module = Module::builder(id.clone(), path.clone(), SourceType::Tsx)
        .imports(Vec::new())
        .exports(Vec::new())
        .side_effects(true)
        .entry(true)
        .external(true)
        .original_size(512)
        .bundled_size(Some(256))
        .build();

    assert_eq!(module.id, id);
    assert_eq!(module.path, path);
    assert!(module.has_side_effects);
    assert!(module.is_entry);
    assert!(module.is_external);
    assert_eq!(module.original_size, 512);
    assert_eq!(module.bundled_size, Some(256));
}

struct DirGuard {
    original: PathBuf,
}

impl DirGuard {
    fn new(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.original).unwrap();
    }
}
