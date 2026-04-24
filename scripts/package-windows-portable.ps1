param(
    [string]$ProjectRoot = (Join-Path $PSScriptRoot ".."),
    [string]$OutputDir = "dist/portable",
    [string]$SourceExe = "src-tauri/target/release/member-club.exe"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-SafeFileName {
    param([Parameter(Mandatory = $true)][string]$Name)

    $safeName = $Name
    foreach ($char in [System.IO.Path]::GetInvalidFileNameChars()) {
        $safeName = $safeName.Replace($char, "_")
    }

    return $safeName.Trim()
}

$resolvedProjectRoot = (Resolve-Path -LiteralPath $ProjectRoot).Path
$packageJsonPath = Join-Path $resolvedProjectRoot "package.json"
$packageJson = Get-Content -LiteralPath $packageJsonPath -Raw -Encoding UTF8 | ConvertFrom-Json
$version = if ([string]::IsNullOrWhiteSpace($packageJson.version)) {
    "0.0.0"
} else {
    [string]$packageJson.version
}

$resolvedSourceExe = [System.IO.Path]::GetFullPath((Join-Path $resolvedProjectRoot $SourceExe))
if (-not (Test-Path -LiteralPath $resolvedSourceExe)) {
    throw "Release executable not found: $resolvedSourceExe"
}

$resolvedOutputDir = [System.IO.Path]::GetFullPath((Join-Path $resolvedProjectRoot $OutputDir))
New-Item -ItemType Directory -Path $resolvedOutputDir -Force | Out-Null

$sourceExeName = [System.IO.Path]::GetFileName($resolvedSourceExe)
$sourceExeBaseName = [System.IO.Path]::GetFileNameWithoutExtension($resolvedSourceExe)
$archiveBaseName = "{0}_{1}_windows_x64_portable" -f (Get-SafeFileName -Name $sourceExeBaseName), $version
$packageDir = Join-Path $resolvedOutputDir $archiveBaseName
$zipPath = Join-Path $resolvedOutputDir ($archiveBaseName + ".zip")

if (Test-Path -LiteralPath $packageDir) {
    Remove-Item -LiteralPath $packageDir -Recurse -Force
}
if (Test-Path -LiteralPath $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}

New-Item -ItemType Directory -Path $packageDir -Force | Out-Null
Copy-Item -LiteralPath $resolvedSourceExe -Destination (Join-Path $packageDir $sourceExeName)

$readmePath = Join-Path $packageDir "README.txt"
$readmeContent = @(
    "Portable Windows package",
    "",
    "1. Extract this archive to any folder.",
    "2. Run $sourceExeName from the extracted folder.",
    "3. On first launch the app will create clean data, config, and logs directories beside the executable.",
    "4. This package does not include existing runtime data, config, or logs from the build machine.",
    "5. To migrate records from another machine, copy the old data directory into this folder after extraction.",
    "6. To migrate settings as well, copy the old config directory into this folder.",
    "7. Microsoft Edge WebView2 Runtime is required on the target machine."
) -join [Environment]::NewLine
Set-Content -LiteralPath $readmePath -Value $readmeContent -Encoding UTF8

Compress-Archive -LiteralPath $packageDir -DestinationPath $zipPath -CompressionLevel Optimal

Write-Host ("Portable directory: " + $packageDir)
Write-Host ("Portable archive: " + $zipPath)
