<#
.SYNOPSIS
  Build the COD installer locally.
.DESCRIPTION
  Compiles Rust native modules, builds the COD client, and packages
  the installer using InnoSetup.
.PARAMETER SkipDeps
  Skip npm ci (use if deps are already installed).
.EXAMPLE
  .\scripts\build-installer.ps1
  .\scripts\build-installer.ps1 -SkipDeps
#>
param([switch]$SkipDeps)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host ("=" * 47)
Write-Host "  COD Installer Builder"
Write-Host ("=" * 47)
Write-Host ""

# Check prerequisites
$missing = @()
if (-not (Get-Command node -ErrorAction SilentlyContinue)) { $missing += "Node.js" }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { $missing += "Rust/Cargo" }
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) { $missing += "npm" }
if ($missing.Count -gt 0) {
    Write-Host "[ERROR] Missing: $($missing -join ', ')" -ForegroundColor Red
    exit 1
}

$version = (Get-Content package.json -Raw | ConvertFrom-Json).version
Write-Host "Version: $version"
Write-Host ""

# Step 1
if (-not $SkipDeps) {
    Write-Host "[1/5] Installing npm dependencies..." -ForegroundColor Cyan
    npm ci
    if ($LASTEXITCODE -ne 0) { throw "npm ci failed" }
    Write-Host "  Done." -ForegroundColor Green
} else {
    Write-Host "[1/5] Skipping npm ci (-SkipDeps)" -ForegroundColor Yellow
}

# Step 2
Write-Host "[2/5] Compiling Rust native module..." -ForegroundColor Cyan
npm run gulp compile-native
if ($LASTEXITCODE -ne 0) { throw "Rust compilation failed" }
Write-Host "  Done." -ForegroundColor Green

# Step 3
Write-Host "[3/5] Building COD client (this takes a while)..." -ForegroundColor Cyan
npm run gulp vscode-win32-x64-min
if ($LASTEXITCODE -ne 0) { throw "COD build failed" }
Write-Host "  Done." -ForegroundColor Green

# Step 4
Write-Host "[4/5] Building InnoSetup updater..." -ForegroundColor Cyan
npm run gulp vscode-win32-x64-inno-updater
if ($LASTEXITCODE -ne 0) { Write-Host "  [WARN] Updater build had issues" -ForegroundColor Yellow }

# Step 5
Write-Host "[5/5] Building installer..." -ForegroundColor Cyan
npm run gulp vscode-win32-x64-user-setup
if ($LASTEXITCODE -ne 0) { throw "Installer build failed" }

npm run gulp vscode-win32-x64-system-setup
if ($LASTEXITCODE -ne 0) { Write-Host "  [WARN] System installer had issues" -ForegroundColor Yellow }

Write-Host ""
Write-Host ("=" * 47)
Write-Host "  Build Complete"
Write-Host ("=" * 47)

# Copy outputs to repo root
$userSetup = ".build/win32-x64/user-setup/VSCodeSetup.exe"
$sysSetup  = ".build/win32-x64/system-setup/VSCodeSetup.exe"

if (Test-Path $userSetup) {
    Copy-Item $userSetup "CODUserSetup-x64-$version.exe"
    Write-Host "  [USER]   CODUserSetup-x64-$version.exe" -ForegroundColor Green
} else {
    Write-Host "  [WARN] User installer not found at $userSetup" -ForegroundColor Yellow
}
if (Test-Path $sysSetup) {
    Copy-Item $sysSetup "CODSetup-x64-$version.exe"
    Write-Host "  [SYSTEM] CODSetup-x64-$version.exe" -ForegroundColor Green
} else {
    Write-Host "  [WARN] System installer not found at $sysSetup" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Installers are in: $RepoRoot"
Write-Host ""
