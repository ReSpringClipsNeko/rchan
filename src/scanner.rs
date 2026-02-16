use std::path::Path;

use anyhow::Result;

use crate::config::RchanConfig;
use crate::pkgbuild;

/// 扫描结果枚举
pub enum ScanResult {
    /// 远程版本已更新
    Updated {
        name: String,
        local_ver: String,
        remote_ver: String,
    },
    /// 版本一致，无需更新
    UpToDate {
        name: String,
        local_ver: String,
    },
    /// 处理过程中出错
    Error {
        name: String,
        message: String,
    },
}

/// 扫描当前目录下所有子目录（深度一层）
/// 查找同时包含 rchan.yaml 和 PKGBUILD 的子目录
pub fn scan_directory(base: &Path) -> Result<Vec<ScanResult>> {
    let mut results = Vec::new();

    let entries = std::fs::read_dir(base)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // 只处理目录
        if !path.is_dir() {
            continue;
        }

        let rchan_yaml = path.join("rchan.yaml");
        let pkgbuild_path = path.join("PKGBUILD");

        // 跳过没有 rchan.yaml 或 PKGBUILD 的目录
        if !rchan_yaml.exists() || !pkgbuild_path.exists() {
            continue;
        }

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let result = check_package(&name, &rchan_yaml, &pkgbuild_path);
        results.push(result);
    }

    // 按名称排序，输出更整齐
    results.sort_by(|a, b| {
        let name_a = match a {
            ScanResult::Updated { name, .. } => name,
            ScanResult::UpToDate { name, .. } => name,
            ScanResult::Error { name, .. } => name,
        };
        let name_b = match b {
            ScanResult::Updated { name, .. } => name,
            ScanResult::UpToDate { name, .. } => name,
            ScanResult::Error { name, .. } => name,
        };
        name_a.cmp(name_b)
    });

    Ok(results)
}

/// 检查单个包：比较本地和远程 PKGBUILD 版本
fn check_package(name: &str, rchan_yaml: &Path, pkgbuild_path: &Path) -> ScanResult {
    let config = match RchanConfig::from_file(rchan_yaml) {
        Ok(c) => c,
        Err(e) => {
            return ScanResult::Error {
                name: name.to_string(),
                message: format!("Failed to parse rchan.yaml: {e}"),
            }
        }
    };

    let local_ver = match pkgbuild::parse_local(pkgbuild_path) {
        Ok(v) => v,
        Err(e) => {
            return ScanResult::Error {
                name: name.to_string(),
                message: format!("Failed to parse local PKGBUILD: {e}"),
            }
        }
    };

    let remote_ver = match pkgbuild::parse_remote(&config.remote_pkgbuild) {
        Ok(v) => v,
        Err(e) => {
            return ScanResult::Error {
                name: name.to_string(),
                message: format!("Failed to fetch remote PKGBUILD: {e}"),
            }
        }
    };

    if local_ver == remote_ver {
        ScanResult::UpToDate {
            name: name.to_string(),
            local_ver: local_ver.to_string(),
        }
    } else {
        ScanResult::Updated {
            name: name.to_string(),
            local_ver: local_ver.to_string(),
            remote_ver: remote_ver.to_string(),
        }
    }
}
