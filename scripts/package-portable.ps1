param(
    [string]$ReleaseDirectory = (Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri\target\release')
)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$releaseRoot = (Resolve-Path -LiteralPath $ReleaseDirectory).Path
$executable = Join-Path $releaseRoot 'winstore.exe'
if (!(Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'Build the release executable before packaging the portable ZIP.' }
$config = Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
if ($config.version -notmatch '^\d+\.\d+\.\d+(?:[-+][a-zA-Z0-9.-]+)?$') { throw 'Invalid app version.' }

# Read the actual executable architecture, so packaging needs no Rust toolchain.
$reader = [IO.BinaryReader]::new([IO.File]::OpenRead($executable))
try {
    if ($reader.ReadUInt16() -ne 0x5a4d) { throw 'Invalid Windows executable.' }
    $reader.BaseStream.Position = 0x3c
    $peOffset = $reader.ReadUInt32()
    $reader.BaseStream.Position = $peOffset
    if ($reader.ReadUInt32() -ne 0x4550) { throw 'Invalid PE header.' }
    $architecture = switch ($reader.ReadUInt16()) {
        0x8664 { 'x64' }
        0xaa64 { 'arm64' }
        0x014c { 'x86' }
        default { throw 'Unsupported executable architecture.' }
    }
} finally { $reader.Dispose() }

$packageName = "Winstore_$($config.version)_$architecture-portable"
$outputDirectory = Join-Path $releaseRoot 'bundle\portable'
$stagingDirectory = Join-Path $outputDirectory $packageName
New-Item -ItemType Directory -Path $stagingDirectory -Force | Out-Null
$portableExecutable = Join-Path $stagingDirectory 'Winstore.exe'
Copy-Item -LiteralPath $executable -Destination $portableExecutable -Force
if ((Get-FileHash -LiteralPath $executable).Hash -ne (Get-FileHash -LiteralPath $portableExecutable).Hash) { throw 'Portable executable verification failed.' }
$readme = Join-Path $stagingDirectory 'README.txt'
@"
Winstore $($config.version) - portable Windows $architecture edition

1. Extract the entire ZIP to a writable folder.
2. Run Winstore.exe. No Winstore installation or administrator access is needed.

Requires the Microsoft Edge WebView2 Runtime. This ZIP does not install that
runtime or any package managers. Package actions use your existing Scoop,
Winget, or Chocolatey installation and may request administrator access.

This is an unsigned application, as requested.
Settings, saved apps, WebView data, and task history remain in your Windows
user profile. This is a portable executable, not a self-contained data profile.
Close any other Winstore window before opening this copy.

To update this edition, close Winstore and extract a newer portable ZIP.
Removing this folder removes the portable executable; user-profile data remains.
"@ | Set-Content -LiteralPath $readme -Encoding utf8NoBOM

$archive = Join-Path $outputDirectory "$packageName.zip"
$temporaryArchive = Join-Path $outputDirectory "$packageName.tmp.zip"
Compress-Archive -LiteralPath @($portableExecutable, $readme) -DestinationPath $temporaryArchive -CompressionLevel Optimal -Force
Move-Item -LiteralPath $temporaryArchive -Destination $archive -Force
$checksum = Get-FileHash -LiteralPath $archive -Algorithm SHA256
"$($checksum.Hash.ToLowerInvariant())  $packageName.zip" | Set-Content -LiteralPath "$archive.sha256" -Encoding ascii
Write-Host "Portable ZIP: $archive"
Write-Host "SHA-256: $($checksum.Hash)"
