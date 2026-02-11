use std::path::Path;

use directories::ProjectDirs;

use crate::DynResult;

pub struct Project {
    project_dirs: ProjectDirs,
}

impl Project {
    pub fn new() -> DynResult<Self> {
        let project_dirs = ProjectDirs::from("", "sdx", "netcheck").ok_or("damn!")?;

        Ok(Project { project_dirs })
    }

    pub fn log_dir(&self) -> &Path {
        self.project_dirs.data_local_dir()
    }
}
