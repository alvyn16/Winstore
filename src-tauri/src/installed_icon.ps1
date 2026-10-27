$ErrorActionPreference = 'SilentlyContinue'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WinstoreIconResources {
  [DllImport("shell32.dll", CharSet = CharSet.Unicode)]
  public static extern int SHDefExtractIconW(string file, int index, uint flags, out IntPtr large, IntPtr small, uint size);
  [DllImport("user32.dll")]
  public static extern bool DestroyIcon(IntPtr icon);
}
'@
$roots = @(
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
  'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall',
  'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall'
)
foreach ($root in $roots) {
  foreach ($entry in (Get-ChildItem -LiteralPath $root)) {
    $app = Get-ItemProperty -LiteralPath $entry.PSPath
    if (![string]::Equals($app.DisplayName, $env:WINSTORE_ICON_NAME, [StringComparison]::OrdinalIgnoreCase)) { continue }
    $resource = [string]$app.DisplayIcon
    if (!$resource) { continue }
    if ($resource -match '^"([^\"]+)"(?:,\s*(-?\d+))?$') { $file = $Matches[1]; $index = $Matches[2] }
    elseif ($resource -match '^(.*?)(?:,\s*(-?\d+))?$') { $file = $Matches[1]; $index = $Matches[2] }
    else { continue }
    $file = [Environment]::ExpandEnvironmentVariables($file.Trim().TrimStart('@'))
    # Only read resources from a local drive. Never invoke the executable.
    if ($file -notmatch '^[A-Za-z]:\\' -or !(Test-Path -LiteralPath $file -PathType Leaf)) { continue }
    $large = [IntPtr]::Zero
    try {
      $result = [WinstoreIconResources]::SHDefExtractIconW($file, [int]$index, 0, [ref]$large, [IntPtr]::Zero, 256)
      if ($result -ne 0 -or $large -eq [IntPtr]::Zero) { continue }
      $icon = [Drawing.Icon]::FromHandle($large)
      $bitmap = $icon.ToBitmap()
      $stream = [IO.MemoryStream]::new()
      try {
        $bitmap.Save($stream, [Drawing.Imaging.ImageFormat]::Png)
        [Convert]::ToBase64String($stream.ToArray())
      } finally { $stream.Dispose(); $bitmap.Dispose(); $icon.Dispose() }
      exit 0
    } finally { if ($large -ne [IntPtr]::Zero) { [void][WinstoreIconResources]::DestroyIcon($large) } }
  }
}
