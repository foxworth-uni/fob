//! Plugin registry with execution phases.
//!
//! This module provides a plugin registry that organizes plugins by execution phase,
//! ensuring plugins run in the correct order during the bundling process.

use crate::SharedPluginable;
use rolldown_plugin::Plugin;
use std::sync::Arc;

/// Plugin execution phases
///
/// Plugins are executed in phase order (lower numbers first).
/// This ensures that virtual file resolution happens before module resolution,
/// which happens before transformation, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PluginPhase {
    /// Virtual file resolution (always first)
    ///
    /// Plugins that provide virtual files that don't exist on disk.
    /// Must run before any other plugins try to resolve or load files.
    Virtual = 0,

    /// Module resolution
    ///
    /// Plugins that modify or extend module resolution behavior.
    /// Runs after virtual files are available but before transformation.
    #[allow(dead_code)]
    Resolve = 10,

    /// Content transformation (MDX, CSS, etc.)
    ///
    /// Plugins that transform file contents (e.g., MDX to JSX, CSS processing).
    /// Runs after resolution but before asset processing.
    Transform = 20,

    /// Asset processing
    ///
    /// Plugins that detect and process assets (images, fonts, etc.).
    /// Runs after transformation.
    Assets = 30,

    /// Post-processing (module collection)
    ///
    /// Plugins that analyze or collect module data after bundling.
    /// Runs last, after all other plugins.
    PostProcess = 100,
}

/// Trait for Fob plugins that specify their execution phase
///
/// This is a marker trait that extends `Plugin` with phase information.
/// All plugins that implement `Plugin` can be used, but implementing `FobPlugin`
/// allows specifying the execution phase for better ordering.
///
/// Note: `Plugin` already requires `Send + Sync`, so we don't repeat those bounds here.
pub(crate) trait FobPlugin: Plugin {
    /// Return the execution phase for this plugin
    ///
    /// Defaults to `Transform` for backward compatibility.
    fn phase(&self) -> PluginPhase {
        PluginPhase::Transform
    }
}

/// Plugin registry that maintains plugins in phase order
pub(crate) struct PluginRegistry {
    plugins: Vec<(PluginPhase, SharedPluginable)>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Add a plugin to the registry
    ///
    /// The plugin will be inserted based on its phase. Sorting happens once
    /// when converting to Rolldown plugins via `into_rolldown_plugins()`.
    pub fn add<P: FobPlugin + 'static>(&mut self, plugin: P) {
        let phase = plugin.phase();
        let plugin_arc: SharedPluginable = Arc::new(plugin);
        self.plugins.push((phase, plugin_arc));
    }

    /// Add a plugin with an explicit phase
    ///
    /// Useful for plugins that don't implement FobPlugin or when you want
    /// to override the default phase.
    pub fn add_with_phase(&mut self, plugin: SharedPluginable, phase: PluginPhase) {
        self.plugins.push((phase, plugin));
    }

    /// Convert to Rolldown plugins in correct order
    ///
    /// Returns a vector of `SharedPluginable` sorted by phase.
    /// Sorting happens here (once) rather than on every add() for O(n log n) total
    /// instead of O(k × n log n) when adding k plugins.
    pub fn into_rolldown_plugins(mut self) -> Vec<SharedPluginable> {
        self.plugins.sort_by_key(|(phase, _)| *phase);
        self.plugins.into_iter().map(|(_, plugin)| plugin).collect()
    }

    /// Get the number of plugins in the registry
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if the registry is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
