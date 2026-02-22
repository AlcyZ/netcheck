use crate::report::Report;

pub fn handle(report: Report) {
    for item in report.iter_items() {
        println!("Logfile: {}", item.logfile_name());

        for result in item.iter_results() {
            let msg = format!("{}: {}", result.get_time(), result.connectivity());
            println!("  {msg}");
        }
        println!();
    }
}
