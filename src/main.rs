use std::fs;
use std::env;
use std::process;
use std::thread;
use std::sync::mpsc::{Sender, channel};
use std::path::PathBuf;

fn usage() {
    println!("
Rudex (Rust-indexer): Simple multi-threaded Filesystem traverser
USAGE: 
    rudex <path to dir/file>
    Ex: rudex /home/user1
        rudex /home/user1 /home/user2
OPTIONS:
    --help: Prints this message")
}

fn traverse(file: &PathBuf, ch: Sender<u64>) {
    let mut total:u64 = 0;
    let mut threads:u32 = 0;
    let (tx, rx) = channel();
    let entries:Vec<PathBuf> = fs::read_dir(file)
        .expect("Rudex: unable to read dir")
        .map(|entry| entry.unwrap().path())
        .collect();
    for path in entries {
        if path.is_file() || path.is_dir() {
            total += path.metadata().expect("Rudex: error reading file").len();
            if path.is_dir() {
                threads += 1;
                let tnx = tx.clone();
                thread::spawn(move|| {
                    traverse(&path, tnx);
                });
            }
        }
    }
    for _ in 0..threads {
        total += rx.recv().unwrap();
    }
    ch.send(total).unwrap_or_else(|err| {
        eprintln!("Rudex: Channel error {}", err);
        process::exit(1);
    });
}

const KILOBYTE:u64 = 1024;
const MEGABYTE:u64 = 1_048_576;
const GIGABYTE:u64 = 1_073_741_824;

fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Rudex: No args provided, for usage do: --help");
        process::exit(1);
    }
    if args[1] == "--help" || args[1] == "-help" {
        usage();
        process::exit(0);
    }
    for fname in args[1..].iter() {
        let metadata = fs::metadata(PathBuf::from(fname)).unwrap_or_else(|err| {
            eprintln!("Rudex: \"{}\" {}", fname, err);
            process::exit(1);
        });

        let total_size:u64;
        if metadata.is_dir() {
            let (tx, rx) = channel();
            let fname = fname.clone();
            thread::spawn(move|| {
                traverse(&fname.into(), tx);
            });
            total_size = rx.recv().unwrap();
        } else {
            total_size = metadata.len()
        }

        if total_size < KILOBYTE {
            println!("{} size: {:}", fname, total_size); 
        } else if total_size < MEGABYTE {
            println!("{} size: {:.1}K", fname, total_size as f64/KILOBYTE as f64); 
        } else if total_size < GIGABYTE {
            println!("{} size: {:.1}Mb", fname, total_size as f64/MEGABYTE as f64); 
        } else {
            println!("{} size: {:.1}Gb", fname, total_size as f64/GIGABYTE as f64); 
        }
    }
}
