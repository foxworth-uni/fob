# Rolldown Error Mapping

This document describes how Rolldown errors are mapped to structured `FobErrorDetails` in the Fob bundler.

## Overview

The Fob bundler extracts structured diagnostic information from Rolldown's error types and maps them to a stable, version-independent error format. This abstraction layer insulates the codebase from upstream Rolldown API changes.

## Architecture

### Diagnostic Extraction (`crates/fob-bundler/src/diagnostics.rs`)

The `ExtractedDiagnostic` struct contains all diagnostic information in a cloneable, serializable format:

- **kind**: The type of diagnostic (MissingExport, ParseError, etc.)
- **severity**: Error or Warning
- **message**: The error message
- **file**: Optional file path
- **line**: Optional line number
- **column**: Optional column number
- **help**: Optional help text

### Error Mapping (`crates/fob-native/src/error_mapper.rs`)

The error mapper converts `ExtractedDiagnostic` instances to `FobErrorDetails` variants:

- **Single diagnostic**: Mapped to the appropriate variant (MissingExport, Transform, etc.)
- **Multiple diagnostics**: Wrapped in `MultipleDiagnostics` variant

## EventKind → FobErrorDetails Mapping

| Rolldown EventKind | FobErrorDetails Variant   | Notes                                              |
| ------------------ | ------------------------- | -------------------------------------------------- |
| MissingExport      | `MissingExportError`      | Extracts export_name, module_id, available_exports |
| ParseError         | `TransformError`          | Contains diagnostics array with line/column info   |
| Transform          | `TransformError`          | Same as ParseError                                 |
| CircularDependency | `CircularDependencyError` | Extracts cycle_path array                          |
| UnresolvedEntry    | `InvalidEntryError`       | Maps to invalid_entry type                         |
| UnresolvedImport   | `RuntimeError`            | Generic runtime error with context                 |
| InvalidOption      | `RuntimeError`            | Generic runtime error with context                 |
| Plugin             | `RuntimeError`            | Generic runtime error with context                 |
| Other              | `RuntimeError`            | Fallback for unknown error types                   |

## Multiple Diagnostics

When Rolldown returns multiple diagnostics, they are wrapped in a `MultipleDiagnostics` variant:

```typescript
interface MultipleDiagnostics {
  type: 'multiple';
  errors: FobErrorDetails[];
  primary_message: string;
}
```

The `primary_message` is derived from the first diagnostic's kind and message.

## Extraction Strategy

Since Rolldown's `BuildDiagnostic` fields may be private, we extract information using:

1. **Public methods**: Use `diag.kind()`, `diag.severity()`, etc. when available
2. **Formatted output**: Parse structured information from `format!("{}", diag)` or `to_string()`
3. **Fallback parsing**: Extract fields using regex patterns from error messages

### Field Extraction Examples

**Missing Export**:

- Export name: Extracted from "export 'Name'" patterns or field extraction
- Module ID: Extracted from "requested_module" field or path extraction
- Available exports: Extracted from help text "Available: ..." patterns

**Transform Errors**:

- File path: Extracted from file extensions (.js, .ts, etc.)
- Line/Column: Extracted from ":line:column" or "line X, column Y" patterns
- Help text: Extracted from "help:", "Hint:", "Suggestion:" patterns

**Circular Dependencies**:

- Cycle path: Extracted from "A -> B -> C" patterns or message parsing

## Version Stability

The abstraction layer provides version stability by:

1. **Stable extraction API**: `ExtractedDiagnostic` struct doesn't depend on Rolldown internals
2. **Flexible parsing**: Can adapt to Rolldown message format changes
3. **Fallback handling**: Unknown error types map to `RuntimeError` with full message

## Rolldown Version Dependencies

Current implementation tested with:

- Rolldown v1.0.0-beta.50

When updating Rolldown versions:

1. Check for new `EventKind` variants
2. Update `DiagnosticKind` enum if needed
3. Add extraction logic for new error types
4. Update mapping table above

## Testing

Error mapping is tested in:

- `packages/fob-bundler/tests/error-handling.test.js` - Integration tests
- `packages/fob-bundler/tests/error-serialization.test.ts` - Serialization tests

## Future Improvements

1. **Direct API access**: When Rolldown exposes public diagnostic API, use it directly instead of parsing
2. **More structured extraction**: Improve field extraction for better accuracy
3. **Error suggestions**: Enhance suggestion generation for common errors
4. **Error grouping**: Group related errors (e.g., multiple missing exports from same module)
