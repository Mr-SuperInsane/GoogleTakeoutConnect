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
                let png_bytes = include_bytes!("../icons/icon.png");
                if let Ok(img) = image::load_from_memory(png_bytes) {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let icon = tauri::image::Image::new(rgba.as_raw(), w, h);
                    let _ = window.set_icon(icon);
                }
            }

            // Windows: exiftool_files.zip を AppData に展開してパスを設定
            #[cfg(target_os = "windows")]
            if let Err(e) = setup_exiftool_home(app) {
                eprintln!("ExifTool setup warning: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![process_takeout, open_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// インストールディレクトリの exiftool_files.zip を AppData に展開する（Windows専用）
#[cfg(target_os = "windows")]
fn setup_exiftool_home(app: &tauri::App) -> anyhow::Result<()> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("app_data_dir: {}", e))?;
    let exiftool_home = app_data_dir.join("exiftool_home");
    let exiftool_exe_dst = exiftool_home.join("exiftool.exe");

    // 既に展開済みならパスを設定して終了
    if exiftool_exe_dst.exists() {
        processor::set_exiftool_home(exiftool_home);
        return Ok(());
    }

    // インストールディレクトリ（exeと同階層）にある exiftool.exe と zip を探す
    let exe_dir = std::env::current_exe()?;
    let exe_dir = exe_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("exe parent not found"))?;

    let et_src = exe_dir.join("exiftool.exe");
    let zip_path = exe_dir.join("exiftool_files.zip");

    // どちらか欠けている場合は開発モードとみなしてスキップ
    if !et_src.exists() || !zip_path.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(&exiftool_home)?;
    std::fs::copy(&et_src, &exiftool_exe_dst)?;

    let zip_data = std::fs::read(&zip_path)?;
    let cursor = std::io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    archive.extract(&exiftool_home)?;

    processor::set_exiftool_home(exiftool_home);
    Ok(())
}
