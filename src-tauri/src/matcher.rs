use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct TakeoutMeta {
    #[allow(dead_code)]
    pub title: Option<String>,
    #[serde(rename = "photoTakenTime")]
    pub photo_taken_time: Option<Timestamp>,
    #[serde(rename = "creationTime")]
    pub creation_time: Option<Timestamp>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Timestamp {
    pub timestamp: Option<String>,
}

impl TakeoutMeta {
    pub fn taken_timestamp(&self) -> Option<i64> {
        self.photo_taken_time
            .as_ref()
            .and_then(|t| t.timestamp.as_ref())
            .or_else(|| self.creation_time.as_ref().and_then(|t| t.timestamp.as_ref()))
            .and_then(|s| s.parse::<i64>().ok())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkipReason {
    NoJsonFound,
    NoTimestamp,
    JsonParseError(String),
}

impl SkipReason {
    pub fn message(&self) -> String {
        match self {
            SkipReason::NoJsonFound => "JSONメタデータが見つかりませんでした".to_string(),
            SkipReason::NoTimestamp => "JSONにタイムスタンプが含まれていません".to_string(),
            SkipReason::JsonParseError(e) => format!("JSON解析エラー: {}", e),
        }
    }
}

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "tiff", "tif", "heic",
    "dng", "cr2", "nef", "arw",
    "mp4", "mov", "m4v", "3gp", "mpg", "mpeg", "mkv",
];

/// 編集済みファイルのサフィックスパターン（多言語対応）
const EDITED_SUFFIXES: &[&str] = &[
    "-edited", "-bearbeitet", "-modifié", "-modifie",
    "-editado", "-modificato", "(1)", "(2)", "(3)",
];

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// JSONインデックスを構築（小文字キー → JSONパス）
pub fn build_json_index(files: &[PathBuf]) -> HashMap<String, PathBuf> {
    files
        .iter()
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        })
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| (n.to_lowercase(), p.clone()))
        })
        .collect()
}

/// メディアファイルに対応するJSONメタデータを探す
/// 複数の命名パターンを試行して最大限マッチングする
pub fn find_meta_for_media(
    media_path: &Path,
    json_index: &HashMap<String, PathBuf>,
) -> Result<TakeoutMeta, SkipReason> {
    let name = match media_path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return Err(SkipReason::NoJsonFound),
    };

    let candidates = build_json_candidates(name);

    for candidate in &candidates {
        let key = candidate.to_lowercase();
        if let Some(json_path) = json_index.get(&key) {
            return load_meta(json_path);
        }
    }

    Err(SkipReason::NoJsonFound)
}

fn load_meta(json_path: &Path) -> Result<TakeoutMeta, SkipReason> {
    let content = std::fs::read_to_string(json_path)
        .map_err(|e| SkipReason::JsonParseError(e.to_string()))?;
    let meta: TakeoutMeta = serde_json::from_str(&content)
        .map_err(|e| SkipReason::JsonParseError(e.to_string()))?;
    if meta.taken_timestamp().is_none() {
        return Err(SkipReason::NoTimestamp);
    }
    Ok(meta)
}

/// JSONファイル名の候補を優先度順に生成する
fn build_json_candidates(media_name: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    // 1. 完全一致
    candidates.push(format!("{}.json", media_name));

    // 2. Google Takeoutのファイル名切り詰め対応（46〜51文字）
    for limit in [46usize, 47, 51] {
        let truncated = truncate_bytes(media_name, limit);
        if truncated != media_name {
            candidates.push(format!("{}.json", truncated));
        }
    }

    // 3. 編集済みサフィックスを取り除いてオリジナルのJSONを探す
    //    例: IMG_0001-edited.jpg → IMG_0001.jpg.json
    let (stem, ext) = split_stem_ext(media_name);
    for suffix in EDITED_SUFFIXES {
        if stem.to_lowercase().ends_with(suffix) {
            let base = &stem[..stem.len() - suffix.len()];
            let original = if ext.is_empty() {
                base.to_string()
            } else {
                format!("{}.{}", base, ext)
            };
            candidates.push(format!("{}.json", original));
            // 切り詰めも試す
            for limit in [46usize, 47, 51] {
                let truncated = truncate_bytes(&original, limit);
                if truncated != original {
                    candidates.push(format!("{}.json", truncated));
                }
            }
        }
    }

    // 重複除去しつつ順序保持
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.clone()));
    candidates
}

/// ファイル名をバイト数でUTF-8境界に合わせて切り詰める
fn truncate_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &s[..boundary]
}

/// ファイル名をstemとextに分割（最後の.で分割）
fn split_stem_ext(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(pos) => (&name[..pos], &name[pos + 1..]),
        None => (name, ""),
    }
}
