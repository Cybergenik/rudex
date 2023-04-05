use std::fs;
use std::env;
use std::process;
use std::path::PathBuf;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

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

fn traverse(file: &PathBuf) -> u64 {
    let dirs = fs::read_dir(file).unwrap_or_else(|err| {
        eprintln!("Rudex: \"{:}\" {}", file.to_str().unwrap(), err);
        process::exit(1);
    });
    let dirs: Result<Vec<_>, _> = dirs.collect();
    dirs.unwrap()
        .par_iter()
        .map(|entry| {
            if entry.path().is_dir() {
                traverse(&entry.path())
            } else {
                entry.metadata().unwrap().len()
            }
        })
        .sum()
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
            total_size = traverse(&fname.into());
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
