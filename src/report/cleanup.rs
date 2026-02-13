use std::{ops::Deref, path::Path};

use chrono::Utc;

use crate::report::Report;

pub fn handle(report: Report) {
    report
        .items
        .iter()
        .map(|i| i.logfile.path.deref())
        .for_each(|p: &Path| {
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
