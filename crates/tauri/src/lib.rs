use std::fs;

use serde::Serialize;

#[derive(Serialize)]
enum FileEntryType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "directory")]
    Directory,
}

impl From<fs::FileType> for FileEntryType {
    fn from(file_type: fs::FileType) -> Self {
        if file_type.is_dir() {
            FileEntryType::Directory
        } else {
            FileEntryType::File
        }
    }
}

/// A simple DTO to send back to the front‑end
#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: FileEntryType,
}

#[tauri::command]
fn list_files(path: &str) -> Vec<FileEntry> {
    // interpret empty path as user's home directory
    let dir = if path.is_empty() { "." } else { path };
    match fs::read_dir(dir) {
        Ok(read_dir) => read_dir
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let file_type = entry.file_type().ok()?;
                let name = entry.file_name().into_string().ok()?;
                let full_path = entry.path();
                let path_str: String = full_path.to_string_lossy().to_string();
                Some(FileEntry {
                    name,
                    path: path_str,
                    entry_type: file_type.into(),
                })
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
