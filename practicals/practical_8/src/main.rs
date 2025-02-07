use notify::{recommended_watcher, Event, RecursiveMode, Result, Watcher};
use std::time::Duration;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    println!("watching {}", path);

    let (tx, rx) = channel(2);
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(event) => println!("File modified: {:?}", event),
            Err(e) => eprintln!("Watch Error: {:?}", e),
        }
    }
    Ok()
}
