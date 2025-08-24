use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;

use crate::config_manager::{AppConfig, ConfigManager};
use crate::decoder_worker::{DecoderState, TaskResult};
use crate::error_manager::ErrorId;
use crate::file_manager::{FileManager, FileSort};
use crate::task_manager::TaskManager;
use crate::ui_components::{ExtensionGrouper, FontManager};

#[derive(Debug, Clone)]
enum FileAction {
    Remove {
        index: usize,
        cancel_if_running: bool,
        path: PathBuf,
    },
    StartDecode {
        path: PathBuf,
    },
    CancelDecode {
        path: PathBuf,
    },
}

pub struct UnlockMusicApp {
    file_manager: FileManager,
    task_manager: TaskManager,
    config: AppConfig,
    config_dirty: bool,
    last_config_save: std::time::Instant,
    show_settings: bool,
    show_about: bool,
    show_errors: bool,
    supported_extensions: Vec<String>,
}

impl UnlockMusicApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        FontManager::configure_fonts(&cc.egui_ctx);

        // Load configuration
        let config = ConfigManager::load();

        // Get supported extensions from decoder
        let supported_extensions = get_supported_extensions();

        // Initialize managers
        let file_manager = FileManager::new(supported_extensions.clone());
        let task_manager = TaskManager::new(config.worker_count);

        Self {
            file_manager,
            task_manager,
            config,
            config_dirty: false,
            last_config_save: std::time::Instant::now(),
            show_settings: false,
            show_about: false,
            show_errors: false,
            supported_extensions,
        }
    }

    fn handle_files_drop(&mut self, files: Vec<egui::DroppedFile>) {
        self.file_manager.validate_and_add_files(files);
    }

    fn get_completion_counts(&self) -> (usize, usize) {
        self.task_manager.get_completion_counts()
    }

    fn remove_file(&mut self, index: usize) {
        if let Some(path) = self.file_manager.remove_file(index) {
            self.task_manager.remove_file_data(&path);
        }
    }

    fn start_decode_all(&mut self) {
        if let Some(output_dir) = &self.config.output_dir {
            if let Err(error) = self.task_manager.validate_output_directory(output_dir) {
                self.task_manager.clear_path_validation_errors();
                self.task_manager.add_error(error);
                return;
            }

            self.task_manager.clear_path_validation_errors();
            self.task_manager.start_decode_all(
                self.file_manager.get_files(),
                output_dir,
                self.config.skip_noop,
            );
        }
    }

    fn start_decode_file(&mut self, path: &PathBuf) {
        if let Some(output_dir) = &self.config.output_dir {
            self.task_manager
                .start_decode_file(path, output_dir, self.config.skip_noop);
        }
    }

    fn cancel_decode_file(&mut self, path: &PathBuf) {
        self.task_manager.cancel_decode_file(path);
    }

    fn cancel_decode_all(&mut self) {
        self.task_manager.cancel_decode_all();
    }

    pub fn open_output_folder(&self) {
        if let Some(output_dir) = &self.config.output_dir {
            if output_dir.exists() {
                if let Err(_e) = open::that(output_dir) {
                    // Error opening folder - could be logged in the future
                }
            }
        }
    }

    fn mark_config_dirty(&mut self) {
        self.config_dirty = true;
    }

    fn select_output_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.config.output_dir = Some(path);
            self.mark_config_dirty();
        }
    }

    fn select_input_files(&mut self) {
        if let Some(files) = rfd::FileDialog::new()
            .set_title("Select Files to Decode")
            .pick_files()
        {
            self.file_manager.validate_and_add_selected_files(files);
        }
    }

    fn select_input_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select Folder to Process")
            .pick_folder()
        {
            self.file_manager.add_directory(&path);
        }
    }

    fn render_menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Files").clicked() {
                    ui.close();
                    self.select_input_files();
                }

                if ui.button("Open Folder").clicked() {
                    ui.close();
                    self.select_input_folder();
                }

                if ui.button("Set Output Folder").clicked() {
                    ui.close();
                    self.select_output_folder();
                }

                if ui.button("Open Output Folder").clicked() {
                    ui.close();
                    self.open_output_folder();
                }

                ui.separator();

                if ui.button("Exit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Options", |ui| {
                if ui
                    .checkbox(&mut self.config.theme_dark, "Dark Theme")
                    .clicked()
                {
                    let theme = if self.config.theme_dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                    ui.ctx().set_visuals(theme);
                    self.mark_config_dirty();
                }

                if ui.button("Settings").clicked() {
                    ui.close();
                    self.show_settings = true;
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    ui.close();
                    self.show_about = true;
                }
            });
        });
    }

    fn render_file_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Sort by:");

            let mut current_sort = self.file_manager.get_sort().clone();

            if ui
                .radio_value(&mut current_sort, FileSort::Name, "Name")
                .clicked()
            {
                self.file_manager.set_sort(FileSort::Name);
                self.config.file_sort = FileSort::Name;
                self.mark_config_dirty();
            }

            if ui
                .radio_value(&mut current_sort, FileSort::Status, "Status")
                .clicked()
            {
                self.file_manager.set_sort(FileSort::Status);
                self.config.file_sort = FileSort::Status;
                self.mark_config_dirty();
            }

            if ui
                .radio_value(&mut current_sort, FileSort::DateAdded, "Date Added")
                .clicked()
            {
                self.file_manager.set_sort(FileSort::DateAdded);
                self.config.file_sort = FileSort::DateAdded;
                self.mark_config_dirty();
            }
        });

        // Collect actions to avoid borrowing conflicts
        let mut actions = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, file) in self.file_manager.get_files().iter().enumerate() {
                let filename = file.file_name().unwrap_or_default().to_string_lossy();

                let status = self.task_manager.get_file_status(file);

                let (status_text, status_color) = match status {
                    DecoderState::Ready => ("Ready", Color32::from_rgb(200, 100, 0)), // Orange-brown, visible in both themes
                    DecoderState::InProgress => ("Processing...", Color32::BLUE),
                    DecoderState::Completed => match self.task_manager.get_file_result(file) {
                        Some(TaskResult::Success(_)) => ("Completed", Color32::GREEN),
                        Some(TaskResult::Error(_)) => ("Failed", Color32::RED),
                        None => ("Unknown", Color32::GRAY),
                    },
                    DecoderState::Canceled => ("Canceled", Color32::GRAY),
                };

                let file_path = file.clone();

                ui.horizontal(|ui| {
                    if ui.button("Remove").clicked() {
                        actions.push(FileAction::Remove {
                            index: i,
                            cancel_if_running: status == DecoderState::InProgress,
                            path: file_path.clone(),
                        });
                    }

                    ui.label(filename.as_ref());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(status_text).color(status_color));

                        match status {
                            DecoderState::Ready => {
                                let decode_enabled = self.config.output_dir.is_some();
                                if ui
                                    .add_enabled(decode_enabled, egui::Button::new("Decode"))
                                    .clicked()
                                {
                                    actions.push(FileAction::StartDecode {
                                        path: file_path.clone(),
                                    });
                                }
                            }
                            DecoderState::InProgress => {
                                if ui.button("Cancel").clicked() {
                                    actions.push(FileAction::CancelDecode {
                                        path: file_path.clone(),
                                    });
                                }
                            }
                            DecoderState::Completed => {
                                if let Some(result) = self.task_manager.get_file_result(file) {
                                    match result {
                                        TaskResult::Success(output_path) => {
                                            let output_name = output_path
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy();

                                            ui.label(format!("Output: {}", output_name));
                                        }
                                        TaskResult::Error(_) => {
                                            // Error details removed from file list
                                        }
                                    }
                                }
                            }
                            DecoderState::Canceled => {
                                if ui.button("Retry").clicked() {
                                    actions.push(FileAction::StartDecode {
                                        path: file_path.clone(),
                                    });
                                }
                            }
                        }
                    });
                });

                ui.separator();
            }
        });

        // Execute collected actions
        for action in actions {
            match action {
                FileAction::Remove {
                    index,
                    cancel_if_running,
                    path,
                } => {
                    if cancel_if_running {
                        self.cancel_decode_file(&path);
                    }
                    self.remove_file(index);
                }
                FileAction::StartDecode { path } => {
                    // Validate output directory path before starting decode
                    if let Some(output_dir) = &self.config.output_dir {
                        if let Err(error) = self.task_manager.validate_output_directory(output_dir)
                        {
                            self.task_manager.add_error(error);
                            self.show_errors = true;
                        } else {
                            self.start_decode_file(&path);
                        }
                    }
                }
                FileAction::CancelDecode { path } => {
                    self.cancel_decode_file(&path);
                }
            }
        }
    }

    fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut config_changed = false;

        egui::Window::new("Settings")
            .open(&mut self.show_settings)
            .resizable(false)
            .show(ctx, |ui| {
                if ui
                    .checkbox(&mut self.config.skip_noop, "Skip No-op Decoders")
                    .changed()
                {
                    config_changed = true;
                }

                ui.horizontal(|ui| {
                    ui.label("Worker Threads:");
                    let max_workers = num_cpus::get() * 2;
                    if ui
                        .add(egui::Slider::new(
                            &mut self.config.worker_count,
                            1..=max_workers,
                        ))
                        .changed()
                    {
                        config_changed = true;
                    }

                    if ui.button("Apply").clicked() {
                        self.task_manager.set_worker_count(self.config.worker_count);
                    }
                });

                ui.separator();
                ui.label("Supported File Extensions:");

                // Group extensions by first letter
                let grouped_extensions =
                    ExtensionGrouper::group_by_first_letter(&self.supported_extensions);

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for (group_name, extensions) in grouped_extensions {
                            ui.horizontal_wrapped(|ui| {
                                ui.strong(format!("{}: ", group_name));
                                for (i, ext) in extensions.iter().enumerate() {
                                    if i > 0 {
                                        ui.label(", ");
                                    }
                                    ui.label(ext);
                                }
                            });
                        }
                    });
            });

        // Mark config as dirty if settings changed
        if config_changed {
            self.mark_config_dirty();
        }
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .open(&mut self.show_about)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Unlock Music");
                ui.label("A tool to decrypt encrypted music files.");
                ui.label("Supports various formats from NetEase, QQ Music, Kugou, Kuwo, etc.");
                ui.separator();
                ui.label(
                    "This software is open source and provided for educational purposes only.",
                );
            });
    }

    fn render_errors_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Error Details")
            .open(&mut self.show_errors)
            .resizable(true)
            .default_size([500.0, 300.0])
            .show(ctx, |ui| {
                ui.label("Error Details:");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Show path validation errors first
                    let path_errors = self
                        .task_manager
                        .get_error_manager()
                        .get_path_validation_errors();
                    if !path_errors.is_empty() {
                        ui.heading("Path Validation Errors:");
                        for error in path_errors {
                            ui.label(
                                RichText::new(error.get_display_message()).color(Color32::RED),
                            );
                            ui.separator();
                        }
                        ui.add_space(10.0);
                    }

                    // Show file processing errors from task manager
                    for file in self.file_manager.get_files() {
                        if let Some(TaskResult::Error(error)) =
                            self.task_manager.get_file_result(file)
                        {
                            ui.horizontal(|ui| {
                                ui.strong(
                                    file.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .as_ref(),
                                );
                                ui.label(":");
                            });
                            ui.label(
                                RichText::new(error.get_display_message()).color(Color32::RED),
                            );
                            ui.separator();
                        }
                    }

                    // Show file errors from error manager
                    let file_errors = self.task_manager.get_error_manager().get_file_errors();
                    if !file_errors.is_empty() {
                        ui.heading("Additional File Processing Errors:");
                        for error in file_errors {
                            if let ErrorId::File(path) = &error.id {
                                ui.horizontal(|ui| {
                                    ui.strong(
                                        path.file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .as_ref(),
                                    );
                                    ui.label(":");
                                });
                            }
                            ui.label(
                                RichText::new(error.get_display_message()).color(Color32::RED),
                            );
                            ui.separator();
                        }
                    }
                });
            });
    }

    fn clear_all_errors(&mut self) {
        self.task_manager.clear_all_errors();
    }

    fn save_config_if_dirty(&mut self) {
        if self.config_dirty {
            if let Err(e) = ConfigManager::save(&self.config) {
                eprintln!("Failed to save config: {}", e);
            }
            self.config_dirty = false;
            self.last_config_save = std::time::Instant::now();
        }
    }

    fn periodic_config_save(&mut self) {
        // Save config every 5 seconds if dirty
        const SAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);
        
        if self.config_dirty && self.last_config_save.elapsed() >= SAVE_INTERVAL {
            self.save_config_if_dirty();
        }
    }
}

// Implement the eframe App trait
impl eframe::App for UnlockMusicApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save config on app shutdown
        self.save_config_if_dirty();
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle file drops
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            self.handle_files_drop(ctx.input(|i| i.raw.dropped_files.clone()));
        }

        // Set theme
        let theme = if self.config.theme_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        ctx.set_visuals(theme);

        // Main layout
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_menu_bar(ui);

            ui.horizontal(|ui| {
                ui.label("Output Path:");

                let mut output_path_text = self
                    .config
                    .output_dir
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                if ui.text_edit_singleline(&mut output_path_text).changed() {
                    if !output_path_text.is_empty() {
                        self.config.output_dir = Some(PathBuf::from(output_path_text));
                    } else {
                        self.config.output_dir = None;
                    }
                    self.mark_config_dirty();
                }

                if ui.button("Browse").clicked() {
                    self.select_output_folder();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Add Files").clicked() {
                    self.select_input_files();
                }

                if ui.button("Add Folder").clicked() {
                    self.select_input_folder();
                }

                ui.separator();

                // Clear All button - always visible but disabled when no files
                let clear_enabled = !self.file_manager.is_empty();
                if ui
                    .add_enabled(clear_enabled, egui::Button::new("Clear All"))
                    .clicked()
                {
                    self.cancel_decode_all();
                    self.file_manager.clear_all();
                }

                // Decode All button - always visible but disabled when no files or no output dir
                let decode_enabled =
                    !self.file_manager.is_empty() && self.config.output_dir.is_some();
                if ui
                    .add_enabled(decode_enabled, egui::Button::new("Decode All"))
                    .clicked()
                {
                    // Validate output directory path before starting decode all
                    if let Some(output_dir) = &self.config.output_dir {
                        if let Err(error) = self.task_manager.validate_output_directory(output_dir)
                        {
                            self.task_manager.clear_path_validation_errors();
                            self.task_manager.add_error(error);
                            self.show_errors = true;
                        } else {
                            self.start_decode_all();
                        }
                    }
                }

                // Cancel All button - always visible but disabled when no files
                let cancel_enabled = !self.file_manager.is_empty();
                if ui
                    .add_enabled(cancel_enabled, egui::Button::new("Cancel All"))
                    .clicked()
                {
                    self.cancel_decode_all();
                }

                ui.separator();
            });

            ui.separator();
            // Enhanced worker status display
            let active_tasks = self.task_manager.get_active_task_count();
            let pending_tasks = self.task_manager.get_pending_task_count();
            let (success_count, error_count) = self.get_completion_counts();
            let total_files = self.file_manager.len();
            // Single line status with spinner and error button
            ui.horizontal(|ui| {
                if active_tasks > 0 {
                    ui.spinner();
                }
                ui.label(format!(
                    "Files: {} | Active: {} | Pending: {} | Success: {} | Errors: {}",
                    total_files, active_tasks, pending_tasks, success_count, error_count
                ));
                if error_count > 0 {
                    if ui.button(format!("{} Errors", error_count)).clicked() {
                        self.show_errors = true;
                    }
                    if ui.button("Clear Errors").clicked() {
                        self.clear_all_errors();
                    }
                }
            });

            ui.separator();

            // Files list
            self.render_file_list(ui);
        });

        // Render dialogs
        self.render_settings_dialog(ctx);
        self.render_about_dialog(ctx);
        self.render_errors_dialog(ctx);

        // Periodic config save
        self.periodic_config_save();

        // Request repaint for animations
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

// Function to get supported extensions from the decoder
fn get_supported_extensions() -> Vec<String> {
    // Return a comprehensive list of supported extensions
    // This matches the extensions registered in the decoder map
    decoder::algo::get_static_decoder_map()
        .0
        .keys()
        .cloned()
        .collect()
}
