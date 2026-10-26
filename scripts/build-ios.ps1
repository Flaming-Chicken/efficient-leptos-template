#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Builds efficient-leptos-template Tauri v2 library for iOS.
.DESCRIPTION
    Compiles crates/desktop as staticlib/cdylib for physical iOS devices (aarch64-apple-ios)
    and iOS simulators (aarch64-apple-ios-sim, x86_64-apple-ios) for linking into Xcode or Tauri iOS runner.
.PARAMETER OutputDir
    Destination directory for compiled iOS binaries. Defaults to "target/ios".
.EXAMPLE
    ./scripts/build-ios.ps1
#>
param(
    [string]$OutputDir = "target/ios"
)

$ErrorActionPreference = "Stop"

Write-Host "============================================================"
Write-Host " Building efficient-leptos-template Tauri v2 for iOS"
Write-Host "============================================================"

$targets = @("aarch64-apple-ios", "aarch64-apple-ios-sim", "x86_64-apple-ios")

$installedTargets = rustup target list --installed
foreach ($target in $targets) {
    if ($installedTargets -notcontains $target) {
        Write-Host "[*] Installing missing target: $target..."
        rustup target add $target
    }
}

New-Item -ItemType Directory -Force -Path "$OutputDir/device" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutputDir/simulator" | Out-Null

foreach ($target in $targets) {
    Write-Host "[*] Compiling crates/desktop for $target..."
    cargo build --package desktop --lib --release --target $target
}

Copy-Item -Force "target/aarch64-apple-ios/release/libdesktop_lib.a" "$OutputDir/device/libLeptosDesktop.a" -ErrorAction SilentlyContinue

if (Get-Command lipo -ErrorAction SilentlyContinue) {
    Write-Host "[*] Assembling universal simulator binary with lipo..."
    lipo -create `
        "target/aarch64-apple-ios-sim/release/libdesktop_lib.a" `
        "target/x86_64-apple-ios/release/libdesktop_lib.a" `
        -output "$OutputDir/simulator/libLeptosDesktop.a"
} else {
    Write-Host "[i] lipo not available on this host. Preserving individual simulator slices."
    Copy-Item -Force "target/aarch64-apple-ios-sim/release/libdesktop_lib.a" "$OutputDir/simulator/libLeptosDesktop_arm64.a" -ErrorAction SilentlyContinue
    Copy-Item -Force "target/x86_64-apple-ios/release/libdesktop_lib.a" "$OutputDir/simulator/libLeptosDesktop_x86_64.a" -ErrorAction SilentlyContinue
}

if (Get-Command xcodebuild -ErrorAction SilentlyContinue) {
    Write-Host "[*] Generating LeptosDesktop.xcframework via xcodebuild..."
    $xcframeworkPath = "$OutputDir/LeptosDesktop.xcframework"
    if (Test-Path $xcframeworkPath) {
        Remove-Item -Recurse -Force $xcframeworkPath
    }
    xcodebuild -create-xcframework `
        -library "$OutputDir/device/libLeptosDesktop.a" `
        -library "$OutputDir/simulator/libLeptosDesktop.a" `
        -output $xcframeworkPath
    Write-Host "[+] Generated $xcframeworkPath"
} else {
    Write-Host "[i] xcodebuild not available on this host. Static libraries ready for Xcode import."
}

Write-Host "[+] iOS build completed successfully! Output: $OutputDir"
