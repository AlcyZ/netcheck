use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    app::find::{FindAction, FindArgs},
    project::Project,
    report::Report,
};

mod longest;

pub fn run(args: FindArgs, project: Project) -> Result<()> {
    let logdir = match args.dir.as_deref() {
        Some(path) => path,
        None => project.log_dir(),
    };
    let logfiles = collect_all_logfiles(logdir)?;
    let report = Report::from_path_bufs(logfiles);

    match args.action {
        FindAction::Longest => longest::run(report),
    }

    Ok(())
}

fn collect_all_logfiles<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>> {
    Ok(read_dir(dir.as_ref())?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();

            if path.extension().and_then(|x| x.to_str()) == Some("jsonl") {
                Some(path)
            } else {
                None
            }
        })
        .collect())
}
