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
