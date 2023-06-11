use std::env;
use std::fs;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::pin::Pin;
use std::process;

use tokio::task::JoinSet;

fn usage() {
    println!(
        "
Rudex (Rust-indexer): Simple multi-threaded Filesystem traverser
USAGE: 
    rudex <path to dir/file>
    Ex: rudex /home/user1
        rudex /home/user1 /home/user2
OPTIONS:
    --help: Prints this message"
    )
}

fn traverse(file: &PathBuf) -> Pin<Box<dyn Future<Output = u64>>> {
    let fut = async move {
        let mut total: u64 = 0;
        let mut tasks = JoinSet::new();
        for entry in fs::read_dir(file)? {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                tasks.spawn(async move {
                    return traverse(&entry.path()).await;
                });
            } else {
                total += entry.metadata().unwrap().len();
            }
        }
        tasks.detach_all();
        while let Some(res) = tasks.join_next().await {
            let sub_total: u64 = res.unwrap();
            total += sub_total;
        }
        Ok(total)
    };
    Box::pin(fut);
}

const KILOBYTE: u64 = 1024;
const MEGABYTE: u64 = 1_048_576;
const GIGABYTE: u64 = 1_073_741_824;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Rudex: No args provided, for usage do: --help");
        process::exit(1);
    }
    if args[1] == "--help" || args[1] == "-help" {
        usage();
        process::exit(0);
    }
    for fname in args[1..].iter() {
        let metadata = fs::metadata(fname).unwrap_or_else(|err| {
            eprintln!("Rudex: \"{}\" {}", fname, err);
            process::exit(1);
        });

        let total_size: u64;
        if metadata.is_dir() {
            let fname = fname.clone();
            total_size = traverse(&fname.into()).await;
        } else {
            total_size = metadata.len()
        }

        if total_size < KILOBYTE {
            println!("{} size: {:}", fname, total_size);
        } else if total_size < MEGABYTE {
            println!(
                "{} size: {:.1}K",
                fname,
                total_size as f64 / KILOBYTE as f64
            );
        } else if total_size < GIGABYTE {
            println!(
                "{} size: {:.1}Mb",
                fname,
                total_size as f64 / MEGABYTE as f64
            );
        } else {
            println!(
                "{} size: {:.1}Gb",
                fname,
                total_size as f64 / GIGABYTE as f64
            );
        }
    }
    Ok(())
}
