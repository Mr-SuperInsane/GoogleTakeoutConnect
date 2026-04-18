# 開発環境セットアップ：システムのExifTool・ffmpegをbinaries/にコピー
$target = "x86_64-pc-windows-msvc"
$binDir = "$PSScriptRoot\..\src-tauri\binaries"

$exiftool = Get-Command exiftool -ErrorAction SilentlyContinue
if ($exiftool) {
    Copy-Item $exiftool.Source "$binDir\exiftool-$target.exe" -Force
    Write-Host "exiftool -> binaries/exiftool-$target.exe" -ForegroundColor Green
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
