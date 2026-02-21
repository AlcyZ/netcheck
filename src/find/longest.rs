use crate::report::Report;

pub fn run(report: Report) {
    if let Some(outage) = report
        .all_outages()
        .iter()
        .max_by_key(|outage| outage.duration())
    {
        println!("Longest {outage}");
    }
}
