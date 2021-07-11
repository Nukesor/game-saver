use std::fs::{create_dir, read_dir};
use std::path::Path;

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

        // It must have an extension
        let extension = if let Some(extension) = path.extension() {
            extension.to_string_lossy().into_owned()
        } else {
            continue;
        };
        // The file must be a zstd compressed tarball
        if !extension.ends_with(".tar.zst") {
            continue;
        }

        // It must have a filename
        let filename = if let Some(name) = path.file_name() {
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
    create_dir(&backup_dir).context("Failed to create backup directory")?;

    // Create subfolders for each game
    for (name, config) in &config.games {
        let game_backup_dir = backup_dir.join(name);
        if !game_backup_dir.exists() {
            create_dir(&game_backup_dir).context(format!(
                "Failed to create backup directory for game {}",
                name
            ))?;
        }

        // Create an autosave directory, if autosave is enabled for this game.
        if config.autosave > 0 {
            let autosave_dir = game_backup_dir.join("autosaves");
            create_dir(autosave_dir).context(format!(
                "Failed to create autosave directory for game {}",
                name
            ))?;
        }
    }

    Ok(())
}
