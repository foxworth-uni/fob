//! Implementation of GraphQueries trait for GraphStorage.

use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use serde_json::json;

use crate::graph::{ExternalDependency, Import, Export, Module, ModuleId, SourceType};
use crate::graph::storage::{GraphQueries, StorageError};
use super::{GraphStorage, queries::{ModuleRecord, EntryPointRecord, ExternalDepRecord}};

impl GraphQueries for GraphStorage {
    fn db(&self) -> &Surreal<Any> {
        &self.db
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }

    fn database(&self) -> &str {
        &self.database
    }

    async fn ensure_context(&self) -> std::result::Result<(), StorageError> {
        self.db()
            .use_ns(self.namespace())
            .use_db(self.database())
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;
        Ok(())
    }

    async fn store_module(&self, module: &Module) -> std::result::Result<(), StorageError> {
        self.ensure_context().await?;

        let id: String = module.id.path_string().to_string();
        let path_str: String = module.path.to_string_lossy().to_string();
        let source_type: String = format!("{:?}", module.source_type);

        // Serialize imports, exports, and symbol_table
        let imports_json: String = serde_json::to_string(&module.imports)
            .map_err(|e| StorageError::Query(format!("Failed to serialize imports: {e}")))?;
        let exports_json: String = serde_json::to_string(&module.exports)
            .map_err(|e| StorageError::Query(format!("Failed to serialize exports: {e}")))?;
        let symbol_table_json: String = serde_json::to_string(&module.symbol_table)
            .map_err(|e| StorageError::Query(format!("Failed to serialize symbol_table: {e}")))?;

        let query = r#"
            UPDATE module SET
                id = $id,
                path = $path,
                source_type = $source_type,
                imports = $imports,
                exports = $exports,
                has_side_effects = $has_side_effects,
                is_entry = $is_entry,
                is_external = $is_external,
                original_size = $original_size,
                bundled_size = $bundled_size,
                symbol_table = $symbol_table
            WHERE id = $id;
            INSERT INTO module {
                id: $id,
                path: $path,
                source_type: $source_type,
                imports: $imports,
                exports: $exports,
                has_side_effects: $has_side_effects,
                is_entry: $is_entry,
                is_external: $is_external,
                original_size: $original_size,
                bundled_size: $bundled_size,
                symbol_table: $symbol_table
            };
        "#;

        let params = json!({
            "id": id,
            "path": path_str,
            "source_type": source_type,
            "imports": imports_json,
            "exports": exports_json,
            "has_side_effects": module.has_side_effects,
            "is_entry": module.is_entry,
            "is_external": module.is_external,
            "original_size": module.original_size as i64,
            "bundled_size": module.bundled_size.map(|n| n as i64),
            "symbol_table": symbol_table_json
        });
        
        self.db()
            .query(query)
            .bind(params)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }

    async fn get_module(&self, id: &ModuleId) -> std::result::Result<Option<Module>, StorageError> {
        self.ensure_context().await?;

        let id_str: String = id.path_string().to_string();
        let query = "SELECT * FROM module WHERE id = $id LIMIT 1";

        let params = json!({
            "id": id_str
        });

        let mut response = self
            .db()
            .query(query)
            .bind(params)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let module: Option<ModuleRecord> = response
            .take(0)
            .map_err(|e| StorageError::Query(e.to_string()))?;

        match module {
            Some(record) => {
                let module = self.module_from_record(record)?;
                Ok(Some(module))
            }
            None => Ok(None),
        }
    }

    async fn get_all_modules(&self) -> std::result::Result<Vec<Module>, StorageError> {
        self.ensure_context().await?;

        let query = "SELECT * FROM module";
        let mut response = self
            .db()
            .query(query)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let records: Vec<ModuleRecord> = response
            .take(0)
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let mut modules = Vec::new();
        for record in records {
            modules.push(self.module_from_record(record)?);
        }

        Ok(modules)
    }

    async fn add_dependency(&self, from: &ModuleId, to: &ModuleId) -> std::result::Result<(), StorageError> {
        self.ensure_context().await?;

        let from_str: String = from.path_string().to_string();
        let to_str: String = to.path_string().to_string();

        let query = r#"
            RELATE module:$from -> depends_on -> module:$to;
        "#;

        let params = json!({
            "from": from_str,
            "to": to_str
        });

        self.db()
            .query(query)
            .bind(params)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }

    async fn get_dependencies(&self, id: &ModuleId) -> std::result::Result<Vec<ModuleId>, StorageError> {
        self.ensure_context().await?;

        let id_str: String = id.path_string().to_string();
        let query = r#"
            SELECT VALUE ->depends_on->module.id FROM module WHERE id = $id;
        "#;

        let params = json!({
            "id": id_str
        });

        let mut response = self
            .db()
            .query(query)
            .bind(params)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let ids: Vec<String> = response
            .take(0)
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let mut module_ids = Vec::new();
        for id_str in ids {
            if let Ok(module_id) = ModuleId::new(&id_str) {
                module_ids.push(module_id);
            } else if id_str.starts_with("virtual:") {
                module_ids.push(ModuleId::new_virtual(id_str));
            }
        }

        Ok(module_ids)
    }

    async fn get_dependents(&self, id: &ModuleId) -> std::result::Result<Vec<ModuleId>, StorageError> {
        self.ensure_context().await?;

        let id_str: String = id.path_string().to_string();
        let query = r#"
            SELECT VALUE <-depends_on<-module.id FROM module WHERE id = $id;
        "#;

        let params = json!({
            "id": id_str
        });

        let mut response = self
            .db()
            .query(query)
            .bind(params)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let ids: Vec<String> = response
            .take(0)
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let mut module_ids = Vec::new();
        for id_str in ids {
            if let Ok(module_id) = ModuleId::new(&id_str) {
                module_ids.push(module_id);
            } else if id_str.starts_with("virtual:") {
                module_ids.push(ModuleId::new_virtual(id_str));
            }
        }

        Ok(module_ids)
    }

    async fn get_entry_points(&self) -> std::result::Result<Vec<ModuleId>, StorageError> {
        self.ensure_context().await?;

        let query = "SELECT id FROM module WHERE is_entry = true";
        let mut response = self
            .db()
            .query(query)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let records: Vec<EntryPointRecord> = response
            .take(0)
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let mut entry_points = Vec::new();
        for record in records {
            if let Ok(module_id) = ModuleId::new(&record.id) {
                entry_points.push(module_id);
            } else if record.id.starts_with("virtual:") {
                entry_points.push(ModuleId::new_virtual(record.id));
            }
        }

        Ok(entry_points)
    }

    async fn store_external_dependency(&self, dep: &ExternalDependency) -> std::result::Result<(), StorageError> {
        self.ensure_context().await?;

        let specifier: String = dep.specifier.clone();
        let importers: Vec<String> = dep
            .importers
            .iter()
            .map(|id| id.path_string().to_string())
            .collect();

        let query = r#"
            UPDATE external_dep SET importers = $importers WHERE specifier = $specifier;
            INSERT INTO external_dep {
                specifier: $specifier,
                importers: $importers
            };
        "#;

        let params = json!({
            "specifier": specifier,
            "importers": importers
        });

        self.db()
            .query(query)
            .bind(params)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        Ok(())
    }

    async fn get_external_dependencies(&self) -> std::result::Result<Vec<ExternalDependency>, StorageError> {
        self.ensure_context().await?;

        let query = "SELECT * FROM external_dep";
        let mut response = self
            .db()
            .query(query)
            .await
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let records: Vec<ExternalDepRecord> = response
            .take(0)
            .map_err(|e| StorageError::Query(e.to_string()))?;

        let mut deps = Vec::new();
        for record in records {
            let mut dep = ExternalDependency::new(&record.specifier);
            for importer_str in record.importers {
                if let Ok(module_id) = ModuleId::new(&importer_str) {
                    dep.push_importer(module_id);
                } else if importer_str.starts_with("virtual:") {
                    dep.push_importer(ModuleId::new_virtual(importer_str));
                }
            }
            deps.push(dep);
        }

        Ok(deps)
    }

    async fn clear_all(&self) -> std::result::Result<(), StorageError> {
        self.ensure_context().await?;

        let queries = vec![
            "DELETE module",
            "DELETE depends_on",
            "DELETE external_dep",
        ];

        for query in queries {
            self.db()
                .query(query)
                .await
                .map_err(|e| StorageError::Query(e.to_string()))?;
        }

        Ok(())
    }

    fn module_from_record(&self, record: ModuleRecord) -> std::result::Result<Module, StorageError> {
        // Parse source type
        let source_type = match record.source_type.as_str() {
            "JavaScript" => SourceType::JavaScript,
            "TypeScript" => SourceType::TypeScript,
            "Jsx" => SourceType::Jsx,
            "Tsx" => SourceType::Tsx,
            "Json" => SourceType::Json,
            "Css" => SourceType::Css,
            _ => SourceType::Unknown,
        };

        // Deserialize imports, exports, and symbol_table
        let imports: Vec<Import> = serde_json::from_str(&record.imports)
            .map_err(|e| StorageError::Query(format!("Failed to deserialize imports: {e}")))?;
        let exports: Vec<Export> = serde_json::from_str(&record.exports)
            .map_err(|e| StorageError::Query(format!("Failed to deserialize exports: {e}")))?;
        let symbol_table = serde_json::from_str(&record.symbol_table)
            .map_err(|e| StorageError::Query(format!("Failed to deserialize symbol_table: {e}")))?;

        // Parse ModuleId
        let module_id = if record.id.starts_with("virtual:") {
            ModuleId::new_virtual(record.id)
        } else {
            ModuleId::new(&record.id)
                .map_err(|e| StorageError::Query(format!("Invalid module ID: {e}")))?
        };

        let path = std::path::PathBuf::from(record.path);

        Ok(Module {
            id: module_id,
            path,
            source_type,
            imports,
            exports,
            has_side_effects: record.has_side_effects,
            is_entry: record.is_entry,
            is_external: record.is_external,
            original_size: record.original_size as usize,
            bundled_size: record.bundled_size.map(|n| n as usize),
            symbol_table,
        })
    }
}


