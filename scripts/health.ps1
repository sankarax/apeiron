$ErrorActionPreference = 'Stop'
$checks = [ordered]@{}
try { $checks.backend = (Invoke-RestMethod http://127.0.0.1:8000/api/health -TimeoutSec 5).status -eq 'healthy' } catch { $checks.backend = $false }
try { $archive = Invoke-RestMethod http://127.0.0.1:8000/api/archive/health -TimeoutSec 5; $checks.postgres = $archive.postgres; $checks.redis = $archive.redis } catch { $checks.postgres = $false; $checks.redis = $false }
try { $models = Invoke-RestMethod http://127.0.0.1:11434/api/tags -TimeoutSec 5; $checks.ollama = $true; $checks.model = 'qwen3:8b' -in $models.models.name } catch { $checks.ollama = $false; $checks.model = $false }
try { $checks.frontend = (Invoke-WebRequest http://127.0.0.1:5173 -UseBasicParsing -TimeoutSec 5).StatusCode -eq 200 } catch { $checks.frontend = $false }
[pscustomobject]$checks | Format-List
if ($checks.Values -contains $false) { exit 1 }
