use crate::matcher::{build_json_index, find_meta_for_media, is_supported, SkipReason};
use crate::processor::{apply_metadata, find_tool};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use zip::ZipArchive;

/// 処理中のリアルタイム進捗イベント
#[derive(Debug, Serialize, Clone)]
pub struct ProgressPayload {
    pub current: u32,
    pub total: u32,
    pub success: u32,
    pub skipped: u32,
    pub failed: u32,
    pub current_file: String,
}

/// 1ファイルの処理結果ログ
#[derive(Debug, Serialize, Clone)]
pub struct LogEntry {
    pub file: String,
    pub status: String,   // "success" | "skipped" | "failed"
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub total: u32,
    pub success: u32,
    pub skipped: u32,
    pub failed: u32,
    pub output_dir: String,
    pub skip_reasons: SkipReasonSummary,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SkipReasonSummary {
    pub no_json: u32,
    pub no_timestamp: u32,
    pub parse_error: u32,
}

#[tauri::command]
pub async fn process_takeout(
    app: AppHandle,
    zip_paths: Vec<String>,
    output_dir: String,
) -> Result<ProcessResult, String> {
    let exiftool = find_tool("exiftool")
        .ok_or("ExifToolが見つかりません。インストールされているか確認してください。")?;
    let ffmpeg = find_tool("ffmpeg")
        .ok_or("ffmpegが見つかりません。インストールされているか確認してください。")?;

    let out_path = PathBuf::from(&output_dir);
    std::fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;

    let mut total = 0u32;
    let mut success = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;
    let mut skip_reasons = SkipReasonSummary::default();
    let mut errors: Vec<String> = Vec::new();

    for zip_path in &zip_paths {
        let result = process_single_zip(
            &app,
            zip_path,
            &out_path,
            &exiftool,
            &ffmpeg,
            &mut total,
            &mut success,
            &mut skipped,
            &mut failed,
            &mut skip_reasons,
        )
        .await;
        if let Err(e) = result {
            errors.push(format!("{}: {}", zip_path, e));
        }
    }

    Ok(ProcessResult {
        total,
        success,
        skipped,
        failed,
        output_dir,
        skip_reasons,
        errors,
    })
}

async fn process_single_zip(
    app: &AppHandle,
    zip_path: &str,
    out_path: &Path,
    exiftool: &Path,
    ffmpeg: &Path,
    total: &mut u32,
    success: &mut u32,
    skipped: &mut u32,
    failed: &mut u32,
    skip_reasons: &mut SkipReasonSummary,
) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let tmp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let extract_dir = tmp_dir.path().to_path_buf();

    // ZIP展開
    let zip_total = archive.len();
    for i in 0..zip_total {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let entry_path = extract_dir.join(entry.name());
        if let Some(parent) = entry_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut out = std::fs::File::create(&entry_path).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut out, &buf).map_err(|e| e.to_string())?;
    }

    // ファイル収集
    let files: Vec<PathBuf> = walkdir::WalkDir::new(&extract_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let media_files: Vec<PathBuf> = files.iter().filter(|p| is_supported(p)).cloned().collect();
    *total += media_files.len() as u32;

    // JSONインデックス構築（全ディレクトリを横断）
    let json_index = build_json_index(&files);

    let completed_dir = out_path.join("completed");
    let failed_dir = out_path.join("failed");
    std::fs::create_dir_all(&completed_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&failed_dir).map_err(|e| e.to_string())?;

    let media_count = media_files.len() as u32;

    for (idx, media_path) in media_files.iter().enumerate() {
        let file_name = media_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let (status, message) = match find_meta_for_media(media_path, &json_index) {
            Ok(meta) => {
                let dst = unique_dst(&completed_dir, &file_name);
                match apply_metadata(media_path, &dst, &meta, exiftool, ffmpeg) {
                    Ok(_) => {
                        *success += 1;
                        ("success".to_string(), "メタデータを書き込みました".to_string())
                    }
                    Err(e) => {
                        *failed += 1;
                        let dst_failed = unique_dst(&failed_dir, &file_name);
                        let _ = std::fs::copy(media_path, &dst_failed);
                        // apply_metadata がコピーした dst を削除
                        let _ = std::fs::remove_file(&dst);
                        ("failed".to_string(), e.to_string())
                    }
                }
            }
            Err(reason) => {
                match &reason {
                    SkipReason::NoJsonFound => skip_reasons.no_json += 1,
                    SkipReason::NoTimestamp => skip_reasons.no_timestamp += 1,
                    SkipReason::JsonParseError(_) => skip_reasons.parse_error += 1,
                }
                *skipped += 1;
                let dst = unique_dst(&completed_dir, &file_name);
                let _ = std::fs::copy(media_path, &dst);
                ("skipped".to_string(), reason.message())
            }
        };

        // ログエントリ送信
        let _ = app.emit("log", LogEntry {
            file: file_name.clone(),
            status: status.clone(),
            message: message.clone(),
        });

        // 進捗送信（10件に1回 or 最後）
        let cur = idx as u32 + 1;
        if cur % 10 == 0 || cur == media_count {
            let _ = app.emit("progress", ProgressPayload {
                current: cur,
                total: media_count,
                success: *success,
                skipped: *skipped,
                failed: *failed,
                current_file: file_name,
            });
        }
    }

    Ok(())
}

/// 同名ファイルが存在する場合は `stem_1.ext` のように連番を付与して返す
fn unique_dst(dir: &Path, file_name: &str) -> PathBuf {
    let path = dir.join(file_name);
    if !path.exists() {
        return path;
    }
    let p = Path::new(file_name);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let ext = p.extension().and_then(|e| e.to_str());
    for i in 1u32.. {
        let candidate = match ext {
            Some(e) => format!("{}_{}.{}", stem, i, e),
            None => format!("{}_{}", stem, i),
        };
        let candidate_path = dir.join(&candidate);
        if !candidate_path.exists() {
            return candidate_path;
        }
    }
    path
}

#[tauri::command]
pub fn open_directory(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}
