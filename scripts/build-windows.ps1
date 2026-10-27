param(
    [Uri]$UpdateFeedUrl,
    [Uri]$InstallerBaseUrl
)
$ErrorActionPreference = 'Stop'
foreach ($uri in @($UpdateFeedUrl, $InstallerBaseUrl)) {
    if ($uri -and ($uri.Scheme -ne 'https' -or $uri.UserInfo)) { throw 'Release URLs must use HTTPS without credentials.' }
}
$projectRoot = Split-Path -Parent $PSScriptRoot
$previousFeed = $env:WINSTORE_UPDATE_FEED_URL
Push-Location -LiteralPath $projectRoot
try {
    if ($UpdateFeedUrl) { $env:WINSTORE_UPDATE_FEED_URL = $UpdateFeedUrl.AbsoluteUri }
    else { Remove-Item Env:\WINSTORE_UPDATE_FEED_URL -ErrorAction SilentlyContinue }
    npm run tauri build -- --bundles nsis
    if ($LASTEXITCODE -ne 0) { throw 'Winstore installer build failed.' }
    $installer = Get-ChildItem -LiteralPath (Join-Path $projectRoot 'src-tauri\target\release\bundle\nsis') -Filter '*-setup.exe' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (!$installer) { throw 'The NSIS installer was not produced.' }
    & (Join-Path $PSScriptRoot 'package-portable.ps1')
    Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256 | Format-List
    if ($InstallerBaseUrl) {
        $config = Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
        $hostLine = rustc -vV | Select-String '^host: '
        $architecture = (($hostLine.ToString() -replace '^host: ', '') -split '-')[0]
        $platforms = @{}
        $platforms["windows-$architecture"] = @{ url = $InstallerBaseUrl.AbsoluteUri.TrimEnd('/') + '/' + [Uri]::EscapeDataString($installer.Name) }
        $feed = @{ version = $config.version; notes = ''; platforms = $platforms }
        $feed | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $installer.Directory.FullName 'latest.json') -Encoding utf8NoBOM
    }
    Write-Host "Unsigned installer: $($installer.FullName)"
} finally {
    if ($null -eq $previousFeed) { Remove-Item Env:\WINSTORE_UPDATE_FEED_URL -ErrorAction SilentlyContinue } else { $env:WINSTORE_UPDATE_FEED_URL = $previousFeed }
    Pop-Location
}
