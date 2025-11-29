use rolldown_plugin::{
    HookLoadArgs, HookLoadReturn, HookResolveIdArgs, HookResolveIdReturn, HookTransformArgs,
    HookTransformReturn, HookUsage, Plugin, PluginContext, TransformPluginContext,
};
use std::sync::{Arc, Mutex};

use fob_graph::collection::{CollectedModule, CollectionState, parse_module_structure};

/// Plugin that collects module information during the bundling process
#[derive(Debug)]
pub struct ModuleCollectionPlugin {
    state: Arc<Mutex<CollectionState>>,
}

impl ModuleCollectionPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CollectionState::new())),
        }
    }

    pub fn state(&self) -> Arc<Mutex<CollectionState>> {
        Arc::clone(&self.state)
    }

    pub fn take_data(&self) -> CollectionState {
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut *state)
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
                let mut state = state.lock().unwrap();
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
            let is_entry = {
                let state = state.lock().unwrap();
                // Match against stored entry specifiers
                state
                    .entry_points
                    .iter()
                    .any(|spec| id.ends_with(spec) || id.contains(spec) || spec == &id)
            };

            if is_entry {
                let mut state = state.lock().unwrap();
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

            let is_entry = {
                let state = state.lock().unwrap();
                state.resolved_entry_ids.contains(&id)
            };

            let module = CollectedModule {
                id: id.clone(),
                code: Some(code),
                is_entry,
                is_external: false, // External modules won't go through transform
                imports,
                exports,
                has_side_effects,
            };

            let mut state = state.lock().unwrap();
            state.add_module(id, module);

            // Don't modify the code
            Ok(None)
        }
    }
}

impl Default for ModuleCollectionPlugin {
    fn default() -> Self {
        Self::new()
    }
}
