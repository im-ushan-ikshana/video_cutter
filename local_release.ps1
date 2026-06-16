Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Universal Video Cutter - Local Release " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Check for GitHub CLI
if (!(Get-Command "gh" -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] GitHub CLI (gh) is not installed." -ForegroundColor Red
    Write-Host "Please install it by running 'winget install GitHub.cli' or from https://cli.github.com/" -ForegroundColor Yellow
    Write-Host "Make sure to run 'gh auth login' before running this script." -ForegroundColor Yellow
    exit 1
}

# 2. Determine Version and Tag
$packageJson = Get-Content -Raw "package.json" | ConvertFrom-Json
$version = $packageJson.version

# Interactive Configuration
Write-Host "Release Configuration" -ForegroundColor Magenta
Write-Host "---------------------" -ForegroundColor Magenta

$customTag = Read-Host "1. Enter a custom release tag [Leave empty to use v$version]"
if ([string]::IsNullOrWhiteSpace($customTag)) {
    $tagName = "v$version"
    Write-Host "   -> Using default tag: " -NoNewline; Write-Host "$tagName" -ForegroundColor Green
} else {
    $tagName = $customTag
    Write-Host "   -> Using custom tag: " -NoNewline; Write-Host "$tagName" -ForegroundColor Green
}

$isPreReleaseStr = Read-Host "2. Is this a pre-release? (y/N)"
$PreRelease = ($isPreReleaseStr -match "^[yY]")
if ($PreRelease) {
    Write-Host "   -> Marking as: " -NoNewline; Write-Host "Pre-Release" -ForegroundColor Yellow
} else {
    Write-Host "   -> Marking as: " -NoNewline; Write-Host "Stable Release" -ForegroundColor Green
}

$skipBuildStr = Read-Host "3. Do you want to skip the Tauri build phase? (y/N)"
$SkipBuild = ($skipBuildStr -match "^[yY]")
if ($SkipBuild) {
    Write-Host "   -> Action: " -NoNewline; Write-Host "Skipping Build" -ForegroundColor Yellow
} else {
    Write-Host "   -> Action: " -NoNewline; Write-Host "Running Build" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan

# 3. Build the Application
if (-not $SkipBuild) {
    Write-Host "[INFO] Ensuring FFmpeg sidecars are downloaded..." -ForegroundColor Blue
    .\download_ffmpeg.ps1

    Write-Host "[INFO] Installing dependencies..." -ForegroundColor Blue
    npm install

    Write-Host "[INFO] Building Tauri application (This may take a while)..." -ForegroundColor Blue
    npm run tauri build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Tauri build failed." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "[INFO] Skipping build phase (-SkipBuild provided)." -ForegroundColor Yellow
}

# 4. Extract Latest Changelog Notes
Write-Host "[INFO] Extracting release notes from CHANGELOG.md..." -ForegroundColor Blue
$changelogPath = "CHANGELOG.md"
$releaseNotes = ""
if (Test-Path $changelogPath) {
    $lines = Get-Content $changelogPath
    $inCurrentRelease = $false
    
    foreach ($line in $lines) {
        if ($line -match "^## \[$version\]") {
            $inCurrentRelease = $true
            continue
        }
        if ($inCurrentRelease -and $line -match "^## \[") {
            break # Reached the previous release
        }
        if ($inCurrentRelease) {
            $releaseNotes += "$line`n"
        }
    }
}

if ([string]::IsNullOrWhiteSpace($releaseNotes)) {
    $releaseNotes = "Release $tagName"
}
$notesFile = "temp_release_notes.txt"
$releaseNotes.Trim() | Out-File -FilePath $notesFile -Encoding utf8

# 5. Gather Artifacts
Write-Host "[INFO] Gathering build artifacts..." -ForegroundColor Blue
$targetDir = "src-tauri\target\release\bundle"
$artifacts = @()

if (Test-Path "$targetDir\msi") {
    $artifacts += Get-ChildItem -Path "$targetDir\msi" -Filter "*.msi" | Select-Object -ExpandProperty FullName
}
if (Test-Path "$targetDir\nsis") {
    $artifacts += Get-ChildItem -Path "$targetDir\nsis" -Filter "*.exe" | Select-Object -ExpandProperty FullName
}

if ($artifacts.Count -eq 0) {
    Write-Host "[ERROR] No MSI or NSIS artifacts found in $targetDir" -ForegroundColor Red
    Remove-Item $notesFile -ErrorAction SilentlyContinue
    exit 1
}

foreach ($art in $artifacts) {
    Write-Host "  -> Found: $(Split-Path $art -Leaf)" -ForegroundColor Gray
}

# 6. Push Release to GitHub
Write-Host "[INFO] Checking if release $tagName already exists..." -ForegroundColor Blue
& gh release view $tagName --json name >$null 2>&1
$releaseExists = ($LASTEXITCODE -eq 0)

if ($releaseExists) {
    Write-Host "[INFO] Release $tagName already exists. Appending artifacts to the existing release..." -ForegroundColor Yellow
    $ghArgs = @("release", "upload", $tagName)
    # Use --clobber to overwrite artifacts with the same name, but we don't overwrite the release itself
    $ghArgs += "--clobber"
    $ghArgs += $artifacts

    & gh $ghArgs
    $ghExitCode = $LASTEXITCODE
} else {
    Write-Host "[INFO] Creating new GitHub Release for $tagName..." -ForegroundColor Blue
    $ghArgs = @("release", "create", $tagName, "--title", "Universal Video Cutter $tagName", "--notes-file", $notesFile, "--target", "master")
    if ($PreRelease) {
        $ghArgs += "--prerelease"
    }
    $ghArgs += $artifacts

    & gh $ghArgs
    $ghExitCode = $LASTEXITCODE
}

# Cleanup
Remove-Item $notesFile -ErrorAction SilentlyContinue

if ($ghExitCode -eq 0) {
    Write-Host "[SUCCESS] Operations completed successfully for $tagName!" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Failed to communicate with GitHub API." -ForegroundColor Red
    exit 1
}
