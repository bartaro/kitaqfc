param(
    [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path,
    [string]$GbCompiler = '',
    [string]$FcCompiler = '',
    [string[]]$Only = @(),
    [string]$Output = (Join-Path $PSScriptRoot 'out')
)
$ErrorActionPreference = 'Stop'
if (!$GbCompiler) { $GbCompiler = Join-Path $Root 'kitaqgb.exe' }
if (!$FcCompiler) { $FcCompiler = Join-Path $Root 'kitaqfc.exe' }
$GbCompiler = [IO.Path]::GetFullPath($GbCompiler)
$FcCompiler = [IO.Path]::GetFullPath($FcCompiler)
$Output = [IO.Path]::GetFullPath($Output)
New-Item -ItemType Directory -Force $Output | Out-Null
$programs = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$failed = @()
# Accept array or comma-separated sample IDs and validate them before building.
$selected = @($Only | ForEach-Object { $_ -split ',' })
$built = 0
foreach ($id in $selected) {
    if ($id -notin $programs.id) { throw ('Unknown sample: '+$id) }
}
# Follow manifest order; known issues are skipped by default but explicitly selected samples are attempted.
foreach ($program in $programs) {
    if ($selected.Count -and $program.id -notin $selected) { continue }
    if (!$Only -and $program.known_issue) { Write-Host ('SKIP '+$program.id+': '+$program.known_issue); continue }
    $targetDir = Join-Path $Output $program.id
    $built++
    New-Item -ItemType Directory -Force $targetDir | Out-Null
    $isGb = $program.platform -eq 'gb'
    $compiler = if ($isGb) { $GbCompiler } else { $FcCompiler }
    $lib = Join-Path $Root $(if ($isGb) { 'lib' } else { 'lib' })
    $compileArgs = @()
    # Compile required library units before the sample so shared declarations are available.
    foreach ($unit in $program.libs) { $compileArgs += Join-Path $lib $unit }
    $compileArgs += Join-Path $PSScriptRoot $program.file
    $compileArgs += @('-I',$lib,'-I',$PSScriptRoot,'--no-disasm')
    $compileArgs += @('-o',(Join-Path $targetDir $(if ($isGb) { 'program.gb' } else { 'program.nes' })))
    if ($isGb) { $compileArgs += @('--profile=dev','--rst-disable','--stack-bank=fixed') }
    # Supply the original sample font as CHR ROM for the NROM cartridge.
    else { $compileArgs += @('--mapper=nrom',('--nes-chr='+(Join-Path $PSScriptRoot 'font.chr'))) }
    $compileArgs += $program.options
    # Isolate per-sample compiler sidecars and always restore the previous working directory.
    Push-Location $targetDir
    try {
        & $compiler @compileArgs *> build.log
        if ($LASTEXITCODE -ne 0) { $failed += $program.id; Write-Host ('FAIL '+$program.id) }
        else { Write-Host ('OK '+$program.id) }
    } finally { Pop-Location }
}
# Attempt the selected set, then return failure if any native compiler invocation failed.
if ($failed.Count) { throw ('Build failures: '+($failed -join ', ')) }
if ($built -eq 0) { throw 'No samples were selected' }
Write-Host ('ROMs and logs: '+$Output)
