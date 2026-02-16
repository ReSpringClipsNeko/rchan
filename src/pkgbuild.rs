use anyhow::{Context, Result};
use regex::Regex;

/// PKGBUILD 中提取的版本信息
#[derive(Debug, Clone, PartialEq)]
pub struct PkgVersion {
    pub pkgver: String,
    pub pkgrel: String,
}

impl std::fmt::Display for PkgVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.pkgver, self.pkgrel)
    }
}

/// 从 PKGBUILD 文本内容中提取 pkgver 和 pkgrel
///
/// 按 Arch Linux 官方规范，格式为无引号直接赋值：
///   pkgver=1.02.3
///   pkgrel=1
pub fn parse_pkgbuild(content: &str) -> Result<PkgVersion> {
    let ver_re = Regex::new(r"(?m)^pkgver=([0-9][0-9.]*)")?;
    let rel_re = Regex::new(r"(?m)^pkgrel=([0-9]+)")?;

    let pkgver = ver_re
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .context("Failed to find pkgver in PKGBUILD")?;

    let pkgrel = rel_re
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .context("Failed to find pkgrel in PKGBUILD")?;

    Ok(PkgVersion { pkgver, pkgrel })
}

/// 从本地文件解析 PKGBUILD
pub fn parse_local(path: &std::path::Path) -> Result<PkgVersion> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read PKGBUILD: {}", path.display()))?;
    parse_pkgbuild(&content)
}

/// 从远程 URL 获取并解析 PKGBUILD
pub fn parse_remote(url: &str) -> Result<PkgVersion> {
    let content = reqwest::blocking::get(url)
        .with_context(|| format!("Failed to fetch remote PKGBUILD: {url}"))?
        .error_for_status()
        .with_context(|| format!("HTTP error fetching: {url}"))?
        .text()
        .context("Failed to read response body")?;
    parse_pkgbuild(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pkgbuild() {
        let content = "\
pkgname=example
pkgver=1.2.3
pkgrel=2
pkgdesc=\"An example package\"
";
        let ver = parse_pkgbuild(content).unwrap();
        assert_eq!(ver.pkgver, "1.2.3");
        assert_eq!(ver.pkgrel, "2");
        assert_eq!(ver.to_string(), "1.2.3-2");
    }

    #[test]
    fn test_parse_pkgbuild_long_version() {
        let content = "pkgver=1.02.3.4\npkgrel=10\n";
        let ver = parse_pkgbuild(content).unwrap();
        assert_eq!(ver.pkgver, "1.02.3.4");
        assert_eq!(ver.pkgrel, "10");
    }

    #[test]
    fn test_parse_pkgbuild_single_digit() {
        let content = "pkgver=3\npkgrel=1\n";
        let ver = parse_pkgbuild(content).unwrap();
        assert_eq!(ver.pkgver, "3");
        assert_eq!(ver.pkgrel, "1");
    }

    #[test]
    fn test_parse_pkgbuild_missing_pkgver() {
        let content = "pkgrel=1\n";
        assert!(parse_pkgbuild(content).is_err());
    }

    #[test]
    fn test_parse_pkgbuild_missing_pkgrel() {
        let content = "pkgver=1.0.0\n";
        assert!(parse_pkgbuild(content).is_err());
    }
}
