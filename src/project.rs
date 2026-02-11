use std::path::Path;

use anyhow::Result;
use directories::ProjectDirs;

pub struct Project {
    project_dirs: ProjectDirs,
}

impl Project {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("", "sdx", "netcheck")
            .ok_or(anyhow::anyhow!("Failed to create project directories"))?;

        Ok(Project { project_dirs })
    }

    pub fn log_dir(&self) -> &Path {
        self.project_dirs.data_local_dir()
    }
}
