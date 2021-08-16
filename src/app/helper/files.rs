use std::{
    convert::TryInto,
    fs::{create_dir, create_dir_all, read_dir},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local, TimeZone};

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct SaveFile {
    pub path: PathBuf,
    pub file_name: String,
    pub last_modified: DateTime<Local>,
}

/// Return all paths and filenames of *.tar.zst files for a given directory.
/// The files are sorted by datetime.
pub fn get_archive_files(path: &Path) -> Result<Vec<SaveFile>> {
    let mut files = Vec::new();

    let dir_files = read_dir(path).context(format!("Couldn't read directory {:?}", path))?;
    for dir_entry in dir_files {
        let dir_entry = dir_entry.context(format!("Couldn't get dir entry in {:?}", path))?;
        let path = dir_entry.path();

        // Extract a DateTime<Local> from the file's creation date
        let metadata = dir_entry
            .metadata()
            .context(format!("Couldn't read metadata of file {:?}", path))?;
        let last_modified = metadata
            .modified()
            .context(format!("Couldn't read creation time of file {:?}", path))?;
        let seconds = last_modified.duration_since(UNIX_EPOCH)?.as_secs();
        let last_modified = Local.timestamp(seconds.try_into().unwrap_or(i64::MAX), 0);

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

        // Get the inner file_name (*.tar)
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

        // Get the innermost file_name without .tar.zst
        let file_name = if let Some(name) = tar_name.file_stem() {
            name.to_string_lossy().into_owned()
        } else {
            continue;
        };

        files.push(SaveFile {
            path,
            file_name,
            last_modified,
        });
    }

    // Sort by descending order -> b.cmp(a)
    files.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Ok(files)
}

/// Create all directories that are needed for normal operation.
pub fn init_directories(config: &Config) -> Result<()> {
    let backup_dir = config.backup_directory();
    // Create the backup directory
    create_dir_all(&backup_dir).context("Failed to create backup directory")?;

    // Create subfolders for each game
    for (name, game_config) in &config.games {
        // Show a special error message, if the user is using the path from the example file.
        if game_config.savegame_location == "~/some/path/to/your/save/files" {
            bail!("Please adjust the default configuration file at ~/.config/game_saver.toml",);
        }

        // Create the backup directory for this game.
        let game_backup_dir = config.save_dir(name);
        if !game_backup_dir.exists() {
            create_dir(&game_backup_dir).context(format!(
                "Failed to create backup directory for game {}",
                name
            ))?;
        }

        // Only create an autosave directory, if autosave is enabled for this game.
        if !game_config.has_autosaves() {
            continue;
        }

        let autosave_dir = config.autosave_dir(name);
        if !autosave_dir.exists() {
            create_dir(autosave_dir).context(format!(
                "Failed to create autosave directory for game {}",
                name
            ))?;
        }
    }

    Ok(())
}
