mod danser;
mod ffmpeg;
mod skins;
mod watch;

use ::error_chain::bail;
use bland::Store;
use danser::run_danser_async;
use error_chain::error_chain;
use std::fs;
use std::path::{Path, PathBuf};

use crate::danser::set_danser_settings;
use crate::ffmpeg::handle_ffmpeg;
use crate::skins::skin_picker;

error_chain! {
    errors {
        DanserInstallError {
            description("Danser was unable to be installed.")
            display("Danser was unable to be installed.")
        }
    }

    foreign_links {
        NotifyError(notify::Error);
        DanserError(danser::Error);
        BlandError(bland::Error);
        FFmpegError(ffmpeg::Error);
        SkinError(skins::Error);
    }
}

fn print_section_indicator(section: &str) {
    match term_size::dimensions() {
        Some((w, _)) => {
            println!("{}", "-".repeat(w));
            println!("{}", section);
            println!("{}", "-".repeat(w));
        }
        None => {
            println!("--");
            println!("{}", section);
            println!("--");
        }
    };
}

fn get_folder(store: &Store<'_>, name: &str, store_path: &str) -> PathBuf {
    loop {
        match store.get(store_path) {
            Ok(path) => {
                match path {
                    Some(path) => return PathBuf::from(path.as_str().unwrap()),
                    None => {
                        set_folder(store, name, store_path);
                    }
                };
            }
            Err(_) => set_folder(store, name, store_path),
        }
    }
}

fn set_folder(store: &Store<'_>, name: &str, dot_path: &str) {
    println!(
        "{} folder not found, please input the {} folder path:",
        &name, &name
    );
    let mut buffer = String::new();
    let stdin: std::io::Stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer).unwrap();
        let clean_buffer = buffer.trim();
        let clean_buffer_path = Path::new(&clean_buffer);
        if clean_buffer_path.is_dir() {
            println!("Setting {} folder to {}", &name, clean_buffer);
            store.set(dot_path, clean_buffer).unwrap();
            break;
        } else {
            println!(
                "{} is not a directory, please input a valid directory:",
                clean_buffer
            );
            buffer.clear();
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut store = Store::new("orw").unwrap();
    store.set_project_suffix(Some("settings"));
    store.set_path(PathBuf::from("./"));
    store.set_pretty(true);

    if !Path::new("./orw-danser").exists() {
        fs::create_dir(PathBuf::from("./orw-danser")).unwrap();
    }

    let mut danser_store = Store::new("settings")?;
    danser_store.set_project_suffix(None);
    danser_store.set_path(PathBuf::from("./orw-danser"));
    danser_store.set_config_name("default");
    let songs_folder = get_folder(&danser_store, "Songs", "General.OsuSongsDir");
    let skins_folder = get_folder(&danser_store, "Skins", "General.OsuSkinsDir");
    let replays_folder = get_folder(&store, "Replays", "replays_folder");
    let skins_folder_path = skins_folder.as_path();
    let current_skin: String = match store.get("skin")? {
        Some(skin) => match skin.as_str() {
            Some(res_skin) => res_skin.to_string(),
            None => skin_picker(skins_folder_path)?.to_string(),
        },
        None => {
            let selected_skin = skin_picker(skins_folder_path)?;
            store.set("skin", &selected_skin)?;
            selected_skin
        }
    };
    handle_danser(
        songs_folder.to_str().unwrap(),
        skins_folder.to_str().unwrap(),
    )
    .await?;
    handle_ffmpeg().await?;
    print_section_indicator(format!("Watching For File Changes in {:?}", replays_folder).as_str());
    watch::watch(replays_folder, current_skin).await?;
    Ok(())
}

async fn handle_danser(songs_folder: &str, skins_folder: &str) -> Result<()> {
    print_section_indicator("Checking danser");
    if !danser::check_danser()? {
        print_section_indicator("Getting Danser");
        danser::get_danser().await?;
        print_section_indicator("Checking danser");
        if !danser::check_danser()? {
            bail!(ErrorKind::DanserInstallError);
        }
        println!("Setting danser settings");
        set_danser_settings(songs_folder, skins_folder)?;
    }
    println!("Initializing Danser");
    if !Path::new("./orw-danser/danser.db").exists() {
        run_danser_async(vec!["-md5", "0"], true).await;
    }
    Ok(())
}
