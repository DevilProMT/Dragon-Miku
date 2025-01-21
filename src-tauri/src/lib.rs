#[cfg_attr(mobile, tauri::mobile_entry_point)]
mod converter;

use tauri_plugin_dialog::{MessageDialogKind,MessageDialogBuilder,DialogExt};
use glob::glob;
use std::path::Path;
use std::fs::{self};
use std::time::Instant;

#[tauri::command(rename_all = "snake_case")]
fn convert(app: tauri::AppHandle, input_file: String, output_file: String, open_mode: String, convert_mode: String) {
    let start = Instant::now();
    if open_mode == "Folder" {
        fs::create_dir_all(&output_file).expect("Failed to create output directory");
        
        for entry in glob(&input_file).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let input_name = path.to_str().unwrap();
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    if convert_mode == "Convert to .tsv" {
                        let output_file_path = Path::new(&output_file).join(file_name.replace(".dnt", ".tsv"));
                        let _ = converter::convert_to_tsv(input_name, output_file_path.to_str().unwrap());
                    } else {
                        let output_file_path = Path::new(&output_file).join(file_name.replace(".tsv", ".dnt"));
                        let _ = converter::convert_to_dnt(input_name, output_file_path.to_str().unwrap());
                    }
                }
                Err(_e) => {}
            }
        }
    } else {
        //let _ = converter::convert_to_tsv(input_file.as_str(), output_file.as_str());
        if convert_mode == "Convert to .tsv" {
            let _ = converter::convert_to_tsv(input_file.as_str(), output_file.as_str());
        } else {
            let _ = converter::convert_to_dnt(input_file.as_str(), output_file.as_str());
        }
    }
    let duration = start.elapsed();
    
    MessageDialogBuilder::new(app.dialog().clone(), "DNTableConverter", "Total time elapsed: ".to_string() + &duration.as_secs_f32().to_string() + " seconds")
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
