<#
.SYNOPSIS
    Install chesstui on Windows.
.DESCRIPTION
    Downloads the latest chesstui release from GitHub and installs it to
    %LOCALAPPDATA%\chesstui. Adds that directory to the user PATH so you
    can run `chesstui` from any terminal.
.EXAMPLE
    irm https://raw.githubusercontent.com/millerc13/chesstui/main/install.ps1 | iex
#>

$ErrorActionPreference = 'Stop'

$repo  = "millerc13/chesstui"
$bin   = "chesstui.exe"
$installDir = "$env:LOCALAPPDATA\chesstui"

# Determine latest release tag
Write-Host "Fetching latest release..." -ForegroundColor Cyan
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
$tag = $release.tag_name
Write-Host "Latest release: $tag" -ForegroundColor Green

# Find the Windows asset
$asset = $release.assets | Where-Object { $_.name -like "*x86_64-pc-windows-msvc*" }
if (-not $asset) {
    Write-Error "No Windows binary found in release $tag"
    exit 1
}

$url = $asset.browser_download_url
$zipFile = "$env:TEMP\chesstui-$tag.zip"
$extractDir = "$env:TEMP\chesstui-$tag"

# Download
Write-Host "Downloading $($asset.name)..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $url -OutFile $zipFile -UseBasicParsing

# Extract
Write-Host "Extracting..." -ForegroundColor Cyan
if (Test-Path $extractDir) { Remove-Item $extractDir -Recurse -Force }
Expand-Archive -Path $zipFile -DestinationPath $extractDir

# Find the exe inside the extracted folder
$exe = Get-ChildItem -Path $extractDir -Recurse -Filter $bin | Select-Object -First 1
if (-not $exe) {
    Write-Error "Could not find $bin in archive"
    exit 1
}

# Install
Write-Host "Installing to $installDir..." -ForegroundColor Cyan
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}
Copy-Item $exe.FullName "$installDir\$bin" -Force

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable('Path', "$userPath;$installDir", 'User')
    Write-Host "Added $installDir to user PATH" -ForegroundColor Yellow
    Write-Host "Restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
}

# Cleanup
Remove-Item $zipFile -Force -ErrorAction SilentlyContinue
Remove-Item $extractDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "chesstui $tag installed successfully!" -ForegroundColor Green
Write-Host "Run 'chesstui' to start playing." -ForegroundColor Cyan
