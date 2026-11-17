$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location (Join-Path $root 'engine')
try {
    wasm-pack build --target web --release --out-dir pkg
    if ($LASTEXITCODE -ne 0) { throw 'WASM build failed' }
    $dest = Join-Path $root 'frontend/wasm'
    New-Item -ItemType Directory -Force $dest | Out-Null
    Copy-Item pkg/apeiron_engine* $dest -Force
} finally { Pop-Location }
