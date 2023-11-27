use std::io;
use std::env;
use std::process;
use std::path::PathBuf;
use std::u64;

use tokio::fs;
use tokio::task::JoinSet;

use async_recursion::async_recursion;

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

#[async_recursion]
async fn traverse(file: &PathBuf) -> u64 {
    let mut total: u64 = 0;
    let mut set = JoinSet::new();
    let mut dirs = match fs::read_dir(file).await {
        Ok(dirs) => dirs,
        Err(err) => {
            eprintln!("Rudex Error: {err}");
            return total
        },
    };
    while let Ok(Some(entry)) = dirs.next_entry().await {
        let metadata = entry.metadata().await.unwrap();
        if metadata.is_file() || metadata.is_dir() {
            total += metadata.len();
            if metadata.is_dir() {
                set.spawn(async move {
                    traverse(&entry.path()).await
                });
            }
        }
    }
    while let Some(task) = set.join_next().await { 
        total += task.unwrap() as u64
    }
    total
}

const KILOBYTE:u64 = 1024;
const MEGABYTE:u64 = 1_048_576;
const GIGABYTE:u64 = 1_073_741_824;

#[tokio::main]
async fn main() -> io::Result<()> {
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
        let metadata = fs::metadata(PathBuf::from(fname)).await.unwrap_or_else(|err| {
            eprintln!("Rudex: \"{}\" {}", fname, err);
            process::exit(1);
        });

        let total_size:u64;
        if metadata.is_dir() {
            let fname = fname.clone();
            total_size = traverse(&fname.into()).await;
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
    Ok(())
}
