use anyhow::{bail, Result};
use std::path::PathBuf;

use crate::config::{save_config, Config};
use crate::storage::flightrec_home;

pub fn run(force: bool, roots: Vec<PathBuf>) -> Result<()> {
    let home = flightrec_home();
    let config_path = home.join("config.toml");

    if config_path.exists() && !force {
        bail!(
            "config already exists at {}\nRun with --force to overwrite.",
            config_path.display()
        );
    }

    let watch_roots: Vec<String> = if roots.is_empty() {
        let cwd = std::env::current_dir()?;
        let canonical = std::fs::canonicalize(&cwd).unwrap_or(cwd);
        vec![canonical.to_string_lossy().into_owned()]
    } else {
        roots
            .iter()
            .map(|r| {
                std::fs::canonicalize(r)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| r.to_string_lossy().into_owned())
            })
            .collect()
    };

    let mut config = Config::default();
    config.watch.roots = watch_roots;

    save_config(&config, &config_path)?;

    println!("Config written to {}", config_path.display());
    println!();
    println!("Next steps:");
    println!("  flightrec watch --once   # take a snapshot of your watch roots");
    println!("  flightrec tui            # explore recorded diffs in the TUI");

    Ok(())
}
