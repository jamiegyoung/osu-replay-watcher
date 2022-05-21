use std::{
    fs::{self, File},
    io::{copy, Read, Write},
    path::{Path, PathBuf},
};

use ::error_chain::bail;
use error_chain::error_chain;

error_chain! {
  errors {
      FFMpegError {
          description("FFmpeg was unable to be installed.")
          display("FFmpeg was unable to be installed.")
      }
  }

  foreign_links {
    Io(std::io::Error);
    Reqwest(reqwest::Error);
    ZipError(zip::result::ZipError);
  }
}

const DANSER_DIR: &str = "./orw-danser";

// https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.7z

pub async fn handle_ffmpeg() -> Result<bool> {
    match which::which("ffmpeg") {
        Ok(_) => {
            println!("FFmpeg found in PATH.");
            return Ok(true);
        }
        Err(_) => {
            let danser_path = PathBuf::from(DANSER_DIR);
            let expected_ffmpeg_file_path: PathBuf = danser_path.join("ffmpeg.exe");
            println!("FFmpeg not found in PATH.");
            if expected_ffmpeg_file_path.is_file() {
                println!(
                    "FFmpeg is already downloaded in danser at {}.",
                    expected_ffmpeg_file_path.display()
                );
                return Ok(true);
            }
            println!("Downloading FFmpeg...");

            let zip_path = danser_path.join("ffmpeg.zip");
            download_ffmpeg(&danser_path, &zip_path).await?;
            extract_ffmpeg(&zip_path, &danser_path)?;
            fs::remove_file(&zip_path)?;
            Ok(false)
        }
    }
}

async fn download_ffmpeg(ffmpeg_dir_path: &PathBuf, zip_path: &PathBuf) -> Result<()> {
    fs::create_dir_all(&ffmpeg_dir_path)?;
    let mut dest = File::create(&zip_path)?;
    let target = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
    let response = reqwest::get(target).await?;
    println!("Downloading {:?}", target);
    let content = response.bytes().await?;
    copy(&mut content.as_ref(), &mut dest)?;
    Ok(())
}

fn extract_ffmpeg(zip_path: &Path, ffmpeg_path: &Path) -> Result<()> {
    let ffmpeg = File::open(zip_path)?;
    let archive = zip::ZipArchive::new(ffmpeg)?;
    extract_file(
        archive,
        ffmpeg_path,
        "ffmpeg.exe",
        "ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe",
    )?;
    Ok(())
}

fn extract_file(
    mut archive: zip::ZipArchive<File>,
    ffmpeg_path: &Path,
    file_name: &str,
    zip_file_path: &str,
) -> Result<()> {
    match archive.by_name(zip_file_path) {
        Ok(mut zip_file) => {
            let zip_metadata = zip_file.size();
            let mut dest_file = File::create(ffmpeg_path.join(file_name))?;
            let mut buffer: Vec<u8> = vec![0; zip_metadata as usize];
            zip_file.read_exact(&mut buffer)?;
            dest_file.write_all(buffer.as_slice())?;
            Ok(())
        }
        Err(_) => bail!(ErrorKind::FFMpegError),
    }
}
