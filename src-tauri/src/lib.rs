mod commands;
mod matcher;
mod processor;

use commands::{open_directory, process_takeout};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                // PNGをRGBAにデコードしてウィンドウアイコンを設定
                let png_bytes = include_bytes!("../icons/icon.png");
                if let Ok(img) = image::load_from_memory(png_bytes) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let icon = tauri::image::Image::new(rgba.as_raw(), w, h);
                    let _ = window.set_icon(icon);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![process_takeout, open_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
