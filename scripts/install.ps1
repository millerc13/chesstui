# chesstui installer for Windows
# Usage:  irm https://raw.githubusercontent.com/cjmiller/chesstui/main/scripts/install.ps1 | iex
#
# Or pin a version:
#   $env:CHESSTUI_VERSION = "v0.2.0"; irm ... | iex

$ErrorActionPreference = "Stop"

$repo    = "cjmiller/chesstui"
$binary  = "chesstui"
$target  = "x86_64-pc-windows-msvc"

# ---------------------------------------------------------------------------
# Resolve version
# ---------------------------------------------------------------------------
$version = $env:CHESSTUI_VERSION
if (-not $version) {
    $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $version = $release.tag_name
}
Write-Host "Installing $binary $version for $target" -ForegroundColor Cyan

# ---------------------------------------------------------------------------
# Download and extract
# ---------------------------------------------------------------------------
$archiveName = "$binary-$version-$target.zip"
$url = "https://github.com/$repo/releases/download/$version/$archiveName"
$checksumsUrl = "https://github.com/$repo/releases/download/$version/SHA256SUMS.txt"

$tmpDir = Join-Path $env:TEMP "chesstui-install-$(Get-Random)"
New-Item -ItemType Directory -Path $tmpDir | Out-Null

try {
    Write-Host "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile (Join-Path $tmpDir $archiveName)

    # Verify checksum
    Write-Host "Verifying checksum..."
    $checksums = Invoke-RestMethod -Uri $checksumsUrl
    $expectedHash = ($checksums -split "`n" | Where-Object { $_ -match $archiveName } | ForEach-Object { ($_ -split "\s+")[0] })
    $actualHash = (Get-FileHash (Join-Path $tmpDir $archiveName) -Algorithm SHA256).Hash.ToLower()
    if ($expectedHash -and $actualHash -ne $expectedHash) {
        throw "Checksum mismatch! Expected: $expectedHash Got: $actualHash"
    }
    Write-Host "Checksum OK" -ForegroundColor Green

    Write-Host "Extracting..."
    Expand-Archive -Path (Join-Path $tmpDir $archiveName) -DestinationPath $tmpDir

    # Install to user-local bin directory
    $installDir = Join-Path $env:LOCALAPPDATA "chesstui\bin"
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir | Out-Null
    }

    Copy-Item (Join-Path $tmpDir "$binary-$version-$target\$binary.exe") (Join-Path $installDir "$binary.exe") -Force

    # Add to PATH if not already present
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$installDir;$userPath", "User")
        Write-Host "Added $installDir to your PATH (restart your terminal to use)" -ForegroundColor Yellow
    }

    Write-Host "Installed $binary $version to $installDir\$binary.exe" -ForegroundColor Green
    Write-Host "Run '$binary --help' to get started"
}
finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}
