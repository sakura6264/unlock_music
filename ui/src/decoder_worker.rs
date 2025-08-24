use crate::error_manager::ManagedError;
use bytes::Bytes;
use decoder::{dec_init, get_ext, get_result};
use rayon::ThreadPool;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecoderState {
    Ready,
    InProgress,
    Canceled,
    Completed,
}

#[derive(Debug, Clone)]
pub struct DecoderTask {
    pub input_path: PathBuf,
    pub output_dir: PathBuf,
    pub skip_noop: bool,
}

#[derive(Debug, Clone)]
pub enum TaskResult {
    Success(PathBuf),
    Error(ManagedError),
}

pub struct DecoderWorker {
    thread_pool: ThreadPool,
    progress: Arc<Mutex<HashMap<PathBuf, DecoderState>>>,
    results: Arc<Mutex<HashMap<PathBuf, TaskResult>>>,
    canceled_tasks: Arc<Mutex<HashMap<PathBuf, bool>>>,
}

impl DecoderWorker {
    pub fn new(
        progress: Arc<Mutex<HashMap<PathBuf, DecoderState>>>,
        results: Arc<Mutex<HashMap<PathBuf, TaskResult>>>,
        worker_count: usize,
    ) -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(worker_count)
            .build()
            .expect("Failed to create thread pool");

        Self {
            thread_pool,
            progress,
            results,
            canceled_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_task(&self, task: DecoderTask) {
        let input_path = task.input_path.clone();

        // Check if task is already in progress
        {
            let progress = self.progress.lock().unwrap();
            if let Some(state) = progress.get(&input_path) {
                if *state == DecoderState::InProgress {
                    return;
                }
            }
        }

        // Mark as ready and remove from canceled list
        {
            let mut progress = self.progress.lock().unwrap();
            progress.insert(input_path.clone(), DecoderState::Ready);

            let mut canceled = self.canceled_tasks.lock().unwrap();
            canceled.remove(&input_path);
        }

        // Submit task to thread pool
        let progress = self.progress.clone();
        let results = self.results.clone();
        let canceled_tasks = self.canceled_tasks.clone();

        self.thread_pool.spawn(move || {
            Self::execute_task(task, progress, results, canceled_tasks);
        });
    }

    fn execute_task(
        task: DecoderTask,
        progress: Arc<Mutex<HashMap<PathBuf, DecoderState>>>,
        results: Arc<Mutex<HashMap<PathBuf, TaskResult>>>,
        canceled_tasks: Arc<Mutex<HashMap<PathBuf, bool>>>,
    ) {
        let input_path = task.input_path.clone();

        // Check if task was canceled before starting
        {
            let canceled = canceled_tasks.lock().unwrap();
            if canceled.get(&input_path).copied().unwrap_or(false) {
                let mut progress = progress.lock().unwrap();
                progress.insert(input_path, DecoderState::Canceled);
                return;
            }
        }

        // Mark as in progress
        {
            let mut progress = progress.lock().unwrap();
            progress.insert(input_path.clone(), DecoderState::InProgress);
        }

        // Execute the decoding
        let result = Self::decode_file(&task);

        // Check if task was canceled during execution
        {
            let canceled = canceled_tasks.lock().unwrap();
            if canceled.get(&input_path).copied().unwrap_or(false) {
                let mut progress = progress.lock().unwrap();
                progress.insert(input_path, DecoderState::Canceled);
                return;
            }
        }

        // Store result and mark as completed
        {
            let mut progress = progress.lock().unwrap();
            progress.insert(input_path.clone(), DecoderState::Completed);

            let mut results = results.lock().unwrap();
            results.insert(input_path, result);
        }
    }

    fn decode_file(task: &DecoderTask) -> TaskResult {
        let input_path = &task.input_path;
        let output_dir = &task.output_dir;
        let skip_noop = task.skip_noop;

        // Read input file
        let mut file = match fs::File::open(input_path) {
            Ok(file) => file,
            Err(e) => return TaskResult::Error(ManagedError::file_open_failed(input_path, &e)),
        };

        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            return TaskResult::Error(ManagedError::file_read_failed(input_path, &e));
        }

        // Get file extension
        let path_string = input_path.to_string_lossy();
        let ext = get_ext(&path_string);
        if ext.is_empty() {
            return TaskResult::Error(ManagedError::file_no_extension(input_path));
        }

        // Initialize decoder
        let decoder = match dec_init(Bytes::from(buffer), skip_noop, ext) {
            Ok(decoder) => decoder,
            Err(e) => {
                return TaskResult::Error(ManagedError::decoder_init_failed(
                    input_path,
                    &e.to_string(),
                ))
            }
        };

        // Decode the file
        let decoded_data = match get_result(decoder, Some(&path_string)) {
            Ok(data) => data,
            Err(e) => {
                return TaskResult::Error(ManagedError::decoding_failed(input_path, &e.to_string()))
            }
        };

        // Determine output path and extension
        let output_ext = Self::determine_output_extension(&decoded_data);
        let file_stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        let output_filename = format!("{}{}", file_stem, output_ext);
        let output_path = output_dir.join(output_filename);

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    return TaskResult::Error(ManagedError::directory_create_failed(
                        &parent.to_path_buf(),
                        &e,
                    ));
                }
            }
        }

        // Write decoded file
        if let Err(e) = fs::write(&output_path, decoded_data) {
            return TaskResult::Error(ManagedError::file_write_failed(&output_path, &e));
        }

        TaskResult::Success(output_path)
    }

    fn determine_output_extension(data: &[u8]) -> &'static str {
        // Use decoder's sniffing capability to determine the correct extension
        match decoder::internal::sniff::audio_extension_with_fallback(data, String::new()).as_str()
        {
            ".mp3" => ".mp3",
            ".wav" => ".wav",
            ".flac" => ".flac",
            ".ogg" => ".ogg",
            ".m4a" => ".m4a",
            ".aac" => ".aac",
            ".wma" => ".wma",
            _ => ".mp3", // Default fallback
        }
    }

    pub fn cancel_task(&self, path: &PathBuf) {
        // Mark task as canceled
        {
            let mut canceled = self.canceled_tasks.lock().unwrap();
            canceled.insert(path.clone(), true);
        }

        // Update progress state
        {
            let mut progress = self.progress.lock().unwrap();
            progress.insert(path.clone(), DecoderState::Canceled);
        }
    }

    pub fn cancel_all(&self) {
        // Get all current tasks
        let paths: Vec<PathBuf> = {
            let progress = self.progress.lock().unwrap();
            progress
                .keys()
                .filter(|&path| {
                    matches!(
                        progress.get(path),
                        Some(DecoderState::Ready | DecoderState::InProgress)
                    )
                })
                .cloned()
                .collect()
        };

        // Cancel all tasks
        for path in paths {
            self.cancel_task(&path);
        }
    }

    pub fn set_worker_count(&mut self, count: usize) {
        // Create new thread pool with updated worker count
        self.thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(count)
            .build()
            .expect("Failed to create thread pool");
    }

    pub fn get_active_task_count(&self) -> usize {
        let progress = self.progress.lock().unwrap();
        progress
            .values()
            .filter(|&state| *state == DecoderState::InProgress)
            .count()
    }

    pub fn get_pending_task_count(&self) -> usize {
        let progress = self.progress.lock().unwrap();
        progress
            .values()
            .filter(|&state| *state == DecoderState::Ready)
            .count()
    }
}

impl Drop for DecoderWorker {
    fn drop(&mut self) {
        // Cancel all remaining tasks
        self.cancel_all();
        // Rayon's ThreadPool will handle cleanup automatically
    }
}
