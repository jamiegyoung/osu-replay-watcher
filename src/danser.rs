use std::{
    ffi::OsString,
    fs::{self, File},
    io::{copy, Read},
    path::{Path, PathBuf},
    process::Command,
};

use bland::Store;
use crc32fast::Hasher;

const DANSER_DIR: &str = "./orw-danser";
const DANSER_HASH: u32 = 2142748014;
const DANSER_DOWNLOAD_URI: &str =
    "https://github.com/Wieku/danser-go/releases/download/0.6.9/danser-0.6.9-win.zip";

const IGNORED_FILES: [&str; 4] = ["settings", "danser.log", "danser.db", "ffmpeg.exe"];

use error_chain::error_chain;
use tokio::process::Command as AsyncCommand;

error_chain! {
  errors {
      DanserInstallError {
          description("Danser was unable to be installed.")
          display("Danser was unable to be installed.")
      }
  }

  foreign_links {
    Io(std::io::Error);
    Reqwest(reqwest::Error);
    ZipError(zip::result::ZipError);
    BlandError(bland::Error);
  }
}

pub async fn get_danser() -> Result<()> {
    let app_dir = "./tmp";
    // maybe make it work for linux later
    let mut danser_path = PathBuf::from(app_dir);
    fs::create_dir_all(&danser_path)?;
    danser_path.push("danser-win");
    danser_path.set_extension("zip");
    download_danser(&danser_path).await?;
    extract_danser(&danser_path)?;
    danser_path.pop();
    fs::remove_dir_all(danser_path)?;
    Ok(())
}

fn remove_dir_contents(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let file = entry?;
        if let Some(file_name) = file.file_name().to_str() {
            if !IGNORED_FILES.contains(&file_name) {
                fs::remove_file(file.path())?;
            }
        }
    }
    Ok(())
}

pub fn check_danser() -> Result<bool> {
    let danser_path = PathBuf::from(DANSER_DIR);
    if danser_path.is_dir() {
        println!("Danser installation found, checking version...");
        // println!("{}", get_local_danser_hash()?);
        if get_local_danser_hash()? == DANSER_HASH {
            println!("Correct version found!");
            return Ok(true);
        }
        println!("Incorrect version found, removing");
        remove_dir_contents(&danser_path)?;
    }
    println!("Danser not found");
    Ok(false)
}

fn get_local_danser_hash() -> Result<u32> {
    let mut hasher = Hasher::new();
    let danser_dir = PathBuf::from(DANSER_DIR);
    let mapped_ignored_files: Vec<OsString> = IGNORED_FILES.iter().map(OsString::from).collect();
    for entry in fs::read_dir(&danser_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if mapped_ignored_files.contains(&file_name.to_os_string()) {
                    continue;
                }
            }
            let mut file = File::open(&path)?;
            let metadata = fs::metadata(path)?;
            let mut buffer: Vec<u8> = vec![0; metadata.len() as usize];
            file.read_exact(&mut buffer)?;
            hasher.update(&buffer[..]);
        }
    }
    Ok(hasher.finalize())
}

async fn download_danser(danser_path: &Path) -> Result<()> {
    let mut dest = File::create(danser_path)?;
    let target = DANSER_DOWNLOAD_URI;
    let response = reqwest::get(target).await?;
    println!("Downloading {:?}", target);
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;
    Ok(())
}

fn extract_danser(danser_path: &Path) -> Result<()> {
    fs::create_dir_all(DANSER_DIR)?;
    let danser = File::open(danser_path)?;
    let mut archive = zip::ZipArchive::new(danser)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outfile_name = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        let outpath = PathBuf::from(DANSER_DIR).join(&outfile_name);
        println!("\rExtracting {:?}", outfile_name.into_os_string());
        let mut outfile = File::create(&outpath).unwrap();
        copy(&mut file, &mut outfile).unwrap();
    }
    Ok(())
}

pub fn run_danser(args: &[&str], verbose: bool) {
    println!("Running danser with args {:?}", args);
    let mut danser_child = Command::new("./orw-danser/danser.exe");
    if verbose {
        danser_child.args(args).spawn().unwrap();
        return;
    }
    danser_child.args(args).output().unwrap();
}

pub async fn run_danser_async(args: &[&str], verbose: bool) {
    println!("Running danser with args {:?}", args);
    let mut danser_child = AsyncCommand::new("./orw-danser/danser.exe");
    if verbose {
        danser_child
            .args(args)
            .spawn()
            .expect("ls command failed to start")
            .wait()
            .await
            .unwrap();
        return;
    }
    danser_child.args(args).output().await.unwrap();
}

pub fn set_danser_settings(songs_dir: &str, skins_dir: &str) -> Result<()> {
    let mut tmp_store = Store::new("settings")?;
    tmp_store.set_project_suffix(None);
    tmp_store.set_path(PathBuf::from("./orw-danser"));
    tmp_store.set_config_name("default");
    tmp_store.set("General.OsuSongsDir", songs_dir)?;
    tmp_store.set("General.OsuSkinsDir", skins_dir)?;
    tmp_store.set("Skin.UseColorsFromSkin", true)?;
    tmp_store.set("Skin.Cursor.UseSkinCursor", true)?;
    tmp_store.set("Cursor.Colors.EnableRainbow", false)?;
    tmp_store.set("Objects.Colors.UseComboColors", true)?;
    tmp_store.set("Objects.Colors.UseSkinComboColors", true)?;
    tmp_store.set("Playfield.Logo.Dim.Intro", 1)?;
    tmp_store.set("Playfield.Background.Parallax.Amount", 0.0)?;
    tmp_store.set("Playfield.SeizureWarning.Enabled", false)?;
    tmp_store.set("Recording.OutputDir", "../videos")?;
    tmp_store.set("General.DiscordPresenceOn", false)?;
    tmp_store.set("Graphics.ShowFPS", false)?;
    tmp_store.set("Recording.MotionBlur.Enabled", true)?;
    tmp_store.set("Recording.MotionBlur.OversampleMultiplier", 8)?;
    tmp_store.set("Recording.MotionBlur.BlendFrames", 12)?;
    Ok(())
}
