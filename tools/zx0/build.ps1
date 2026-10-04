$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
& cargo build --locked --release --manifest-path (Join-Path $projectRoot 'Cargo.toml') --bin kitaqfc-zx0
if ($LASTEXITCODE -ne 0) { throw 'ZX0 native build failed.' }
