use serde::Deserialize;

/// rchan.yaml 配置文件结构
#[derive(Debug, Deserialize)]
pub struct RchanConfig {
    /// 远程 PKGBUILD 的 URL
    pub remote_pkgbuild: String,
}

impl RchanConfig {
    /// 从文件路径读取并解析 rchan.yaml
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: RchanConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
