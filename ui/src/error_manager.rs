use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Error ID enum for different types of errors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorId {
    /// File-specific errors with the file path
    File(PathBuf),
    /// Path validation errors (affects all files)
    PathValidation,
    /// System errors
    System,
}

/// Simple error structure with ID and message
#[derive(Debug, Clone)]
pub struct ManagedError {
    pub id: ErrorId,
    pub message: String,
    pub context: Option<String>,
}

impl ManagedError {
    pub fn new(id: ErrorId, message: String) -> Self {
        Self {
            id,
            message,
            context: None,
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn get_display_message(&self) -> String {
        let mut msg = self.message.clone();

        if let Some(context) = &self.context {
            msg.push_str(&format!(" ({})", context));
        }

        if let ErrorId::File(path) = &self.id {
            msg.push_str(&format!(" [File: {}]", path.display()));
        }

        msg
    }
}

impl fmt::Display for ManagedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_display_message())
    }
}

/// Simple error manager using HashMap with ErrorId as key
pub struct ErrorManager {
    errors: HashMap<ErrorId, ManagedError>,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: ManagedError) {
        self.errors.insert(error.id.clone(), error);
    }

    /// Clear all errors
    pub fn clear_all(&mut self) {
        self.errors.clear();
    }

    /// Get total error count
    pub fn get_total_error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get file errors
    pub fn get_file_errors(&self) -> Vec<&ManagedError> {
        self.errors
            .values()
            .filter(|error| matches!(error.id, ErrorId::File(_)))
            .collect()
    }

    /// Get path validation errors
    pub fn get_path_validation_errors(&self) -> Vec<&ManagedError> {
        self.errors
            .values()
            .filter(|error| matches!(error.id, ErrorId::PathValidation))
            .collect()
    }

    /// Clear path validation errors
    pub fn clear_path_validation_errors(&mut self) {
        self.errors
            .retain(|id, _| !matches!(id, ErrorId::PathValidation));
    }
}

impl Default for ErrorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions to create common errors
impl ManagedError {
    /// Create path validation errors
    pub fn path_not_absolute(path: &PathBuf) -> Self {
        Self::new(
            ErrorId::PathValidation,
            format!(
                "Output directory must be an absolute path: {}",
                path.display()
            ),
        )
    }

    pub fn path_not_exists(path: &PathBuf) -> Self {
        Self::new(
            ErrorId::PathValidation,
            format!("Output directory does not exist: {}", path.display()),
        )
    }

    pub fn path_not_directory(path: &PathBuf) -> Self {
        Self::new(
            ErrorId::PathValidation,
            format!("Output path is not a directory: {}", path.display()),
        )
    }

    /// Create file operation errors
    pub fn file_open_failed(path: &PathBuf, error: &std::io::Error) -> Self {
        Self::new(
            ErrorId::File(path.clone()),
            format!("Failed to open file: {}", error),
        )
    }

    pub fn file_read_failed(path: &PathBuf, error: &std::io::Error) -> Self {
        Self::new(
            ErrorId::File(path.clone()),
            format!("Failed to read file: {}", error),
        )
    }

    pub fn file_write_failed(path: &PathBuf, error: &std::io::Error) -> Self {
        Self::new(
            ErrorId::File(path.clone()),
            format!("Failed to write output file: {}", error),
        )
    }

    pub fn file_no_extension(path: &PathBuf) -> Self {
        Self::new(
            ErrorId::File(path.clone()),
            "File has no extension".to_string(),
        )
    }

    pub fn directory_create_failed(path: &PathBuf, error: &std::io::Error) -> Self {
        Self::new(
            ErrorId::System,
            format!("Failed to create output directory: {}", error),
        )
        .with_context(format!("Directory: {}", path.display()))
    }

    /// Create decoder errors
    pub fn decoder_init_failed(path: &PathBuf, error: &str) -> Self {
        Self::new(
            ErrorId::File(path.clone()),
            format!("Failed to initialize decoder: {}", error),
        )
    }

    pub fn decoding_failed(path: &PathBuf, error: &str) -> Self {
        Self::new(
            ErrorId::File(path.clone()),
            format!("Failed to decode file: {}", error),
        )
    }
}
