use std::{
    fs::{read_dir, remove_dir_all, remove_file},
    path::Path,
    process::Command,
};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Local;

use super::helper::files::{get_archive_files, SaveFile};
use crate::config::Config;

/// A wrapper around [save_game], which handles the cycling of autosaves.
pub fn autosave_game(config: &Config, game: &str) -> Result<()> {
    let autosave_dir = config.autosave_dir(game);
    let game_config = config.games.get(game).unwrap();

    let mut save_files = get_archive_files(&autosave_dir)?;

    // Delete old autosave files until we have one slot left for the new save.
    while save_files.len() >= game_config.autosaves {
        let save_to_delete = if let Some(file) = save_files.pop() {
            file
        } else {
            break;
        };
        if !save_to_delete.path.exists() {
            continue;
        }

        let path = save_to_delete.path;
        remove_file(&path).context(format!("Failed to remove old autosave: {:?}", path))?;
    }

    let file_name = Local::now()
        .format("autosave_%Y-%m-%d_%H-%M-%S.tar.zst")
        .to_string();

    let autosave_path = autosave_dir.join(&file_name);
    save_game(&game_config.savegame_location(), &autosave_path)
        .context("Failed to create autosave")?;

    Ok(())
}

/// A wrapper around [save_game], which handles manual saving of files.
pub fn manually_save_game(config: &Config, game: &str, name: &str) -> Result<()> {
    let save_dir = config.save_dir(game);
    let game_config = config.games.get(game).unwrap();

    let file_name = format!("{}.tar.zst", name);

    let save_path = save_dir.join(&file_name);
    save_game(&game_config.savegame_location(), &save_path)
        .context("Failed to create manual save")?;

    Ok(())
}

fn save_game(source: &Path, dest: &Path) -> Result<()> {
    // Use the parent of the souce as working directory for tar.
    // It should always have a parent, but fallback to the directory itself in case it doesn't.
    let cwd = if let Some(parent) = source.parent() {
        parent
    } else {
        source
    };
    let source_filename = source.file_name().ok_or(anyhow!(
        "Failed to get filename from savegame_location {:?}",
        source
    ))?;

    let args = vec![
        "-I".into(),
        "zstd".into(),
        "-cf".into(),
        dest.to_string_lossy().into_owned(),
        "-C".into(),
        cwd.to_string_lossy().into_owned(),
        source_filename.to_string_lossy().into_owned(),
    ];

    let output = Command::new("tar")
        .args(&args)
        .current_dir(cwd)
        .output()
        .context(format!("Failed to spawn tar command: tar {:?}", args))?;

    if !output.status.success() {
        bail!(
            "tar command '{:?}' failed:\nSTDOUT:\n{}\n\nSTDERR:\n{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
    }

    Ok(())
}

/// Take a savefile and restore the save of the respective game.
pub fn restore_save(config: &Config, game_name: &str, save: &SaveFile) -> Result<()> {
    let game_config = config.games.get(game_name).unwrap();
    let dest = game_config.savegame_location();

    remove_all_children(&dest)
        .context("Failed while removing existing savefiles during restore.")?;
    // Use the parent of the souce as working directory for tar.
    // It should always have a parent, but fallback to the directory itself in case it doesn't.
    let cwd = if let Some(parent) = dest.parent() {
        parent.to_path_buf()
    } else {
        dest.clone()
    };

    let args = vec![
        "-I".into(),
        "zstd".into(),
        "-xf".into(),
        save.path.to_string_lossy().into_owned(),
        "-C".into(),
        cwd.to_string_lossy().into_owned(),
    ];

    let output = Command::new("tar")
        .args(&args)
        .current_dir(cwd)
        .output()
        .context(format!("Failed to spawn tar command: tar {:?}", args))?;

    if !output.status.success() {
        bail!(
            "tar command '{:?}' failed:\nSTDOUT:\n{}\n\nSTDERR:\n{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
    }

    Ok(())
}

/// Remove all files in a directory.
/// We remove all files in a `savegame_location` before untarring.
/// That way we ensure that no artifacts from old or newer saves remain.
pub fn remove_all_children(path: &Path) -> Result<()> {
    let dir_files = read_dir(path).context(format!("Couldn't read directory {:?}", path))?;
    for dir_entry in dir_files {
        let dir_entry = dir_entry.context(format!("Couldn't get dir entry in {:?}", path))?;
        let path = dir_entry.path();
        if path.is_dir() {
            remove_dir_all(&path)?;
        } else if path.is_file() {
            remove_file(&path)?;
        }
    }

    Ok(())
}
