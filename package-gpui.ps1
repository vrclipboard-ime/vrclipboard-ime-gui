[CmdletBinding()]
param(
    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Release',
    [string]$OutputDirectory = 'dist-gpui'
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath $PSScriptRoot).Path
$profile = if ($Configuration -eq 'Release') { 'release' } else { 'debug' }

Push-Location $repoRoot
try {
    $cargoArgs = @('build', '--package', 'vrclipboard-ime-gui')
    if ($Configuration -eq 'Release') {
        $cargoArgs += '--release'
    }
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build exited with $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

$outputPath = if ([System.IO.Path]::IsPathRooted($OutputDirectory)) {
    $OutputDirectory
} else {
    Join-Path $repoRoot $OutputDirectory
}
$output = [System.IO.Path]::GetFullPath($outputPath)
$outputPrefix = $repoRoot.TrimEnd(
    [System.IO.Path]::DirectorySeparatorChar,
    [System.IO.Path]::AltDirectorySeparatorChar
) + [System.IO.Path]::DirectorySeparatorChar
if (-not $output.StartsWith($outputPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Invalid package output path: $output"
}

if (Test-Path -LiteralPath $output) {
    Remove-Item -LiteralPath $output -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $output | Out-Null

$binary = Join-Path $repoRoot "target\$profile\vrclipboard-ime-gui.exe"
$resources = Join-Path $repoRoot 'src-tauri\resources'
Copy-Item -LiteralPath $binary -Destination $output
Copy-Item -LiteralPath (Join-Path $resources 'ggml-model-Q5_K_M.gguf') -Destination $output
Copy-Item -LiteralPath (Join-Path $resources 'azookey-native') -Destination $output -Recurse
# SwiftPM Bundle.module uses Bundle.main (the EXE directory), even when the
# bridge DLL lives in a subdirectory. Keep every resource bundle beside the EXE.
Get-ChildItem -LiteralPath (Join-Path $resources 'azookey-native') -Directory -Filter '*.resources' |
    Copy-Item -Destination $output -Recurse
Copy-Item -LiteralPath (Join-Path $repoRoot 'LICENSE') -Destination $output

Write-Host "GPUI package created: $output"
