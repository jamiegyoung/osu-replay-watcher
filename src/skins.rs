use std::{
    fs::{self},
    path::{Path, PathBuf},
    usize,
};

use error_chain::error_chain;

error_chain! {
  foreign_links {
      Io(std::io::Error);
      FromStr(std::num::ParseIntError);
  }
}

fn clamp(current: usize, max: usize) -> usize {
    if current > max {
        return max;
    }
    current
}

fn paths_to_dir_names(skins: Vec<PathBuf>) -> Vec<String> {
    skins
        .iter()
        .filter_map(|p| {
            if let Some(file_os_str) = p.file_name() {
                if let Some(file_str) = file_os_str.to_str() {
                    return Some(file_str.to_string());
                }
            }
            None
        })
        .collect::<Vec<String>>()
}

fn get_skin_selection(skin_names: Vec<String>) -> Result<String> {
    if skin_names.is_empty() {
        println!("No skins found, skin set to default");
        return Ok("default".to_string());
    }

    let mut offset = 0;

    let stdin: std::io::Stdin = std::io::stdin();
    loop {
        println!("Please pick a skin to use:");
        let line_offset = offset * 8;
        let max_lines_to_print = clamp((offset + 1) * 8, skin_names.len());
        if offset != 0 {
            println!("0: previous page");
        }

        for (i, skin) in skin_names
            .iter()
            .enumerate()
            .take(max_lines_to_print)
            .skip(line_offset)
        {
            println!("{}: \"{}\"", i + 1 - line_offset, skin);
        }

        let max_value = max_lines_to_print - line_offset;

        if max_lines_to_print < skin_names.len() - 1 {
            println!("{}: next page", max_value + 1);
        }

        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();
        let clean_buffer = buffer.trim();
        let chosen_value = clean_buffer.parse::<usize>()?;
        if chosen_value == max_value + 1 {
            offset += 1;
            continue;
        }

        if offset != 0 && chosen_value == 0 {
            offset -= 1;
            continue;
        }
        if chosen_value <= max_value && chosen_value > 0 {
            let chosen_offset = (chosen_value - 1) + line_offset;
            if chosen_offset < skin_names.len() - 1 {
                let chosen = skin_names[chosen_offset].clone();
                println!("Chosen: {}", &chosen);
                return Ok(chosen);
            }
        }

        println!("Chosen skin is not valid, please pick a valid skin.");
    }
}

fn filter_skin_dir(skin_path: &Path) -> Result<Vec<PathBuf>> {
    let path_files = fs::read_dir(skin_path)?;
    Ok(path_files
        .filter_map(|p| {
            if let Ok(valid_path) = p {
                if let true = valid_path.path().is_dir() {
                    return Some(valid_path.path());
                }
            }
            None
        })
        .collect::<Vec<PathBuf>>())
}

pub fn skin_picker(skin_path: &Path) -> Result<String> {
    let skin_paths = filter_skin_dir(skin_path)?;
    let skin_names = paths_to_dir_names(skin_paths);
    get_skin_selection(skin_names)
}
