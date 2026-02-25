use std::path::Path;

use anyhow::Result;

use crate::config::RchanConfig;
use crate::pkgbuild;

/// Scan result enum
pub enum ScanResult {
    /// Remote version has been updated
    Updated {
        name: String,
        local_ver: String,
        remote_ver: String,
    },
    /// Versions match, no update needed
    UpToDate {
        name: String,
        local_ver: String,
    },
    /// An error occurred during processing
    Error {
        name: String,
        message: String,
    },
}

/// Scan all subdirectories (one level deep) under the current directory
/// looking for those containing both rchan.yaml and PKGBUILD
pub fn scan_directory(base: &Path) -> Result<Vec<ScanResult>> {
    let mut results = Vec::new();

    let entries = std::fs::read_dir(base)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Only process directories
        if !path.is_dir() {
            continue;
        }

        let rchan_yaml = path.join("rchan.yaml");
        let pkgbuild_path = path.join("PKGBUILD");

        // Skip directories without rchan.yaml or PKGBUILD
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

    // Sort by name for cleaner output
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

/// Check a single package: compare local and remote PKGBUILD versions
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
