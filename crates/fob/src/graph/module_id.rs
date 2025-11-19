use std::borrow::Cow;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use path_clean::PathClean;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

const VIRTUAL_PREFIX: &str = "virtual:";

/// Canonical identifier for a module in the Joy graph.
///
/// The identifier prefers canonical filesystem paths so we can safely compare modules
/// originating from different user inputs (relative vs absolute, `.` vs `..`, etc.).
/// When Rolldown emits virtual modules (e.g. `virtual:entry`), we retain the virtual
/// prefix and skip canonicalisation altogether.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId(PathBuf);

impl ModuleId {
    /// Create a new module identifier from a filesystem path.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ModuleIdError> {
        let path = path.as_ref();

        if path.as_os_str().is_empty() {
            return Err(ModuleIdError::EmptyPath);
        }

        if looks_like_virtual(path) {
            return Ok(Self(normalize_virtual(path)));
        }

        let joined = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|source| ModuleIdError::CurrentDir { source })?
                .join(path)
        };

        let cleaned = joined.clean();

        // On WASM, canonicalize is not available, so we just use the cleaned path
        #[cfg(target_family = "wasm")]
        {
            Ok(Self(cleaned))
        }

        // On native platforms, try to canonicalize
        #[cfg(not(target_family = "wasm"))]
        {
            match std::fs::canonicalize(&cleaned) {
                Ok(canonical) => Ok(Self(canonical)),
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self(cleaned)),
                Err(err) => Err(ModuleIdError::Canonicalization {
                    path: cleaned,
                    source: err,
                }),
            }
        }
    }

    /// Create a module identifier for a Rolldown virtual module (e.g. `virtual:...`).
    pub fn new_virtual(id: impl Into<String>) -> Self {
        let id = id.into();

        if id.is_empty() {
            return Self(PathBuf::from(VIRTUAL_PREFIX));
        }

        let normalized = if id.starts_with(VIRTUAL_PREFIX) {
            id
        } else {
            format!("{VIRTUAL_PREFIX}{id}")
        };

        Self(PathBuf::from(normalized))
    }

    /// Returns the underlying path representation.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Consume the identifier and return the owned path.
    pub fn into_path(self) -> PathBuf {
        self.0
    }

    /// Returns `true` if the identifier represents a virtual module.
    pub fn is_virtual(&self) -> bool {
        let text = self.path_string();
        text.starts_with(VIRTUAL_PREFIX) || text.starts_with("rolldown:") || text.starts_with('\0')
    }

    /// Borrow the identifier as a string for logging/serialization.
    pub fn path_string(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }

    #[cfg(test)]
    pub(crate) fn from_canonical_path(path: PathBuf) -> Self {
        Self(path)
    }

    fn from_serialized_path(path: PathBuf) -> Self {
        Self(path)
    }

    #[cfg(feature = "storage")]
    pub fn from_rolldown(id: &rolldown_common::ModuleId) -> Result<Self, ModuleIdError> {
        let raw = id.as_ref();
        if raw.starts_with('\0') || raw.starts_with("rolldown:") {
            return Ok(Self::new_virtual(raw.to_string()));
        }
        Self::new(raw)
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path_string())
    }
}

impl Serialize for ModuleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.path_string())
    }
}

impl<'de> Deserialize<'de> for ModuleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        if value.starts_with(VIRTUAL_PREFIX) {
            Ok(ModuleId::new_virtual(value))
        } else {
            Ok(ModuleId::from_serialized_path(PathBuf::from(value)))
        }
    }
}

/// Error type for `ModuleId` construction failures.
#[derive(Debug, Error)]
pub enum ModuleIdError {
    /// The provided path was empty.
    #[error("module id path is empty")]
    EmptyPath,

    /// Failed to resolve the current working directory for canonicalisation.
    #[error("failed to resolve current directory: {source}")]
    CurrentDir {
        #[source]
        source: io::Error,
    },

    /// Canonicalisation failed for reasons other than `NotFound`.
    #[error("failed to canonicalize path '{path}': {source}")]
    Canonicalization {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

fn looks_like_virtual(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.starts_with(VIRTUAL_PREFIX) || text.starts_with("rolldown:") || text.starts_with('\0')
}

fn normalize_virtual(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if text.starts_with(VIRTUAL_PREFIX) || text.starts_with("rolldown:") {
        PathBuf::from(text.into_owned())
    } else if text.starts_with('\0') {
        let trimmed = text.trim_start_matches('\0');
        PathBuf::from(format!("{VIRTUAL_PREFIX}{trimmed}"))
    } else {
        PathBuf::from(format!("{VIRTUAL_PREFIX}{text}"))
    }
}
