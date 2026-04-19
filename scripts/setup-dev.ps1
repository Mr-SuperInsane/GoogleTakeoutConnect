# 開発環境セットアップ：システムのExifTool・ffmpegをbinaries/にコピー
$target = "x86_64-pc-windows-msvc"
$binDir = "$PSScriptRoot\..\src-tauri\binaries"

$exiftool = Get-Command exiftool -ErrorAction SilentlyContinue
if ($exiftool) {
    Copy-Item $exiftool.Source "$binDir\exiftool-$target.exe" -Force
    Write-Host "exiftool -> binaries/exiftool-$target.exe" -ForegroundColor Green

    # exiftool_files（Perl DLL群）も一緒にコピー
    $etDir = Split-Path $exiftool.Source
    $etFilesDir = Join-Path $etDir "exiftool_files"
    if (Test-Path $etFilesDir) {
        Copy-Item $etFilesDir "$binDir\exiftool_files" -Recurse -Force
        Write-Host "exiftool_files -> binaries/exiftool_files/" -ForegroundColor Green
    } else {
        Write-Host "exiftool_filesが見つかりません（ExifTool 12以前の場合は不要です）" -ForegroundColor Yellow
    }
} else {
    Write-Host "ExifTool が見つかりません。インストールしてPATHを通してください。" -ForegroundColor Red
}

$ffmpeg = Get-Command ffmpeg -ErrorAction SilentlyContinue
if ($ffmpeg) {
    Copy-Item $ffmpeg.Source "$binDir\ffmpeg-$target.exe" -Force
    Write-Host "ffmpeg   -> binaries/ffmpeg-$target.exe" -ForegroundColor Green
} else {
    Write-Host "ffmpeg が見つかりません。インストールしてPATHを通してください。" -ForegroundColor Red
}

Write-Host "`n完了。npm run tauri dev で起動できます。" -ForegroundColor Cyan
