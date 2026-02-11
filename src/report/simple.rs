use crate::report::Report;

pub fn handle(report: Report) {
    for item in report.items {
        println!("Logfile: {}", item.logfile.name);

        for result in item.results {
            let msg = format!("{}: {}", result.get_time(), result.connectivity());
            println!("  {msg}");
        }
        println!();
    }
}
