use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use crate::event::Event;
use std::{path::Path, sync::mpsc::channel};
use std::env;

pub async fn file_watcher(sender: mpsc::Sender<Event>) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher =
    Watcher::new(tx, notify::Config::default())?;

    let path = &env::var("WATCH_FILE")?;
    watcher.watch(Path::new(path), RecursiveMode::NonRecursive)?;

    println!("Watching...");

    let mut last_file_size: usize = 0;

    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Modify(_) |
                    EventKind::Create(_) |
                    EventKind::Remove(_) => {
                        println!("File changed");


                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e)
        }
    }

    Ok(())
}

async fn read_file(path: &Path, offset: usize) -> usize {
    

    todo!()
}