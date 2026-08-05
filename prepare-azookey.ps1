[CmdletBinding()]
param(
    [string]$AzookeyRoot = (Join-Path $PSScriptRoot '..\azookey-kkc-rs'),
    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Release',
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath $PSScriptRoot).Path
$azookeyRootPath = (Resolve-Path -LiteralPath $AzookeyRoot).Path
$resourceRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot 'src-tauri\resources'))
$destination = [System.IO.Path]::GetFullPath((Join-Path $resourceRoot 'azookey-native'))
$resourcePrefix = $resourceRoot.TrimEnd(
    [System.IO.Path]::DirectorySeparatorChar,
    [System.IO.Path]::AltDirectorySeparatorChar
) + [System.IO.Path]::DirectorySeparatorChar
if (-not $destination.StartsWith($resourcePrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Invalid native resource destination: $destination"
}

if (-not $SkipBuild) {
    & (Join-Path $azookeyRootPath 'setup-windows.ps1') `
        -Build -Configuration $Configuration
    if ($LASTEXITCODE -ne 0) {
        throw "azookey-kkc native build exited with $LASTEXITCODE"
    }
}

$source = Join-Path $azookeyRootPath 'native\dist'
if (-not (Test-Path -LiteralPath (Join-Path $source 'azk_bridge.dll') -PathType Leaf)) {
    throw "azookey-kkc native artifacts were not found under $source"
}

if (Test-Path -LiteralPath $destination) {
    Remove-Item -LiteralPath $destination -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $destination | Out-Null
Copy-Item -Path (Join-Path $source '*') -Destination $destination -Recurse -Force
Get-ChildItem -LiteralPath $destination -Recurse -Force | ForEach-Object {
    if ($_.Attributes -band [System.IO.FileAttributes]::ReadOnly) {
        $_.Attributes = $_.Attributes -band (-bnot [System.IO.FileAttributes]::ReadOnly)
    }
}

Write-Host "Staged azookey-kkc native resources: $destination"
