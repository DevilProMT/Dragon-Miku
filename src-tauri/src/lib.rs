#[cfg_attr(mobile, tauri::mobile_entry_point)]
mod dnt_converter;
mod act_converter;

use tauri_plugin_dialog::{MessageDialogKind, MessageDialogBuilder, DialogExt};
use glob::glob;
use std::path::{Path, PathBuf};
use std::fs::{self, read_dir};
use std::time::Instant;

fn get_all_act_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(get_all_act_files(&path));
            } else if let Some(ext) = path.extension() {
                if ext == "act" {
                    files.push(path);
                }
            }
        }
    }
    files
}

#[tauri::command(rename_all = "snake_case")]
fn convert(app: tauri::AppHandle, input_file: String, output_file: String, open_mode: String, convert_mode: String) {
    let start = Instant::now();
    let mut total_act_convert = 0;

    if open_mode == "Folder" {
        fs::create_dir_all(&output_file).expect("Failed to create output directory");

        if convert_mode == "Convert act v6 to v5" {
            let input_path = Path::new(&input_file);
            for path in get_all_act_files(input_path) {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let output_file_path = Path::new(&output_file).join(file_name);

                if let Ok(is_v6) = act_converter::convert_act_v6_to_v5(path.to_str().unwrap(), output_file_path.to_str().unwrap()) {
                    if is_v6 {
                        total_act_convert += 1;
                    }
                }
            }
        } else {
            for entry in glob(&input_file).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        let input_name = path.to_str().unwrap();
                        let file_name = path.file_name().unwrap().to_str().unwrap();

                        let output_file_path = if convert_mode == "Convert to .tsv" {
                            Path::new(&output_file).join(file_name.replace(".dnt", ".tsv"))
                        } else {
                            Path::new(&output_file).join(file_name.replace(".tsv", ".dnt"))
                        };

                        match convert_mode.as_str() {
                            "Convert to .tsv" => {
                                let _ = dnt_converter::convert_to_tsv(input_name, output_file_path.to_str().unwrap());
                            }
                            "Convert to .dnt" => {
                                let _ = dnt_converter::convert_to_dnt(input_name, output_file_path.to_str().unwrap());
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    } else {
        match convert_mode.as_str() {
            "Convert to .tsv" => {
                let _ = dnt_converter::convert_to_tsv(input_file.as_str(), output_file.as_str());
            }
            "Convert to .dnt" => {
                let _ = dnt_converter::convert_to_dnt(input_file.as_str(), output_file.as_str());
            }
            "Convert act v6 to v5" => {
                if let Ok(is_v6) = act_converter::convert_act_v6_to_v5(input_file.as_str(), output_file.as_str()) {
                    if is_v6 {
                        total_act_convert += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let duration = start.elapsed();
    
    let message = if convert_mode == "Convert act v6 to v5" {
        format!(
            "Converted {} act v6 to v5\nElapsed time: {:.2} seconds",
            total_act_convert, duration.as_secs_f32()
        )
    } else {
        format!("Total time elapsed: {:.2} seconds", duration.as_secs_f32())
    };

    MessageDialogBuilder::new(
        app.dialog().clone(), 
        "ActConverter", 
        message
    )
    .kind(MessageDialogKind::Info)
    .show(move |_response| {});
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![convert])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}