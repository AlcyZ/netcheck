use std::sync::OnceLock;

use regex::Regex;

static DATE_RE: OnceLock<Regex> = OnceLock::new();

pub fn sort_by_filename_date<I, F>(collection: &mut Vec<I>, cb: F)
where
    F: Fn(&I) -> &str,
{
    let re = DATE_RE.get_or_init(|| Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap());

    collection.sort_by_cached_key(|item| {
        re.captures(cb(item).as_ref())
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "0000-00-00".to_string())
    });

    collection.reverse();
}
