//! Internal AST utility traits for fob-core
//!
//! These utilities are marked as internal and may change without notice.
//! They're intended for use within the fob ecosystem, not as public API.

#[cfg(feature = "fob_internal")]
mod internal_impl {
    use crate::JsBuilder;
    use oxc_ast::ast::*;

    /// Trait for manipulating imports in AST
    pub trait ImportManipulation<'a> {
        /// Rename an import from one name to another
        fn rename_import(&mut self, from: &str, to: &str);

        /// Ensure a default export exists with the given identifier
        fn ensure_default_export(&mut self, ident: &str);

        /// Add a side-effect import
        fn add_side_effect_import(&mut self, source: &str);
    }

    /// Trait for injecting development helpers
    pub trait DevInjection<'a> {
        /// Inject reload client script reference
        fn inject_reload_client(&mut self, entry_ident: &str);

        /// Wrap exports with development boundary
        fn wrap_with_dev_boundary(&mut self, builder: &JsBuilder<'a>);
    }

    /// Trait for common AST mutations
    pub trait AstMutations<'a> {
        /// Rename all references to an identifier
        fn rename_identifier(&mut self, old_name: &str, new_name: &str);

        /// Wrap a function call with error handling
        fn wrap_call_with_error_handling(
            &mut self,
            builder: &JsBuilder<'a>,
            call: &CallExpression<'a>,
        );
    }
}

#[cfg(feature = "fob_internal")]
pub use internal_impl::*;
