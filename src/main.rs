mod builder;
mod config;
mod pkgbuild;
mod scanner;

use anyhow::Result;
use colored::Colorize;
use scanner::ScanResult;

fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let args: Vec<String> = std::env::args().collect();

    // Subcommand dispatch
    if args.len() > 1 {
        match args[1].as_str() {
            "build" => return builder::run_build(&cwd),
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => {
                eprintln!("{} unknown command '{}'\n", "error:".red().bold(), other);
                print_help();
                std::process::exit(1);
            }
        }
    }

    // Default behavior: check for updates
    println!(
        "{} {}",
        "rchan".bold().cyan(),
        "- PKGBUILD update checker".dimmed()
    );
    println!("{} {}\n", "Scanning:".bold(), cwd.display());

    let results = scanner::scan_directory(&cwd)?;

    if results.is_empty() {
        println!(
            "{}",
            "No subdirectories with rchan.yaml + PKGBUILD found.".yellow()
        );
        return Ok(());
    }

    let mut updated_count = 0;
    let mut error_count = 0;
    let mut up_to_date_count = 0;

    for result in &results {
        match result {
            ScanResult::Updated {
                name,
                local_ver,
                remote_ver,
            } => {
                println!(
                    "{} {} {} -> {}",
                    "UPDATED".green().bold(),
                    name.white().bold(),
                    local_ver.dimmed(),
                    remote_ver.green()
                );
                updated_count += 1;
            }
            ScanResult::UpToDate { name, local_ver } => {
                println!(
                    "{} {} ({})",
                    "OK".blue().bold(),
                    name.white(),
                    local_ver.dimmed()
                );
                up_to_date_count += 1;
            }
            ScanResult::Error { name, message } => {
                println!("{} {} - {}", "ERROR".red().bold(), name.white(), message);
                error_count += 1;
            }
        }
    }

    println!();
    println!(
        "{}: {} checked, {} updated, {} up-to-date, {} errors",
        "Summary".bold(),
        results.len(),
        updated_count.to_string().green(),
        up_to_date_count.to_string().blue(),
        error_count.to_string().red()
    );

    Ok(())
}

fn print_help() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "{} {} {}",
        "rchan".bold().cyan(),
        version,
        "- PKGBUILD update checker & builder".dimmed()
    );
    println!();
    println!("{}", "USAGE:".bold());
    println!("  rchan              Check PKGBUILD updates for all packages");
    println!("  rchan build        Build all packages with makepkg");
    println!("  rchan --help, -h   Show this help message");
}
