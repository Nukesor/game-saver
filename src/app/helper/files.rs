use std::fs::read_dir;
use std::path::Path;

use anyhow::Result;

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
