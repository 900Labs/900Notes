use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub db_path: String,
    pub created_at: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRegistry {
    pub workspaces: Vec<Workspace>,
    pub active_id: String,
}

pub struct WorkspaceService {
    registry_path: PathBuf,
}

impl WorkspaceService {
    pub fn new(app_data_dir: &Path) -> Self {
        let registry_path = app_data_dir.join("workspaces.json");
        WorkspaceService { registry_path }
    }

    pub fn load_registry(&self) -> Result<WorkspaceRegistry, String> {
        if !self.registry_path.exists() {
            let default = WorkspaceRegistry {
                workspaces: vec![Workspace {
                    id: "default".to_string(),
                    name: "Default Workspace".to_string(),
                    db_path: "900notes.db".to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    is_default: true,
                }],
                active_id: "default".to_string(),
            };
            self.save_registry(&default)?;
            return Ok(default);
        }

        let content = std::fs::read_to_string(&self.registry_path)
            .map_err(|e| format!("Read registry: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("Parse registry: {e}"))
    }

    pub fn save_registry(&self, registry: &WorkspaceRegistry) -> Result<(), String> {
        let content = serde_json::to_string_pretty(registry)
            .map_err(|e| format!("Serialize registry: {e}"))?;
        let temp = self.registry_path.with_extension("json.tmp");
        let backup = self.registry_path.with_extension("json.previous");
        std::fs::write(&temp, content).map_err(|e| format!("Write registry: {e}"))?;
        if backup.exists() {
            let _ = std::fs::remove_file(&backup);
        }
        if self.registry_path.exists() {
            std::fs::rename(&self.registry_path, &backup)
                .map_err(|e| format!("Stage registry replacement: {e}"))?;
        }
        if let Err(error) = std::fs::rename(&temp, &self.registry_path) {
            if backup.exists() {
                let _ = std::fs::rename(&backup, &self.registry_path);
            }
            return Err(format!("Replace registry: {error}"));
        }
        let _ = std::fs::remove_file(backup);
        Ok(())
    }

    pub fn active_workspace(&self) -> Result<Workspace, String> {
        let registry = self.load_registry()?;
        registry
            .workspaces
            .into_iter()
            .find(|workspace| workspace.id == registry.active_id)
            .ok_or_else(|| "Active workspace is missing from the registry".to_string())
    }

    pub fn workspace_db_path(&self, workspace: &Workspace) -> PathBuf {
        self.registry_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(&workspace.db_path)
    }

    pub fn create_workspace(&self, name: &str) -> Result<Workspace, String> {
        let mut registry = self.load_registry()?;
        let id = uuid::Uuid::new_v4().to_string();
        let db_filename = format!("{}.db", id);
        let workspace = Workspace {
            id: id.clone(),
            name: name.to_string(),
            db_path: db_filename,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_default: false,
        };
        registry.workspaces.push(workspace.clone());
        self.save_registry(&registry)?;
        Ok(workspace)
    }

    pub fn delete_workspace(&self, id: &str) -> Result<(), String> {
        let mut registry = self.load_registry()?;
        if id == "default" {
            return Err("Cannot delete default workspace".to_string());
        }
        if registry.active_id == id {
            return Err("Cannot delete active workspace".to_string());
        }
        let workspace = registry
            .workspaces
            .iter()
            .find(|w| w.id == id)
            .ok_or("Workspace not found")?;

        let registry_dir = self.registry_path.parent().unwrap_or(Path::new("."));
        let db_path = registry_dir.join(&workspace.db_path);
        if db_path.exists() {
            std::fs::remove_file(&db_path).map_err(|e| format!("Delete workspace DB: {e}"))?;
        }
        for suffix in ["enc", "meta"] {
            let companion = PathBuf::from(format!("{}.{}", db_path.display(), suffix));
            if companion.exists() {
                std::fs::remove_file(&companion)
                    .map_err(|e| format!("Delete workspace encryption file: {e}"))?;
            }
        }

        registry.workspaces.retain(|w| w.id != id);
        self.save_registry(&registry)?;
        Ok(())
    }

    pub fn switch_workspace(&self, id: &str) -> Result<Workspace, String> {
        let mut registry = self.load_registry()?;
        let workspace = registry
            .workspaces
            .iter()
            .find(|w| w.id == id)
            .ok_or("Workspace not found")?
            .clone();
        registry.active_id = id.to_string();
        self.save_registry(&registry)?;
        Ok(workspace)
    }

    pub fn rename_workspace(&self, id: &str, name: &str) -> Result<Workspace, String> {
        let mut registry = self.load_registry()?;
        let workspace = registry
            .workspaces
            .iter_mut()
            .find(|w| w.id == id)
            .ok_or("Workspace not found")?;
        workspace.name = name.to_string();
        let result = workspace.clone();
        self.save_registry(&registry)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_workspace_survives_service_restart() {
        let root =
            std::env::temp_dir().join(format!("900notes-workspace-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let service = WorkspaceService::new(&root);
        let created = service.create_workspace("Research").unwrap();
        service.switch_workspace(&created.id).unwrap();

        let restarted = WorkspaceService::new(&root);
        assert_eq!(restarted.active_workspace().unwrap().id, created.id);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_registry_write_does_not_change_active_workspace() {
        let root =
            std::env::temp_dir().join(format!("900notes-workspace-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let service = WorkspaceService::new(&root);
        let created = service.create_workspace("Research").unwrap();
        std::fs::create_dir(root.join("workspaces.json.tmp")).unwrap();
        assert!(service.switch_workspace(&created.id).is_err());
        assert_eq!(service.active_workspace().unwrap().id, "default");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unknown_workspace_does_not_change_registry() {
        let root =
            std::env::temp_dir().join(format!("900notes-workspace-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let service = WorkspaceService::new(&root);
        assert!(service.switch_workspace("missing").is_err());
        assert_eq!(service.active_workspace().unwrap().id, "default");
        std::fs::remove_dir_all(root).unwrap();
    }
}
