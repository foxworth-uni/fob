//! Query API for ergonomic AST traversal and transformation
//!
//! Provides a high-level API for finding and modifying AST nodes
//! without writing custom visitors.

#[cfg(feature = "query-api")]
mod query_impl {
    use oxc_allocator::Allocator;
    use oxc_ast::ast::*;
    use oxc_ast_visit::{Visit, walk};

    /// Query builder for finding AST nodes
    pub struct QueryBuilder<'a> {
        allocator: &'a Allocator,
        program: &'a Program<'a>,
    }

    impl<'a> QueryBuilder<'a> {
        /// Create a new query builder
        pub fn new(allocator: &'a Allocator, program: &'a Program<'a>) -> Self {
            Self { allocator, program }
        }

        /// Find all function calls
        pub fn find_calls(&self, callee_name: Option<&str>) -> CallQuery<'a> {
            CallQuery::new(self.allocator, self.program, callee_name)
        }

        /// Find all JSX elements
        pub fn find_jsx(&self, tag_name: Option<&str>) -> JsxQuery<'a> {
            JsxQuery::new(self.allocator, self.program, tag_name)
        }

        /// Find all imports
        pub fn find_imports(&self, source: Option<&str>) -> ImportQuery<'a> {
            ImportQuery::new(self.allocator, self.program, source)
        }

        /// Find all exports
        pub fn find_exports(&self) -> ExportQuery<'a> {
            ExportQuery::new(self.allocator, self.program)
        }
    }

    /// Query for function calls
    pub struct CallQuery<'a> {
        program: &'a Program<'a>,
        callee_name: Option<String>,
        matches: Vec<*const CallExpression<'a>>,
    }

    impl<'a> CallQuery<'a> {
        fn new(
            _allocator: &'a Allocator,
            program: &'a Program<'a>,
            callee_name: Option<&str>,
        ) -> Self {
            let mut query = Self {
                program,
                callee_name: callee_name.map(|s| s.to_string()),
                matches: Vec::new(),
            };
            query.collect();
            query
        }

        fn collect(&mut self) {
            struct CallCollector<'a> {
                callee_name: Option<String>,
                matches: Vec<*const CallExpression<'a>>,
            }

            impl<'a, 'ast> Visit<'ast> for CallCollector<'a>
            where
                'ast: 'a,
            {
                fn visit_call_expression(&mut self, call: &CallExpression<'ast>) {
                    if let Some(ref name) = self.callee_name {
                        if let Expression::Identifier(ident) = &call.callee {
                            if ident.name.as_str() == name {
                                self.matches.push(call as *const _);
                            }
                        }
                    } else {
                        self.matches.push(call as *const _);
                    }
                    walk::walk_call_expression(self, call);
                }
            }

            let mut collector = CallCollector {
                callee_name: self.callee_name.clone(),
                matches: Vec::new(),
            };
            walk::walk_program(&mut collector, self.program);
            self.matches = collector.matches;
        }

        /// Filter matches by a predicate
        pub fn filter<F>(mut self, predicate: F) -> Self
        where
            F: Fn(&CallExpression<'a>) -> bool,
        {
            self.matches.retain(|&ptr| {
                // SAFETY: These pointers are valid during the query lifetime
                unsafe { predicate(&*ptr) }
            });
            self
        }

        /// Get the number of matches
        pub fn count(&self) -> usize {
            self.matches.len()
        }
    }

    /// Query for JSX elements
    pub struct JsxQuery<'a> {
        program: &'a Program<'a>,
        tag_name: Option<String>,
        matches: Vec<*const JSXElement<'a>>,
    }

    impl<'a> JsxQuery<'a> {
        fn new(
            _allocator: &'a Allocator,
            program: &'a Program<'a>,
            tag_name: Option<&str>,
        ) -> Self {
            let mut query = Self {
                program,
                tag_name: tag_name.map(|s| s.to_string()),
                matches: Vec::new(),
            };
            query.collect();
            query
        }

        fn collect(&mut self) {
            struct JsxCollector<'a> {
                tag_name: Option<String>,
                matches: Vec<*const JSXElement<'a>>,
            }

            impl<'a, 'ast> Visit<'ast> for JsxCollector<'a>
            where
                'ast: 'a,
            {
                fn visit_jsx_element(&mut self, element: &JSXElement<'ast>) {
                    if let Some(ref name) = self.tag_name {
                        // Extract tag name from JSX element
                        // This is simplified - real implementation would handle namespaced names
                        if let JSXElementName::Identifier(ident) = &element.opening_element.name {
                            if ident.name.as_str() == name {
                                self.matches.push(element as *const _);
                            }
                        }
                    } else {
                        self.matches.push(element as *const _);
                    }
                    walk::walk_jsx_element(self, element);
                }
            }

            let mut collector = JsxCollector {
                tag_name: self.tag_name.clone(),
                matches: Vec::new(),
            };
            walk::walk_program(&mut collector, self.program);
            self.matches = collector.matches;
        }

        /// Filter matches by a predicate
        pub fn filter<F>(mut self, predicate: F) -> Self
        where
            F: Fn(&JSXElement<'a>) -> bool,
        {
            self.matches.retain(|&ptr| unsafe { predicate(&*ptr) });
            self
        }

        /// Get the number of matches
        pub fn count(&self) -> usize {
            self.matches.len()
        }
    }

    /// Query for imports
    pub struct ImportQuery<'a> {
        program: &'a Program<'a>,
        source: Option<String>,
        matches: Vec<*const ImportDeclaration<'a>>,
    }

    impl<'a> ImportQuery<'a> {
        fn new(_allocator: &'a Allocator, program: &'a Program<'a>, source: Option<&str>) -> Self {
            let mut query = Self {
                program,
                source: source.map(|s| s.to_string()),
                matches: Vec::new(),
            };
            query.collect();
            query
        }

        fn collect(&mut self) {
            struct ImportCollector<'a> {
                source: Option<String>,
                matches: Vec<*const ImportDeclaration<'a>>,
            }

            impl<'a, 'ast> Visit<'ast> for ImportCollector<'a>
            where
                'ast: 'a,
            {
                fn visit_import_declaration(&mut self, import: &ImportDeclaration<'ast>) {
                    if let Some(ref source) = self.source {
                        if import.source.value.as_str() == source {
                            self.matches.push(import as *const _);
                        }
                    } else {
                        self.matches.push(import as *const _);
                    }
                }
            }

            let mut collector = ImportCollector {
                source: self.source.clone(),
                matches: Vec::new(),
            };
            walk::walk_program(&mut collector, self.program);
            self.matches = collector.matches;
        }

        /// Get the number of matches
        pub fn count(&self) -> usize {
            self.matches.len()
        }
    }

    /// Query for exports
    pub struct ExportQuery<'a> {
        program: &'a Program<'a>,
        matches: Vec<*const ModuleDeclaration<'a>>,
    }

    impl<'a> ExportQuery<'a> {
        fn new(_allocator: &'a Allocator, program: &'a Program<'a>) -> Self {
            let mut query = Self {
                program,
                matches: Vec::new(),
            };
            query.collect();
            query
        }

        fn collect(&mut self) {
            struct ExportCollector<'a> {
                matches: Vec<*const ModuleDeclaration<'a>>,
            }

            impl<'a, 'ast> Visit<'ast> for ExportCollector<'a>
            where
                'ast: 'a,
            {
                fn visit_module_declaration(&mut self, decl: &ModuleDeclaration<'ast>) {
                    if matches!(
                        decl,
                        ModuleDeclaration::ExportNamedDeclaration(_)
                            | ModuleDeclaration::ExportDefaultDeclaration(_)
                    ) {
                        self.matches.push(decl as *const _);
                    }
                }
            }

            let mut collector = ExportCollector {
                matches: Vec::new(),
            };
            walk::walk_program(&mut collector, self.program);
            self.matches = collector.matches;
        }

        /// Get the number of matches
        pub fn count(&self) -> usize {
            self.matches.len()
        }
    }
}

#[cfg(feature = "query-api")]
pub use query_impl::*;
