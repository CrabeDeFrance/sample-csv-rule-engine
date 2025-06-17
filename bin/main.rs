use clap::Parser;
use crossbeam_channel::{Receiver, Sender, unbounded};
use csv_ruler::{
    process::{Match, process_file},
    rule::CompiledRule,
};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File to analyze
    #[arg(short, long)]
    input: PathBuf,

    /// verification rules
    #[arg(short, long)]
    rules: PathBuf,

    /// number of threads
    #[arg(short, long, default_value_t = 1)]
    threads: usize,

    /// watch input dir period in milliseconds
    #[arg(short, long)]
    period: Option<u64>,
}

struct File {
    path: PathBuf,
    content: String,
}

fn process_thread(
    r: crossbeam_channel::Receiver<Option<File>>,
    rules: Vec<CompiledRule>,
    send_report: crossbeam_channel::Sender<Option<Vec<Match>>>,
    delete_file_after_processing: bool,
) {
    while let Ok(Some(file)) = r.recv() {
        let csv = csv_ruler::csv::read_from_str(&file.content).expect("Can't open csv file");

        let results = process_file(csv, &rules).expect("Can't process file");

        let _ = send_report.send(Some(results));

        if delete_file_after_processing {
            if let Err(e) = std::fs::remove_file(&file.path) {
                eprintln!("Cannot delete file {:?}: {e}", file.path);
            }
        }
    }

    // notify end process thread to report
    let _ = send_report.send(None);
}

fn send_end_message_to_threads(threads: usize, queue: &Sender<Option<File>>) {
    (0..threads).for_each(|_| {
        // stop threads (1 message per thread)
        let _ = queue.send(None);
    });
}

fn read_dir(input: &Path, send_file: &Sender<Option<File>>) -> std::io::Result<bool> {
    let mut ret = false;
    for entry in std::fs::read_dir(input)? {
        let path = entry?.path();

        // ignore hidden files
        if path.starts_with(".") {
            continue;
        }

        // read it
        let content = std::fs::read_to_string(&path)?;

        // send it
        let _ = send_file.send(Some(File { path, content }));
        // a file was sent
        ret = true;
    }
    Ok(ret)
}

fn watch_thread(path: PathBuf, send_file: Sender<Option<File>>, period: u64) {
    // send many files
    loop {
        // watch dir
        match read_dir(&path, &send_file) {
            Ok(true) => (),
            Ok(false) => std::thread::sleep(Duration::from_millis(period)),
            Err(e) => panic!("Can't read dir: {e}"),
        }
    }
}

fn report_thread(threads: usize, report_queue: Receiver<Option<Vec<Match>>>) {
    let mut threads_ended = 0;
    loop {
        if let Ok(results) = report_queue.recv() {
            if let Some(results) = results {
                results.iter().for_each(|m| {
                    println!("Match: rule '{}' record '{}'", m.rule, m.csv.join(";"))
                });
            } else {
                // end thread, count
                threads_ended += 1;

                if threads_ended == threads {
                    // quit report thread
                    break;
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut threads = vec![];

    // Create a channel of unbounded capacity for files.
    let (send_file, rcv_file) = unbounded();
    let (send_report, rcv_report) = unbounded();

    if args.input.is_file() {
        // send one file
        let content = std::fs::read_to_string(&args.input)
            .unwrap_or_else(|_| panic!("Cannot read file: {:?}", args.input));
        let _ = send_file.send(Some(File {
            path: args.input,
            content,
        }));
        // end processing
        send_end_message_to_threads(args.threads, &send_file);
    } else if args.input.is_dir() {
        if let Some(period) = args.period {
            let send_file = send_file.clone();
            std::thread::spawn(move || {
                watch_thread(args.input, send_file, period);
            });
        } else {
            // do not wait for more file
            if let Err(e) = read_dir(&args.input, &send_file) {
                panic!("Can't read dir: {e}");
            }
            // end processing
            send_end_message_to_threads(args.threads, &send_file);
        }
    } else {
        panic!("unknown file type");
    }

    // create report thread
    threads.push(std::thread::spawn(move || {
        report_thread(args.threads, rcv_report);
    }));

    // create each threads
    (0..args.threads).for_each(|_| {
        let rules = csv_ruler::rule::read_from_path(&args.rules).expect("Can't open rule file");
        let r = rcv_file.clone();
        let report = send_report.clone();

        threads.push(std::thread::spawn(move || {
            process_thread(r, rules, report, args.period.is_some())
        }));
    });

    // wait for threads to end
    for th in threads {
        let _ = th.join();
    }
}
