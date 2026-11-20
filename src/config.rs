use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgConfig {
    pub version: String,
    pub install_dir: String,
    pub data_dir: String,
    pub port: u16,
    pub username: String,
    pub installed_at: String,
}

impl PgConfig {
    pub fn config_path(install_dir: &Path) -> PathBuf {
        install_dir.join("mini-dbs.json")
    }

    pub fn load(install_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path(install_dir);
        let content = fs::read_to_string(path)?;
        let config: PgConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, install_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path(install_dir);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct InstanceRegistry {
    pub instances: Vec<String>,
}

impl InstanceRegistry {
    pub fn registry_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("mini-dbs")
            .join("instances.json")
    }

    pub fn load() -> Self {
        let path = Self::registry_path();
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn register(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let canonical = dir.to_string_lossy().to_string();
        if !self.instances.iter().any(|i| i.eq_ignore_ascii_case(&canonical)) {
            self.instances.push(canonical);
            self.save()?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn unregister(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let canonical = dir.to_string_lossy().to_string();
        self.instances.retain(|i| !i.eq_ignore_ascii_case(&canonical));
        self.save()?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
