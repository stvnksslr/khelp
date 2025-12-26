#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

# Set installation directory
$InstallDir = "$env:LOCALAPPDATA\Programs\khelp"

# Create install directory if it doesn't exist
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Check if directory is in PATH
$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to PATH..."
    [Environment]::SetEnvironmentVariable('Path', "$UserPath;$InstallDir", 'User')
    $env:Path = "$env:Path;$InstallDir"
}

# Detect system architecture
$Arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    'X64' { 'x86_64-pc-windows-gnu' }
    default {
        Write-Error "Unsupported architecture: $_"
        exit 1
    }
}

# Create temporary directory for download
$TmpDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString()))
try {
    # Get latest release and download URL
    $Release = Invoke-RestMethod -Uri 'https://api.github.com/repos/stvnksslr/khelp/releases/latest'
    $Asset = $Release.assets | Where-Object { $_.name -eq "khelp-$Arch.zip" }

    if (-not $Asset) {
        Write-Error "Could not find release for $Arch"
        exit 1
    }

    # Download and extract
    $ZipPath = Join-Path $TmpDir 'khelp.zip'
    Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ZipPath
    Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

    # Install binary
    Move-Item -Path (Join-Path $TmpDir 'khelp.exe') -Destination $InstallDir -Force
    Write-Host "Installed khelp to $InstallDir"
}
finally {
    # Cleanup temp directory
    Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
