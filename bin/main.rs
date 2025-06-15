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

    /// File to analyze
    #[arg(short, long)]
    work: Option<PathBuf>,

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

fn process_thread(
    r: crossbeam_channel::Receiver<Option<PathBuf>>,
    rules: Vec<CompiledRule>,
    send_report: crossbeam_channel::Sender<Option<Vec<Match>>>,
    delete_file_after_processing: bool,
) {
    while let Ok(Some(path)) = r.recv() {
        let csv = csv_ruler::csv::read_from_path(&path).expect("Can't open csv file");

        let results = process_file(csv, &rules).expect("Can't process file");

        let _ = send_report.send(Some(results));

        if delete_file_after_processing {
            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("Cannot delete file {path:?}: {e}");
            }
        }
    }

    // notify end process thread to report
    let _ = send_report.send(None);
}

fn send_end_message_to_threads(threads: usize, queue: &Sender<Option<PathBuf>>) {
    (0..threads).for_each(|_| {
        // stop threads (1 message per thread)
        let _ = queue.send(None);
    });
}

fn read_dir(
    input: &Path,
    work: Option<&PathBuf>,
    send_file: &Sender<Option<PathBuf>>,
) -> std::io::Result<bool> {
    let mut ret = false;
    for entry in std::fs::read_dir(input)? {
        let mut path = entry?.path();
        // maybe move in work
        if let Some(work) = work {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            let mut work_path = work.clone();
            work_path.push(&filename);
            if let Err(e) = std::fs::rename(&path, &work_path) {
                panic!("can't rename file {filename} : {e}");
            } else {
                path = work_path;
            }
        }
        //let moved_path =
        let _ = send_file.send(Some(path));
        // a file was sent
        ret = true;
    }
    Ok(ret)
}

fn watch_thread(
    path: PathBuf,
    work: Option<&PathBuf>,
    send_file: Sender<Option<PathBuf>>,
    period: u64,
) {
    // send many files
    loop {
        // watch dir
        match read_dir(&path, work, &send_file) {
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
        let _ = send_file.send(Some(args.input));
        // end processing
        send_end_message_to_threads(args.threads, &send_file);
    } else if args.input.is_dir() {
        if let Some(period) = args.period {
            let send_file = send_file.clone();
            if args.work.is_none() {
                panic!("workdir is mandatory in watch mode");
            }
            std::thread::spawn(move || {
                watch_thread(args.input, args.work.as_ref(), send_file, period);
            });
        } else {
            // do not wait for more file
            if let Err(e) = read_dir(&args.input, None, &send_file) {
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
