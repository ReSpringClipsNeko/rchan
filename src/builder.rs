use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use colored::Colorize;

/// Run the build process: iterate over all subdirectories containing a PKGBUILD and build each one
pub fn run_build(base: &Path) -> Result<()> {
    let pkgs_dir = base.join("pkgs");
    let build_dir = base.join("build");

    // Create pkgs and build directories
    std::fs::create_dir_all(&pkgs_dir)
        .context("Failed to create pkgs directory")?;
    std::fs::create_dir_all(&build_dir)
        .context("Failed to create build directory")?;

    println!(
        "{} {}",
        "rchan build".bold().cyan(),
        "- PKGBUILD batch builder".dimmed()
    );
    println!("{} {}\n", "Working directory:".bold(), base.display());

    let mut entries: Vec<_> = std::fs::read_dir(base)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.is_dir()
                && path.join("PKGBUILD").exists()
                && path.file_name().map_or(true, |n| n != "pkgs" && n != "build")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!(
            "{}",
            "No subdirectories with PKGBUILD found.".yellow()
        );
        return Ok(());
    }

    let total = entries.len();
    let mut success_count = 0;
    let mut fail_count = 0;

    for (i, entry) in entries.iter().enumerate() {
        let pkg_src = entry.path();
        let name = pkg_src
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        println!(
            "[{}/{}] {} {}",
            i + 1,
            total,
            "Building".bold().blue(),
            name.white().bold()
        );

        // Clean build directory
        clean_dir(&build_dir)?;

        // Copy all contents from source directory to build directory
        if let Err(e) = copy_dir_contents(&pkg_src, &build_dir) {
            println!(
                "  {} Failed to copy files: {}\n",
                "ERROR".red().bold(),
                e
            );
            fail_count += 1;
            continue;
        }

        // Run makepkg in the build directory
        let status = Command::new("makepkg")
            .arg("-s")
            .arg("--noconfirm")
            .current_dir(&build_dir)
            .status()
            .context("Failed to execute makepkg")?;

        if !status.success() {
            println!(
                "  {} makepkg exited with {}\n",
                "FAIL".red().bold(),
                status
            );
            fail_count += 1;
            continue;
        }

        // Move generated .pkg.tar.zst files to the pkgs directory
        let mut pkg_found = false;
        for file in std::fs::read_dir(&build_dir)? {
            let file = file?;
            let fname = file.file_name();
            let fname_str = fname.to_string_lossy();
            if fname_str.ends_with(".pkg.tar.zst") {
                let dest = pkgs_dir.join(&fname);
                std::fs::rename(file.path(), &dest).with_context(|| {
                    format!("Failed to move {} to pkgs/", fname_str)
                })?;
                println!(
                    "  {} {}",
                    "->".green(),
                    fname_str.green()
                );
                pkg_found = true;
            }
        }

        if pkg_found {
            println!("  {}\n", "OK".green().bold());
            success_count += 1;
        } else {
            println!(
                "  {} No .pkg.tar.zst found after build\n",
                "WARN".yellow().bold()
            );
            fail_count += 1;
        }
    }

    // Final cleanup of the build directory
    clean_dir(&build_dir)?;

    println!(
        "{}: {} packages, {} succeeded, {} failed",
        "Summary".bold(),
        total,
        success_count.to_string().green(),
        fail_count.to_string().red()
    );

    Ok(())
}

/// Recursively copy all files and subdirectories from src to dst
fn copy_dir_contents(src: &Path, dst: &Path) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path)?;
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Remove all contents of a directory (keeping the directory itself)
fn clean_dir(dir: &Path) -> Result<()> {
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}
