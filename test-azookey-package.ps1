[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ZipPath
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath $PSScriptRoot).Path
$checkRoot = Join-Path $repoRoot 'target/azookey-package-check'
# Generate Unicode through code points so Windows PowerShell 5 also handles
# this script correctly without requiring a particular source-file encoding.
$unicodeName = [string][char]0x65e5 + [char]0x672c + [char]0x8a9e
$extractRoot = Join-Path $checkRoot ($unicodeName + ' ZIP ' + [Guid]::NewGuid().ToString('N'))
Expand-Archive -LiteralPath (Resolve-Path -LiteralPath $ZipPath).Path -DestinationPath $extractRoot
$fixturePrefix = [System.IO.Path]::GetFullPath($extractRoot) + [System.IO.Path]::DirectorySeparatorChar

function Invoke-AzookeyProbe([string]$Label) {
    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = Join-Path $extractRoot 'vrclipboard-ime-gui.exe'
    $dataDirectory = Join-Path $checkRoot ($unicodeName + ' data ' + $Label)
    $startInfo.Arguments = '--check-azookey "' + $extractRoot + '" "' + $dataDirectory + '"'
    $startInfo.WorkingDirectory = $checkRoot
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Hidden
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    # The package must not depend on a developer's Swift/toolchain PATH.
    $startInfo.EnvironmentVariables['PATH'] = "$env:SystemRoot\System32;$env:SystemRoot"
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    try {
        [void]$process.Start()
        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(30000)) {
            $process.Kill()
            throw "AzooKey probe timed out: $Label"
        }
        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        Set-Content -LiteralPath (Join-Path $extractRoot "$Label-stdout.txt") -Value $stdout -Encoding UTF8
        Set-Content -LiteralPath (Join-Path $extractRoot "$Label-stderr.txt") -Value $stderr -Encoding UTF8
        if ($process.ExitCode -ne 0) {
            throw "AzooKey probe exited with $($process.ExitCode): $Label. Logs: $extractRoot"
        }
        if ($stdout -notmatch 'AzooKey conversion OK:' -or $stdout -notmatch 'AzooKey reconversion OK:') {
            throw "AzooKey conversion results are missing: $Label"
        }
        if ($Label -eq 'generic-old-layout' -and $stderr -notmatch 'ggml-cpu-generic.dll') {
            throw 'The generic CPU backend was not exercised'
        }
        Write-Host "$Label : $($stdout.Trim())"
    }
    finally {
        $process.Dispose()
    }
}

Invoke-AzookeyProbe 'unicode-clean-path'
# Modify only this disposable extracted fixture to exercise automatic resource
# placement and the same AzooKey converter with the generic CPU backend.
foreach ($bundle in Get-ChildItem -LiteralPath $extractRoot -Directory -Filter '*.resources') {
    if (-not $bundle.FullName.StartsWith($fixturePrefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Unexpected fixture resource path: $($bundle.FullName)"
    }
    Remove-Item -LiteralPath $bundle.FullName -Recurse -Force
}
$nativeRoot = Join-Path $extractRoot 'azookey-native'
foreach ($variant in Get-ChildItem -LiteralPath $nativeRoot -File -Filter 'ggml-cpu-*.dll') {
    if ($variant.Name -ne 'ggml-cpu-generic.dll') {
        if (-not $variant.FullName.StartsWith($fixturePrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Unexpected fixture backend path: $($variant.FullName)"
        }
        Remove-Item -LiteralPath $variant.FullName -Force
    }
}
Invoke-AzookeyProbe 'generic-old-layout'
Write-Host "AzooKey package checks passed. Logs: $extractRoot"
