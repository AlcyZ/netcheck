use std::{
    fs::{File, OpenOptions, metadata},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::Result;
use chrono::Local;
use clap::ValueEnum;
use serde::Serialize;

#[macro_export]
macro_rules! log {
    ($logger:expr, $msg:expr, $($key:ident $(= $val:expr)? ),* $(,)?) => {
        {
            let logger_ref = $crate::log::ensure_logger(&$logger);
            let data = serde_json::json!({
                "timestamp": chrono::Utc::now(),
                "message": $msg,
                $(
                    stringify!($key): $crate::log_val!($key $(, $val)?)
                ),*
            });
            logger_ref.log(data)
        }
    };

    ($logger:expr, $msg:expr $(,)?) => {{
        let logger_ref = $crate::log::ensure_logger(&$logger);
        let data = serde_json::json!({
            "timestamp": chrono::Utc::now(),
            "message": $msg,
        });
        logger_ref.log(data)
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_val {
    ($key:ident) => {
        $key
    };
    ($key:ident, $val:expr) => {
        $val
    };
}

pub const DEFAULT_FILE_PREFIX: &str = "netcheck";
pub const DEFAULT_MAX_SIZE: u64 = 2 * 1024 * 1024;
pub const DEFAULT_LOG_MODE: LogMode = LogMode::All;

pub struct Logger {
    dir: PathBuf,
    file_prefix: String,
    max_size: u64,
    state: Mutex<Option<LoggerState>>,
    mode: LogMode,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum LogMode {
    Silent,
    Stdout,
    File,
    All,
}

impl Logger {
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::default()
    }

    pub fn log(&self, data: impl Serialize) -> Result<()> {
        match self.mode {
            LogMode::Stdout => self.log_stdout(&data),
            LogMode::File => self.log_file(&data),
            LogMode::All => self.log_all(&data),
            LogMode::Silent => Ok(()),
        }
    }

    fn log_all(&self, data: impl Serialize) -> Result<()> {
        self.log_file(&data)?;
        self.log_stdout(&data)?;

        Ok(())
    }

    fn log_stdout(&self, data: impl Serialize) -> Result<()> {
        let content = serde_json::to_string(&data)?;
        println!("{content}");

        Ok(())
    }

    fn log_file(&self, data: impl Serialize) -> Result<()> {
        let target_path = self.get_current_file_path()?;
        let mut lock = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisened"))?;
        let needs_new_file = match &*lock {
            Some(state) => state.path != target_path || state.current_size >= self.max_size,
            None => true,
        };

        if needs_new_file {
            if let Some(old_state) = lock.take() {
                let _ = old_state.file.sync_all();
            }

            if !self.dir.exists() {
                std::fs::create_dir_all(&self.dir)?;
            }

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&target_path)?;
            let initial_size = file.metadata()?.len();

            *lock = Some(LoggerState {
                file,
                path: target_path,
                current_size: initial_size,
            })
        }

        if let Some(state) = lock.as_mut() {
            let mut buffer = serde_json::to_vec(&data)?;
            buffer.push(b'\n');

            state.file.write_all(&buffer)?;
            state.current_size += buffer.len() as u64;
        }

        Ok(())
    }

    pub fn sync(&self) -> Result<()> {
        let mut lock = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("Mutex poisened"))?;
        if let Some(state) = lock.as_mut() {
            state.file.sync_all()?;
        }

        Ok(())
    }

    fn get_current_file_path(&self) -> std::io::Result<PathBuf> {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let mut index = 0;

        loop {
            let filename = format!("{}_{}_{}.jsonl", self.file_prefix, date_str, index);
            let path = self.dir.join(filename);

            if !path.exists() {
                return Ok(path);
            }

            if metadata(&path)?.len() < self.max_size {
                return Ok(path);
            }

            index += 1;
        }
    }
}

#[derive(Default)]
pub struct LoggerBuilder {
    file_prefix: Option<String>,
    dir: Option<PathBuf>,
    max_size: Option<u64>,
    mode: Option<LogMode>,
}

impl LoggerBuilder {
    pub fn with_max_size(mut self, max_size: u64) -> Self {
        self.max_size = Some(max_size);

        self
    }

    pub fn with_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.dir = Some(dir.as_ref().to_path_buf());

        self
    }

    pub fn with_file_prefix<F: AsRef<str>>(mut self, file_prefix: F) -> Self {
        self.file_prefix = Some(file_prefix.as_ref().into());

        self
    }

    pub fn with_mode(mut self, mode: LogMode) -> Self {
        self.mode = Some(mode);

        self
    }

    pub fn build(self) -> Result<Logger> {
        let dir = self.dir.ok_or(anyhow::anyhow!(
            "Log directory is required, but was not set!"
        ))?;
        let file_prefix = self.file_prefix.unwrap_or(DEFAULT_FILE_PREFIX.into());
        let max_size = self.max_size.unwrap_or(DEFAULT_MAX_SIZE);
        let state = Mutex::new(None);
        let mode = self.mode.unwrap_or(DEFAULT_LOG_MODE);

        Ok(Logger {
            dir,
            file_prefix,
            max_size,
            state,
            mode,
        })
    }
}

struct LoggerState {
    file: File,
    path: PathBuf,
    current_size: u64,
}

#[doc(hidden)]
pub fn ensure_logger(logger: &Logger) -> &Logger {
    logger
}
