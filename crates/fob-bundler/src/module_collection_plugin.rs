use rolldown_plugin::{
    HookLoadArgs, HookLoadReturn, HookResolveIdArgs, HookResolveIdReturn, HookTransformArgs,
    HookTransformReturn, HookUsage, Plugin, PluginContext, TransformPluginContext,
};
use std::sync::Arc;

use crate::plugins::{FobPlugin, PluginPhase};
use fob_graph::collection::{CollectedModule, CollectionState, parse_module_structure};

/// Plugin that collects module information during the bundling process
///
/// Uses concurrent collections (DashMap/DashSet) internally for thread-safe access
/// during parallel bundling, eliminating the need for Mutex locks.
#[derive(Debug)]
pub struct ModuleCollectionPlugin {
    state: Arc<CollectionState>,
}

impl ModuleCollectionPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(CollectionState::new()),
        }
    }

    pub fn state(&self) -> Arc<CollectionState> {
        Arc::clone(&self.state)
    }

    /// Extract the collected data.
    ///
    /// Creates a new CollectionState with cloned data from the concurrent collections.
    /// The original state remains intact (since it's behind an Arc).
    pub fn take_data(&self) -> CollectionState {
        // Clone the data from DashMap/DashSet into a new CollectionState
        // This preserves the concurrent collections for potential future use
        let new_state = CollectionState::new();

        // Clone modules
        for entry in self.state.modules.iter() {
            new_state
                .modules
                .insert(entry.key().clone(), entry.value().clone());
        }

        // Clone entry points
        for entry in self.state.entry_points.iter() {
            new_state.entry_points.insert(entry.key().clone());
        }

        // Clone resolved entry IDs
        for entry in self.state.resolved_entry_ids.iter() {
            new_state.resolved_entry_ids.insert(entry.key().clone());
        }

        new_state
    }
}

impl Plugin for ModuleCollectionPlugin {
    fn name(&self) -> std::borrow::Cow<'static, str> {
        "module-collection-plugin".into()
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId | HookUsage::Load | HookUsage::Transform
    }

    fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs,
    ) -> impl std::future::Future<Output = HookResolveIdReturn> + Send {
        let state = Arc::clone(&self.state);
        let specifier = args.specifier.to_string();
        let is_entry = args.importer.is_none();

        async move {
            // Track entry specifiers - we'll match resolved IDs in load hook
            if is_entry {
                state.mark_entry(specifier);
            }

            // Let Rolldown handle the actual resolution
            Ok(None)
        }
    }

    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        let state = Arc::clone(&self.state);
        let id = args.id.to_string();

        async move {
            // Check if this resolved ID corresponds to an entry point
            let is_entry = state.entry_points.iter().any(|spec| {
                id.ends_with(spec.key()) || id.contains(spec.key()) || spec.key() == &id
            });

            if is_entry {
                state.resolved_entry_ids.insert(id);
            }

            // Don't modify load behavior
            Ok(None)
        }
    }

    fn transform(
        &self,
        _ctx: Arc<TransformPluginContext>,
        args: &HookTransformArgs,
    ) -> impl std::future::Future<Output = HookTransformReturn> + Send {
        let state = Arc::clone(&self.state);
        let code = args.code.to_string();
        let id = args.id.to_string();

        async move {
            // Parse the module to extract imports/exports
            // If parsing fails, treat as having side effects and no imports/exports
            let (imports, exports, has_side_effects) =
                parse_module_structure(&code).unwrap_or_else(|_| (vec![], vec![], true));

            let is_entry = state.resolved_entry_ids.contains(&id);

            let module = CollectedModule {
                id: id.clone(),
                code: Some(code),
                is_entry,
                is_external: false, // External modules won't go through transform
                imports,
                exports,
                has_side_effects,
            };

            state.add_module(id, module);

            // Don't modify the code
            Ok(None)
        }
    }
}

impl FobPlugin for ModuleCollectionPlugin {
    fn phase(&self) -> PluginPhase {
        PluginPhase::PostProcess
    }
}

impl Default for ModuleCollectionPlugin {
    fn default() -> Self {
        Self::new()
    }
}
