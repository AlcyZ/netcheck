use crate::report::Report;

pub fn run(report: Report) {
    if let Some(item) = report.iter_items().max_by_key(|item| item.outages().len()) {
        println!("Logfile with most outages: {}", item.logfile_name());
    }
}
