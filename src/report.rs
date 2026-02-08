use std::{
    fs::{File, read_dir},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::{DynResult, app::ReportArgs};

pub async fn run(args: ReportArgs) -> DynResult<()> {
    let files = get_matching_files(&args.location.dir, &args.location.filename)?;

    let path = files.iter().next().unwrap();
    let file = File::open(path)?;
    println!("so!: {:#?}", file);
    let reader = BufReader::new(file);

    let line = reader.lines().next().unwrap().unwrap();

    println!(":LINE: - {line}");

    Ok(())
}

fn get_matching_files<D: AsRef<Path>, P: AsRef<str>>(dir: D, prefix: P) -> DynResult<Vec<PathBuf>> {
    if !dir.as_ref().is_dir() {
        let msg = format!(
            "'{}' - No such directory",
            dir.as_ref()
                .to_str()
                .unwrap_or("<path contains invalid unicode>")
        );
        return Err(msg.into());
    }

    let pattern = format!(
        r"^{}_\d{{4}}-\d{{2}}-\d{{2}}_\d+\.jsonl$",
        regex::escape(prefix.as_ref())
    );
    let re = Regex::new(&pattern)?;

    let mut files = read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| re.is_match(s))
                .unwrap_or(false)
        })
        .collect::<Vec<PathBuf>>();

    files.sort();

    Ok(files)
}
