//! Tests for the Query API

use fob_gen::{ExportDeclaration, ParseOptions, QueryBuilder, parse};
use oxc_allocator::Allocator;

#[test]
fn test_find_imports_iter() {
    let allocator = Allocator::default();
    let code = "
        import defaultExport from 'module-a';
        import * as name from 'module-b';
        import { export1, export2 as alias2 } from 'module-c';
    ";
    let parsed = parse(&allocator, code, ParseOptions::default()).unwrap();
    let query = QueryBuilder::new(&allocator, parsed.ast());

    let imports: Vec<_> = query.find_imports(None).iter().collect();
    assert_eq!(imports.len(), 3);

    assert_eq!(imports[0].source.value, "module-a");
    assert_eq!(imports[1].source.value, "module-b");
    assert_eq!(imports[2].source.value, "module-c");

    let specific_imports: Vec<_> = query.find_imports(Some("module-b")).iter().collect();
    assert_eq!(specific_imports.len(), 1);
    assert_eq!(specific_imports[0].source.value, "module-b");
}

#[test]
fn test_find_exports_iter() {
    let allocator = Allocator::default();
    let code = "
        export const a = 1;
        export default function() {};
        export * from 'module-a';
        export { b, c as d } from 'module-b';
    ";
    let parsed = parse(&allocator, code, ParseOptions::default()).unwrap();
    let query = QueryBuilder::new(&allocator, parsed.ast());

    let exports: Vec<_> = query.find_exports().iter().collect();
    assert_eq!(exports.len(), 4);

    let mut named_count = 0;
    let mut default_count = 0;
    let mut all_count = 0;

    for export in exports {
        match export {
            ExportDeclaration::Named(decl) => {
                named_count += 1;
                if let Some(source) = &decl.source {
                    assert_eq!(source.value, "module-b");
                }
            }
            ExportDeclaration::Default(_) => default_count += 1,
            ExportDeclaration::All(decl) => {
                all_count += 1;
                assert_eq!(decl.source.value, "module-a");
            }
        }
    }

    assert_eq!(named_count, 2, "Expected two ExportNamedDeclarations");
    assert_eq!(default_count, 1, "Expected one ExportDefaultDeclaration");
    assert_eq!(all_count, 1, "Expected one ExportAllDeclaration");
}
