//! Fob bundler NAPI class

use crate::api::config::BundleConfig;
use crate::conversion::result::BundleResult;
use crate::core::bundler::CoreBundler;
use crate::error_mapper::map_bundler_error;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Fob bundler for Node.js
#[napi]
pub struct Fob {
    bundler: CoreBundler,
}

#[napi]
impl Fob {
    /// Create a new bundler instance
    #[napi(constructor)]
    pub fn new(config: BundleConfig) -> Result<Self> {
        let bundler = CoreBundler::new(config).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self { bundler })
    }

    /// Bundle the configured entries and return detailed bundle information
    #[napi]
    pub async fn bundle(&self) -> Result<BundleResult> {
        std::fs::write(
            "/tmp/fob-napi-debug.txt",
            format!(
                "NAPI bundle() called at {:?}\n",
                std::time::SystemTime::now()
            ),
        )
        .ok();

        let result = self.bundler.bundle().await.map_err(|e| {
            let details = map_bundler_error(&e);
            Error::from_reason(details.to_napi_json_string())
        })?;

        std::fs::write(
            "/tmp/fob-napi-debug.txt",
            format!(
                "NAPI bundle() completed at {:?}\n",
                std::time::SystemTime::now()
            ),
        )
        .ok();

        Ok(result)
    }
}
