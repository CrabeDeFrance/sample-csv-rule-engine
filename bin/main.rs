use clap::Parser;
use csv_ruler::process::process_file;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to analyze
    #[arg(short, long)]
    csv: String,

    /// verification rules
    #[arg(short, long)]
    rules: String,
}

fn main() {
    let args = Args::parse();

    let csv = csv_ruler::csv::read_from_path(Path::new(&args.csv)).expect("Can't open csv file");
    let rules =
        csv_ruler::rule::read_from_path(Path::new(&args.rules)).expect("Can't open rule file");

    let results = process_file(csv, rules).expect("Can't process file");
    results
        .iter()
        .for_each(|m| println!("Match: rule '{}' record '{}'", m.rule, m.csv.join(";")));
}
