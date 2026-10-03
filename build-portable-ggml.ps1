[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$LlamaSourceRoot,
    [string]$DestinationDirectory = 'src-tauri/resources/azookey-native',
    [ValidateSet('Debug', 'Release')]
    [string]$Configuration = 'Release'
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath $PSScriptRoot).Path
$sourceRoot = (Resolve-Path -LiteralPath $LlamaSourceRoot).Path
$expectedRevision = '6c595e43c4a1dd71bf12ddfb48a62b235d137567'
$revision = & git -c "safe.directory=$sourceRoot" -C $sourceRoot rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $revision -ne $expectedRevision) {
    throw "Expected llama.cpp revision $expectedRevision, got $revision. Review the UTF-8 patch before updating the native dependency."
}

# Work on an archived snapshot; never modify the external AzooKey checkout.
$scratch = Join-Path $repoRoot 'target/azookey-ggml-portable'
$source = Join-Path $scratch 'llama.cpp'
$build = Join-Path $scratch 'build'
$archive = Join-Path $scratch 'llama-source.zip'
$destinationPath = if ([System.IO.Path]::IsPathRooted($DestinationDirectory)) {
    $DestinationDirectory
} else {
    Join-Path $repoRoot $DestinationDirectory
}
$destination = [System.IO.Path]::GetFullPath($destinationPath)
$repoPrefix = $repoRoot.TrimEnd([char[]]'\/') + [System.IO.Path]::DirectorySeparatorChar
if (-not $destination.StartsWith($repoPrefix, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Native output must stay inside the workspace: $destination"
}
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
& git -c "safe.directory=$sourceRoot" -C $sourceRoot archive --format=zip "--output=$archive" $expectedRevision
if ($LASTEXITCODE -ne 0) { throw 'Cannot archive the pinned llama.cpp source' }
Expand-Archive -LiteralPath $archive -DestinationPath $source -Force
$patch = Join-Path $repoRoot 'native-patches/ggml-windows-utf8.patch'
# Run from the repository root with an explicit prefix. From a subdirectory,
# git apply can silently skip a patch whose paths are outside that subdirectory.
$patchDirectory = $source.Substring($repoPrefix.Length).Replace('\', '/')
& git -C $repoRoot apply --check "--directory=$patchDirectory" $patch
if ($LASTEXITCODE -ne 0) { throw 'UTF-8 patch does not match the pinned source' }
& git -C $repoRoot apply "--directory=$patchDirectory" $patch
if ($LASTEXITCODE -ne 0) { throw 'Cannot apply the UTF-8 patch' }

& cmake -S $source -B $build -G 'Visual Studio 17 2022' -A x64 `
    -DBUILD_SHARED_LIBS=ON -DGGML_NATIVE=OFF -DGGML_BACKEND_DL=ON `
    -DGGML_CPU_ALL_VARIANTS=ON -DGGML_OPENMP=OFF -DGGML_VULKAN=OFF `
    -DLLAMA_BUILD_TESTS=OFF -DLLAMA_BUILD_EXAMPLES=OFF -DLLAMA_BUILD_TOOLS=OFF -DLLAMA_CURL=OFF
if ($LASTEXITCODE -ne 0) { throw 'Portable ggml CMake configuration failed' }
& cmake --build $build --config $Configuration --target ggml --parallel 4
if ($LASTEXITCODE -ne 0) { throw 'Portable ggml build failed' }
New-Item -ItemType Directory -Force -Path $destination | Out-Null
# Preserve the existing ABI-compatible llama, base and CPU/Vulkan backend DLLs.
Copy-Item -LiteralPath (Join-Path $build "bin/$Configuration/ggml.dll") -Destination $destination -Force
Write-Host "Staged UTF-8-compatible ggml.dll: $destination"
