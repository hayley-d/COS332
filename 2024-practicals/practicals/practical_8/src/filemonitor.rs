use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::path::Path;
use std::sync::mpsc;
use tokio::sync::mpsc::{channel, Receiver};

pub fn file_monitor(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(Path::new(file_name), RecursiveMode::Recursive)?;
    for res in rx {
        match res {
            Ok(event) => println!("Event: {:?}", event),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}

async fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(1);

    let mut watcher = notify::recommended_watcher(tx)?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
