use decoder::get_ext;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileSort {
    Name,
    DateAdded,
    Status,
}

pub struct FileManager {
    files: Vec<PathBuf>,
    files_added_time: HashMap<PathBuf, Instant>,
    file_sort: FileSort,
    supported_extensions: Vec<String>,
}

impl FileManager {
    pub fn new(supported_extensions: Vec<String>) -> Self {
        Self {
            files: Vec::new(),
            files_added_time: HashMap::new(),
            file_sort: FileSort::DateAdded,
            supported_extensions,
        }
    }

    pub fn add_file(&mut self, path: PathBuf) {
        if !self.files.contains(&path) {
            self.files_added_time.insert(path.clone(), Instant::now());
            self.files.push(path);
            self.sort_files();
        }
    }

    pub fn add_directory(&mut self, dir_path: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    self.add_directory(&path);
                } else if path.is_file() {
                    let path_string = path.to_string_lossy();
                    let ext = get_ext(&path_string);
                    if !ext.is_empty() && self.is_supported_extension(ext) {
                        self.add_file(path);
                    }
                }
            }
        }
    }

    pub fn remove_file(&mut self, index: usize) -> Option<PathBuf> {
        if index < self.files.len() {
            let path = self.files.remove(index);
            self.files_added_time.remove(&path);
            Some(path)
        } else {
            None
        }
    }

    pub fn clear_all(&mut self) {
        self.files.clear();
        self.files_added_time.clear();
    }

    pub fn get_files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn set_sort(&mut self, sort: FileSort) {
        self.file_sort = sort;
        self.sort_files();
    }

    pub fn get_sort(&self) -> &FileSort {
        &self.file_sort
    }

    fn is_supported_extension(&self, ext: &str) -> bool {
        self.supported_extensions.iter().any(|e| e == ext)
    }

    fn sort_files(&mut self) {
        match self.file_sort {
            FileSort::Name => {
                self.files.sort_by(|a, b| {
                    a.file_name()
                        .unwrap_or_default()
                        .cmp(b.file_name().unwrap_or_default())
                });
            }
            FileSort::DateAdded => {
                let now = Instant::now();
                self.files.sort_by(|a, b| {
                    let time_a = self.files_added_time.get(a).unwrap_or(&now);
                    let time_b = self.files_added_time.get(b).unwrap_or(&now);
                    time_a.cmp(time_b)
                });
            }
            FileSort::Status => {
                // Status sorting will be handled by the UI layer with progress info
                // For now, just sort by name as fallback
                self.files.sort_by(|a, b| {
                    a.file_name()
                        .unwrap_or_default()
                        .cmp(b.file_name().unwrap_or_default())
                });
            }
        }
    }

    pub fn validate_and_add_files(&mut self, files: Vec<egui::DroppedFile>) {
        for file in files {
            if let Some(path) = file.path {
                if path.is_dir() {
                    self.add_directory(&path);
                } else {
                    let path_string = path.to_string_lossy();
                    let ext = get_ext(&path_string);
                    if !ext.is_empty() && self.is_supported_extension(ext) {
                        self.add_file(path);
                    }
                }
            }
        }
    }

    pub fn validate_and_add_selected_files(&mut self, files: Vec<PathBuf>) {
        for path in files {
            let path_string = path.to_string_lossy();
            let ext = get_ext(&path_string);
            if !ext.is_empty() && self.is_supported_extension(ext) {
                self.add_file(path);
            }
        }
    }
}
