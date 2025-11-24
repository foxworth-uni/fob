use fob_graph::collection::{parse_module_structure, CollectedImportKind};

#[test]
fn test_regular_import() {
    let code = "import { foo, bar } from './module';";
    let (imports, _, _) = parse_module_structure(code).unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "./module");
    assert_eq!(imports[0].specifiers.len(), 2);
    assert_eq!(imports[0].kind, CollectedImportKind::Static);
}

#[test]
fn test_type_only_import() {
    let code = "import type { Type } from './types';";
    let (imports, _, _) = parse_module_structure(code).unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "./types");
    assert_eq!(imports[0].kind, CollectedImportKind::TypeOnly);
}

#[test]
fn test_side_effect_import() {
    let code = "import './polyfill';";
    let (imports, _, _) = parse_module_structure(code).unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "./polyfill");
    assert_eq!(imports[0].specifiers.len(), 0);
    assert_eq!(imports[0].kind, CollectedImportKind::Static);
}

#[test]
fn test_default_import() {
    let code = "import React from 'react';";
    let (imports, _, _) = parse_module_structure(code).unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "react");
    assert_eq!(imports[0].kind, CollectedImportKind::Static);
}

#[test]
fn test_namespace_import() {
    let code = "import * as utils from './utils';";
    let (imports, _, _) = parse_module_structure(code).unwrap();

    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "./utils");
    assert_eq!(imports[0].kind, CollectedImportKind::Static);
}

#[test]
fn test_mixed_imports() {
    let code = r#"
        import { value } from './value';
        import type { Type } from './types';
        import './side-effect';
    "#;
    let (imports, _, _) = parse_module_structure(code).unwrap();

    assert_eq!(imports.len(), 3);
    assert_eq!(imports[0].kind, CollectedImportKind::Static);
    assert_eq!(imports[1].kind, CollectedImportKind::TypeOnly);
    assert_eq!(imports[2].kind, CollectedImportKind::Static);
}
