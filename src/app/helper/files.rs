use std::fs::{create_dir, create_dir_all, read_dir};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::Config;

/// Return all filenames of *.tar.zst files for a given directory.
pub fn get_archive_files(path: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    let dir_files = read_dir(path)?;
    for file in dir_files {
        let file = file?;
        let path = file.path();

        // It must be a file
        if !path.is_file() {
            continue;
        }

        // File must be a zst compressed tarball
        if let Some(extension) = path.extension() {
            if extension != "zst" {
                continue;
            }
        } else {
            continue;
        };

        // Get the inner filename (*.tar)
        let tar_name = if let Some(name) = path.file_stem() {
            PathBuf::from(name)
        } else {
            continue;
        };

        // File must be a zst compressed tarball
        if let Some(extension) = tar_name.extension() {
            if extension != "tar" {
                continue;
            }
        } else {
            continue;
        };

        // Get the innermost filename without .tar.zst
        let filename = if let Some(name) = path.file_stem() {
            name.to_string_lossy().into_owned()
        } else {
            continue;
        };

        files.push(filename);
    }

    Ok(files)
}

/// Create all directories that are needed for normal operation.
pub fn init_directories(config: &Config) -> Result<()> {
    let backup_dir = config.backup_directory();
    // Create the backup directory
    create_dir_all(&backup_dir).context("Failed to create backup directory")?;

    // Create subfolders for each game
    for (name, config) in &config.games {
        let game_backup_dir = backup_dir.join(name);
        if !game_backup_dir.exists() {
            create_dir(&game_backup_dir).context(format!(
                "Failed to create backup directory for game {}",
                name
            ))?;
        }

        // Only create an autosave directory, if autosave is enabled for this game.
        if config.autosave == 0 {
            continue;
        }

        let autosave_dir = game_backup_dir.join("autosaves");
        if !autosave_dir.exists() {
            create_dir(autosave_dir).context(format!(
                "Failed to create autosave directory for game {}",
                name
            ))?;
        }
    }

    Ok(())
}
