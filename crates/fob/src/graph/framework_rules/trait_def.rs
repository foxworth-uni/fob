//! Framework rule trait definition for marking framework-convention exports.

use async_trait::async_trait;
use crate::graph::ModuleGraph;
use crate::Result;

/// Framework-specific rule for marking exports as used by framework conventions.
///
/// Framework rules identify exports that appear unused but are actually consumed
/// by framework magic (React hooks, Next.js data fetching, Vue composables, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use fob::graph::{ModuleGraph, FrameworkRule};
/// use async_trait::async_trait;
///
/// struct MyFrameworkRule;
///
/// #[async_trait]
/// impl FrameworkRule for MyFrameworkRule {
///     async fn apply(&self, graph: &ModuleGraph) -> Result<()> {
///         let modules = graph.modules().await?;
///         
///         for module in modules {
///             let mut updated = module.clone();
///             let mut changed = false;
///             
///             for export in updated.exports.iter_mut() {
///                 if export.name.starts_with("action_") {
///                     export.mark_framework_used();
///                     changed = true;
///                 }
///             }
///             
///             if changed {
///                 graph.add_module(updated).await?;
///             }
///         }
///         
///         Ok(())
///     }
///
///     fn name(&self) -> &'static str {
///         "MyFrameworkRule"
///     }
///
///     fn description(&self) -> &'static str {
///         "Marks exports prefixed with 'action_' as framework-used"
///     }
/// }
/// ```
#[async_trait]
pub trait FrameworkRule: Send + Sync {
    /// Apply the rule to the module graph, marking exports as framework-used.
    ///
    /// This method receives immutable access to the graph and should:
    /// 1. Load modules via `graph.modules().await?`
    /// 2. Clone and modify modules that need changes
    /// 3. Save modified modules back via `graph.add_module(module).await?`
    ///
    /// The async nature allows the database backend to handle concurrent updates safely.
    async fn apply(&self, graph: &ModuleGraph) -> Result<()>;

    /// Human-readable name for the rule (used in diagnostics).
    fn name(&self) -> &'static str;

    /// Description of what patterns this rule matches.
    fn description(&self) -> &'static str;

    /// Whether this rule should be enabled by default.
    ///
    /// Built-in rules return `true`, custom rules typically return `false`.
    fn is_default(&self) -> bool {
        false
    }

    /// Clone the rule into a boxed trait object.
    ///
    /// Required for storing rules in collections.
    fn clone_box(&self) -> Box<dyn FrameworkRule>;
}

impl Clone for Box<dyn FrameworkRule> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
