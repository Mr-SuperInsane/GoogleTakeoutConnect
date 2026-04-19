use crate::matcher::TakeoutMeta;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn make_command(program: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

pub fn find_tool(name: &str) -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Tauriのexternalbin: 実行ファイルと同じディレクトリに配置される
            let beside = dir.join(name);
            if beside.exists() {
                return Some(beside);
            }
            // Windows: .exe 付き
            let beside_exe = dir.join(format!("{}.exe", name));
            if beside_exe.exists() {
                return Some(beside_exe);
            }
        }
    }
    // フォールバック: システムのPATH（開発時）
    which::which(name).ok()
}

#[derive(Debug, PartialEq)]
pub enum FileKind {
    Image,
    Video,
}

pub fn file_kind(path: &Path) -> Option<FileKind> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "tiff" | "tif" | "heic"
        | "dng" | "cr2" | "nef" | "arw" => Some(FileKind::Image),
        "mp4" | "mov" | "m4v" | "3gp" | "mpg" | "mpeg" | "mkv" => Some(FileKind::Video),
        _ => None,
    }
}

pub fn apply_metadata(
    src: &Path,
    dst: &Path,
    meta: &TakeoutMeta,
    exiftool: &Path,
    ffmpeg: &Path,
) -> Result<()> {
    let timestamp = meta
        .taken_timestamp()
        .context("タイムスタンプが見つかりません")?;

    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .context("タイムスタンプの変換に失敗しました")?;

    // EXIFフォーマット（画像用）: "YYYY:MM:DD HH:MM:SS"
    let exif_datetime = dt.format("%Y:%m:%d %H:%M:%S").to_string();

    // ISO 8601フォーマット（動画用）: "YYYY-MM-DDTHH:MM:SSZ"
    let iso_datetime = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    std::fs::copy(src, dst).context("ファイルのコピーに失敗しました")?;

    match file_kind(src) {
        Some(FileKind::Image) => apply_image_meta(dst, &exif_datetime, exiftool)?,
        Some(FileKind::Video) => apply_video_meta(src, dst, &iso_datetime, ffmpeg)?,
        None => return Err(anyhow::anyhow!("未対応のファイル形式")),
    }

    // ファイルシステムのタイムスタンプを撮影日時に更新
    set_file_timestamps(dst, timestamp)?;

    Ok(())
}

/// ExifToolで画像メタデータを書き込む
fn apply_image_meta(dst: &Path, exif_datetime: &str, exiftool: &Path) -> Result<()> {
    let output = make_command(exiftool)
        .args([
            "-overwrite_original",
            &format!("-DateTimeOriginal={}", exif_datetime),
            &format!("-CreateDate={}", exif_datetime),
            &format!("-ModifyDate={}", exif_datetime),
            dst.to_str().context("パスの変換に失敗しました")?,
        ])
        .output()
        .context("ExifToolの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "ExifToolエラー(code={}): {} {}",
            output.status.code().unwrap_or(-1),
            stderr.trim(),
            stdout.trim()
        ));
    }
    Ok(())
}

/// ffmpegで動画メタデータを書き込む
fn apply_video_meta(src: &Path, dst: &Path, iso_datetime: &str, ffmpeg: &Path) -> Result<()> {
    let tmp = dst.with_extension(".__tmp__.mp4");

    let output = make_command(ffmpeg)
        .args([
            "-i",
            src.to_str().context("入力パスの変換に失敗しました")?,
            "-c", "copy",
            "-map_metadata", "0",
            "-metadata", &format!("creation_time={}", iso_datetime),
            "-movflags", "use_metadata_tags",
            "-y",
            tmp.to_str().context("一時パスの変換に失敗しました")?,
        ])
        .output()
        .context("ffmpegの実行に失敗しました")?;

    if output.status.success() {
        std::fs::rename(&tmp, dst).context("一時ファイルの移動に失敗しました")?;
    } else {
        let _ = std::fs::remove_file(&tmp);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "ffmpegエラー(code={}): {} {}",
            output.status.code().unwrap_or(-1),
            stderr.trim(),
            stdout.trim()
        ));
    }
    Ok(())
}

/// ファイルシステムのタイムスタンプ（更新日時・作成日時）を撮影日時に設定する
fn set_file_timestamps(path: &Path, unix_timestamp: i64) -> Result<()> {
    let ft = filetime::FileTime::from_unix_time(unix_timestamp, 0);

    // 更新日時（mtime）と最終アクセス日時を設定
    filetime::set_file_times(path, ft, ft)
        .context("ファイルの更新日時の設定に失敗しました")?;

    // Windows: 作成日時（CreationTime）も設定
    #[cfg(target_os = "windows")]
    set_windows_creation_time(path, unix_timestamp)?;

    Ok(())
}

/// Windows専用：作成日時（CreationTime）をSetFileTime APIで設定する
#[cfg(target_os = "windows")]
fn set_windows_creation_time(path: &Path, unix_timestamp: i64) -> Result<()> {
    use std::os::windows::io::AsRawHandle;

    // UnixタイムスタンプをWindows FILETIMEに変換
    // FILETIME = 1601-01-01からの100ナノ秒単位
    // Unix     = 1970-01-01からの秒単位
    // 差分     = 11644473600秒
    const EPOCH_DIFF_SECS: i64 = 11644473600;
    let ticks = (unix_timestamp + EPOCH_DIFF_SECS) * 10_000_000i64;

    let ft = windows_sys::Win32::Foundation::FILETIME {
        dwLowDateTime: (ticks & 0xFFFF_FFFF) as u32,
        dwHighDateTime: ((ticks >> 32) & 0xFFFF_FFFF) as u32,
    };

    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .context("ファイルを開けませんでした（作成日時設定）")?;

    let result = unsafe {
        windows_sys::Win32::Storage::FileSystem::SetFileTime(
            file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE,
            &ft,           // CreationTime
            std::ptr::null(), // LastAccessTime（変更しない）
            &ft,           // LastWriteTime
        )
    };

    if result == 0 {
        return Err(anyhow::anyhow!("SetFileTime に失敗しました"));
    }

    Ok(())
}
