$root = Split-Path -Parent $PSScriptRoot

# stop frontend and backend
foreach ($port in @(5173, 8000)) {
    $processes = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue |
        Select-Object -ExpandProperty OwningProcess -Unique

    foreach ($processId in $processes) {
        taskkill /PID $processId /T /F
    }
}

# stop ollama
Get-Process -Name "ollama*" -ErrorAction SilentlyContinue |
    Stop-Process -Force

# stop database containers
Push-Location $root
docker compose stop
Pop-Location

Write-Host "Apeiron services stopped." -ForegroundColor Green