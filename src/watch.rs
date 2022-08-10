use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::mpsc::channel, time::Duration, vec};

use crate::danser;

pub async fn watch(osu_path: PathBuf, skin: String) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(10))?;

    watcher.watch(osu_path, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                if let DebouncedEvent::Create(file_path) = event {
                    check_file(file_path, &skin);
                }
            }
            Err(e) => println!("watch error{:?}", e),
        };
    }
}

fn check_file(file_path: PathBuf, skin: &String) {
    if file_path.is_file() {
        if let Some(ext) = file_path.extension() {
            if ext == "osr" {
                if let Some(file_path_string) = file_path.as_os_str().to_str() {
                    if let Some(name) = file_path.file_name() {
                        println!("Starting recording of {:?}", name);
                    }
                    danser::run_danser(
                        vec!["-r", file_path_string, "-record", "-skin", skin],
                        true,
                    );
                }
            }
        };
    }
}
