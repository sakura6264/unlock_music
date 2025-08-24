use crate::decoder_worker::{DecoderState, DecoderTask, DecoderWorker, TaskResult};
use crate::error_manager::{ErrorManager, ManagedError};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct TaskManager {
    worker: DecoderWorker,
    progress: Arc<Mutex<HashMap<PathBuf, DecoderState>>>,
    results: Arc<Mutex<HashMap<PathBuf, TaskResult>>>,
    error_manager: ErrorManager,
}

impl TaskManager {
    pub fn new(worker_count: usize) -> Self {
        let progress = Arc::new(Mutex::new(HashMap::new()));
        let results = Arc::new(Mutex::new(HashMap::new()));
        let worker = DecoderWorker::new(progress.clone(), results.clone(), worker_count);

        Self {
            worker,
            progress,
            results,
            error_manager: ErrorManager::new(),
        }
    }

    pub fn start_decode_file(&mut self, path: &PathBuf, output_dir: &PathBuf, skip_noop: bool) {
        let task = DecoderTask {
            input_path: path.clone(),
            output_dir: output_dir.clone(),
            skip_noop,
        };
        self.worker.add_task(task);
    }

    pub fn start_decode_all(&mut self, files: &[PathBuf], output_dir: &PathBuf, skip_noop: bool) {
        for file in files {
            let task = DecoderTask {
                input_path: file.clone(),
                output_dir: output_dir.clone(),
                skip_noop,
            };
            self.worker.add_task(task);
        }
    }

    pub fn cancel_decode_file(&mut self, path: &PathBuf) {
        self.worker.cancel_task(path);
    }

    pub fn cancel_decode_all(&mut self) {
        self.worker.cancel_all();
    }

    pub fn remove_file_data(&mut self, path: &PathBuf) {
        {
            let mut progress = self.progress.lock().unwrap();
            progress.remove(path);

            let mut results = self.results.lock().unwrap();
            results.remove(path);
        }
    }

    pub fn get_file_status(&self, path: &PathBuf) -> DecoderState {
        let progress = self.progress.lock().unwrap();
        progress.get(path).cloned().unwrap_or(DecoderState::Ready)
    }

    pub fn get_file_result(&self, path: &PathBuf) -> Option<TaskResult> {
        let results = self.results.lock().unwrap();
        results.get(path).cloned()
    }

    pub fn get_completion_counts(&self) -> (usize, usize) {
        let results = self.results.lock().unwrap();
        let mut success_count = 0;
        let mut error_count = 0;

        for result in results.values() {
            match result {
                TaskResult::Success(_) => success_count += 1,
                TaskResult::Error(_) => error_count += 1,
            }
        }

        // Add errors from error manager
        error_count += self.error_manager.get_total_error_count();

        (success_count, error_count)
    }

    pub fn get_active_task_count(&self) -> usize {
        self.worker.get_active_task_count()
    }

    pub fn get_pending_task_count(&self) -> usize {
        self.worker.get_pending_task_count()
    }

    pub fn set_worker_count(&mut self, count: usize) {
        self.worker.set_worker_count(count);
    }

    pub fn add_error(&mut self, error: ManagedError) {
        self.error_manager.add_error(error);
    }

    pub fn clear_path_validation_errors(&mut self) {
        self.error_manager.clear_path_validation_errors();
    }

    pub fn clear_all_errors(&mut self) {
        let mut results = self.results.lock().unwrap();
        results.clear();
        self.error_manager.clear_all();
    }

    pub fn get_error_manager(&self) -> &ErrorManager {
        &self.error_manager
    }

    pub fn validate_output_directory(&self, output_dir: &PathBuf) -> Result<(), ManagedError> {
        if !output_dir.is_absolute() {
            return Err(ManagedError::path_not_absolute(output_dir));
        }

        if !output_dir.exists() {
            return Err(ManagedError::path_not_exists(output_dir));
        }

        if !output_dir.is_dir() {
            return Err(ManagedError::path_not_directory(output_dir));
        }

        Ok(())
    }
}
