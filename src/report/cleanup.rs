use std::path::PathBuf;

use chrono::Utc;

use crate::report::Report;

pub fn handle(report: Report) {
    report.iter_logfile_paths().for_each(|p: &PathBuf| {
        if let Some(str) = p.to_str() {
            let msg = match std::fs::remove_file(p) {
                Ok(_) => format!("[{}] - Success: removed file '{}'", timestamp(), str),
                Err(err) => format!(
                    "[{}] - Error:   removed file '{}' | {}",
                    timestamp(),
                    str,
                    err
                ),
            };
            println!("{msg}");
        }
    });
}

fn timestamp() -> String {
    Utc::now().format("%H:%M:%S").to_string()
}
