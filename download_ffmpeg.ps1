param(
    [switch]$Force = $false
)

$binDir = "src-tauri\bin"
$ffmpegExe = "$binDir\ffmpeg-x86_64-pc-windows-msvc.exe"
$ffprobeExe = "$binDir\ffprobe-x86_64-pc-windows-msvc.exe"

if (-not $Force -and (Test-Path $ffmpegExe) -and (Test-Path $ffprobeExe)) {
    Write-Host "[INFO] FFmpeg binaries already exist in $binDir. Skipping download." -ForegroundColor Green
    exit 0
}

Write-Host "[INFO] Downloading FFmpeg..." -ForegroundColor Yellow
$url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$zipPath = "ffmpeg.zip"
$extractPath = "ffmpeg_extracted"
Invoke-WebRequest -Uri $url -OutFile $zipPath
Expand-Archive -Path $zipPath -DestinationPath $extractPath
$binPath = "$extractPath\ffmpeg-master-latest-win64-gpl\bin"
New-Item -ItemType Directory -Force -Path "src-tauri\bin"
Copy-Item "$binPath\ffmpeg.exe" "src-tauri\bin\ffmpeg-x86_64-pc-windows-msvc.exe"
Copy-Item "$binPath\ffprobe.exe" "src-tauri\bin\ffprobe-x86_64-pc-windows-msvc.exe"
Remove-Item $zipPath
Remove-Item -Recurse -Force $extractPath
