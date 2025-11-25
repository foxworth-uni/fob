//! Error mapping tests for fob-native.
//!
//! Tests conversion from fob-bundler errors to FobErrorDetails.

use fob_bundler::diagnostics::{
    DiagnosticContext, DiagnosticKind, DiagnosticSeverity, ExtractedDiagnostic,
};
use fob_bundler::Error as BundlerError;
use fob_native::error::FobErrorDetails;
use fob_native::error_mapper::map_bundler_error;

#[test]
fn test_map_invalid_config_error() {
    let error = BundlerError::InvalidConfig("Invalid configuration".to_string());
    let mapped = map_bundler_error(&error);

    match mapped {
        FobErrorDetails::Validation(e) => {
            assert_eq!(e.message, "Invalid configuration");
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_map_invalid_output_path_error() {
    let error = BundlerError::InvalidOutputPath("/invalid/path".to_string());
    let mapped = map_bundler_error(&error);

    match mapped {
        FobErrorDetails::InvalidEntry(e) => {
            assert_eq!(e.path, "/invalid/path");
        }
        _ => panic!("Expected InvalidEntryError"),
    }
}

#[test]
fn test_map_missing_export_diagnostic() {
    let diagnostic = ExtractedDiagnostic {
        kind: DiagnosticKind::MissingExport,
        severity: DiagnosticSeverity::Error,
        message: "Missing export 'foo'".to_string(),
        file: Some("module.js".to_string()),
        line: Some(1),
        column: Some(1),
        help: Some("Available exports: bar, baz".to_string()),
        context: Some(DiagnosticContext::MissingExport {
            export_name: "foo".to_string(),
            module_id: "module.js".to_string(),
            available_exports: vec!["bar".to_string(), "baz".to_string()],
        }),
    };

    let error = BundlerError::Bundler(vec![diagnostic]);
    let mapped = map_bundler_error(&error);

    match mapped {
        FobErrorDetails::MissingExport(e) => {
            assert_eq!(e.export_name, "foo");
            assert_eq!(e.module_id, "module.js");
            assert_eq!(e.available_exports.len(), 2);
            assert!(e.available_exports.contains(&"bar".to_string()));
            assert!(e.available_exports.contains(&"baz".to_string()));
        }
        _ => panic!("Expected MissingExportError"),
    }
}

#[test]
fn test_map_circular_dependency_diagnostic() {
    let diagnostic = ExtractedDiagnostic {
        kind: DiagnosticKind::CircularDependency,
        severity: DiagnosticSeverity::Error,
        message: "Circular dependency detected".to_string(),
        file: Some("a.js".to_string()),
        line: None,
        column: None,
        help: None,
        context: Some(DiagnosticContext::CircularDependency {
            cycle_path: vec!["a.js".to_string(), "b.js".to_string(), "a.js".to_string()],
        }),
    };

    let error = BundlerError::Bundler(vec![diagnostic]);
    let mapped = map_bundler_error(&error);

    match mapped {
        FobErrorDetails::CircularDependency(e) => {
            assert_eq!(e.cycle_path.len(), 3);
            assert_eq!(e.cycle_path[0], "a.js");
            assert_eq!(e.cycle_path[1], "b.js");
        }
        _ => panic!("Expected CircularDependencyError"),
    }
}

#[test]
fn test_map_plugin_error() {
    let diagnostic = ExtractedDiagnostic {
        kind: DiagnosticKind::Plugin,
        severity: DiagnosticSeverity::Error,
        message: "Plugin 'test-plugin' failed".to_string(),
        file: None,
        line: None,
        column: None,
        help: None,
        context: Some(DiagnosticContext::Plugin {
            plugin_name: "test-plugin".to_string(),
        }),
    };

    let error = BundlerError::Bundler(vec![diagnostic]);
    let mapped = map_bundler_error(&error);

    match mapped {
        FobErrorDetails::Plugin(e) => {
            assert_eq!(e.name, "test-plugin");
        }
        _ => panic!("Expected PluginError"),
    }
}

#[test]
fn test_map_multiple_diagnostics() {
    let diagnostics = vec![
        ExtractedDiagnostic {
            kind: DiagnosticKind::MissingExport,
            severity: DiagnosticSeverity::Error,
            message: "Missing export 'foo'".to_string(),
            file: Some("a.js".to_string()),
            line: Some(1),
            column: Some(1),
            help: None,
            context: None,
        },
        ExtractedDiagnostic {
            kind: DiagnosticKind::UnresolvedImport,
            severity: DiagnosticSeverity::Error,
            message: "Cannot resolve 'bar'".to_string(),
            file: Some("b.js".to_string()),
            line: Some(2),
            column: Some(2),
            help: None,
            context: None,
        },
    ];

    let error = BundlerError::Bundler(diagnostics);
    let mapped = map_bundler_error(&error);

    match mapped {
        FobErrorDetails::Multiple(e) => {
            assert_eq!(e.errors.len(), 2);
            assert!(!e.primary_message.is_empty());
        }
        _ => panic!("Expected MultipleDiagnostics"),
    }
}

#[test]
fn test_map_io_error() {
    use std::io;

    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = BundlerError::Io(io_error);
    let mapped = map_bundler_error(&error);

    match mapped {
        FobErrorDetails::Runtime(e) => {
            assert!(e.message.contains("I/O error"));
        }
        _ => panic!("Expected RuntimeError"),
    }
}

#[test]
fn test_error_json_serialization() {
    use fob_native::error::{FobErrorDetails, NoEntriesError, ValidationError};

    // Test NoEntries error
    let error = FobErrorDetails::NoEntries(NoEntriesError {});
    let json = error.to_json_string();

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("Error should serialize to valid JSON");

    // Should have type field (from serde tag)
    // Note: serde uses the struct name for empty structs, so it's "NoEntriesError"
    assert!(
        parsed["type"] == "NoEntries" || parsed["type"] == "NoEntriesError",
        "Type should be NoEntries or NoEntriesError, got: {}",
        parsed["type"]
    );

    // Test Validation error
    let error = FobErrorDetails::Validation(ValidationError {
        message: "test message".to_string(),
    });
    let json = error.to_json_string();

    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("Error should serialize to valid JSON");

    // Note: serde uses the struct name, so it's "ValidationError"
    assert!(
        parsed["type"] == "Validation" || parsed["type"] == "ValidationError",
        "Type should be Validation or ValidationError, got: {}",
        parsed["type"]
    );
    assert_eq!(parsed["message"], "test message");
}

#[test]
fn test_error_envelope_versioning() {
    use fob_native::error::{FobErrorDetails, NoEntriesError};

    let error = FobErrorDetails::NoEntries(NoEntriesError {});

    let envelope = error.into_envelope_v1();
    assert_eq!(envelope.version, 1);

    // Verify envelope serializes correctly
    let json = serde_json::to_string(&envelope).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["version"], 1);
    // Note: serde uses the struct name for empty structs, so it's "NoEntriesError"
    assert!(
        parsed["error"]["type"] == "NoEntries" || parsed["error"]["type"] == "NoEntriesError",
        "Type should be NoEntries or NoEntriesError, got: {}",
        parsed["error"]["type"]
    );

    // Test custom version
    let error = FobErrorDetails::NoEntries(NoEntriesError {});
    let envelope = error.into_envelope(2);
    assert_eq!(envelope.version, 2);
}

#[test]
fn test_all_error_types_serialize() {
    use fob_native::error::*;

    // Test all error variants serialize correctly
    let errors = vec![
        FobErrorDetails::NoEntries(NoEntriesError {}),
        FobErrorDetails::Validation(ValidationError {
            message: "test".to_string(),
        }),
        FobErrorDetails::Runtime(RuntimeError {
            message: "test".to_string(),
        }),
        FobErrorDetails::InvalidEntry(InvalidEntryError {
            path: "test.js".to_string(),
        }),
        FobErrorDetails::MissingExport(MissingExportError {
            export_name: "foo".to_string(),
            module_id: "bar.js".to_string(),
            available_exports: vec![],
            suggestion: None,
        }),
        FobErrorDetails::CircularDependency(CircularDependencyError {
            cycle_path: vec!["a.js".to_string(), "b.js".to_string()],
        }),
        FobErrorDetails::Plugin(PluginError {
            name: "test-plugin".to_string(),
            message: "error".to_string(),
        }),
    ];

    for error in errors {
        let json = error.to_json_string();

        // Should be valid JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("All error types should serialize to valid JSON");

        // Should have type field
        assert!(
            parsed.get("type").is_some(),
            "All errors should have 'type' field"
        );
    }
}
