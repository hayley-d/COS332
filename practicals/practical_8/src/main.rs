use notify::{recommended_watcher, Event, RecursiveMode, Result, Watcher};
use std::time::Duration;
use tokio::sync::mpsc::channel;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("No file path provided, please specify file path in command line arguments and try again");
        std::process::exit(1);
    }
    let file_path = args.get(1).unwrap();
    let (tx, rx) = channel(10);
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(file_path, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(event) => println!("File modified: {:?}", event),
            Err(e) => eprintln!("Watch Error: {:?}", e),
            _ => (),
        }
    }
}
