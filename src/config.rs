use serde::Deserialize;

/// rchan.yaml configuration file structure
#[derive(Debug, Deserialize)]
pub struct RchanConfig {
    /// URL of the remote PKGBUILD
    pub remote_pkgbuild: String,
}

impl RchanConfig {
    /// Read and parse rchan.yaml from a file path
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: RchanConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
