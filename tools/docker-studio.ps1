[CmdletBinding()]
param(
    [ValidateSet('Start', 'Stop', 'Restart', 'Status', 'Logs', 'Build', 'RebuildWasm', 'StartMeshyBridge', 'MeshyBridgeLogs', 'Verify')]
    [string]$Action = 'Start'
)

$ErrorActionPreference = 'Stop'

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$workspaceGuard = Join-Path $repositoryRoot 'assert-canonical-workspace.ps1'
$composeFile = Join-Path $repositoryRoot 'docker-compose.yml'

& $workspaceGuard
if ($LASTEXITCODE -ne 0) {
    throw 'The canonical workspace guard failed.'
}

& docker compose version | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw 'Docker Compose v2 is required. Install Docker Desktop and retry.'
}

function Invoke-Compose([string[]]$Arguments) {
    & docker compose -f $composeFile @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "docker compose failed while running: $($Arguments -join ' ')"
    }
}

function Assert-StudioPortAvailable {
    $configuredPort = if ([string]::IsNullOrWhiteSpace($env:M2A_STUDIO_PORT)) {
        5174
    } else {
        $env:M2A_STUDIO_PORT
    }
    $port = 0
    if (-not [int]::TryParse($configuredPort, [ref]$port) -or $port -lt 1 -or $port -gt 65535) {
        throw 'M2A_STUDIO_PORT must be an integer from 1 through 65535.'
    }
    $runningService = (& docker compose -f $composeFile ps --status running -q studio | Out-String).Trim()
    if ($runningService) {
        return
    }
    $listener = Get-NetTCPConnection -State Listen -LocalPort $port -ErrorAction SilentlyContinue |
        Where-Object { $_.LocalAddress -in @('0.0.0.0', '127.0.0.1', '::', '::1') } |
        Select-Object -First 1
    if ($listener) {
        throw "Studio cannot bind localhost:$port because PID $($listener.OwningProcess) already listens there. Stop that process or set M2A_STUDIO_PORT to a free port."
    }
}

function Assert-MeshyApiKeyPresent {
    if ([string]::IsNullOrWhiteSpace($env:MESHY_API_KEY)) {
        throw 'Set MESHY_API_KEY in this PowerShell session before starting the Meshy Bridge. The key is never written to the repository or Compose file.'
    }
}

switch ($Action) {
    'Start' {
        Assert-StudioPortAvailable
        Invoke-Compose @('up', '--detach', '--build', 'studio')
    }
    'Stop' { Invoke-Compose @('stop', 'studio') }
    'Restart' { Invoke-Compose @('restart', 'studio') }
    'Status' { Invoke-Compose @('ps') }
    'Logs' { Invoke-Compose @('logs', '--tail', '200', 'studio') }
    'Build' { Invoke-Compose @('build', 'studio') }
    'RebuildWasm' { Invoke-Compose @('exec', 'studio', 'npm', '--prefix', 'apps/studio-web', 'run', 'build:wasm') }
    'StartMeshyBridge' {
        Assert-MeshyApiKeyPresent
        Invoke-Compose @('--profile', 'meshy', 'up', '--detach', '--build', 'meshy-bridge')
    }
    'MeshyBridgeLogs' { Invoke-Compose @('--profile', 'meshy', 'logs', '--tail', '80', 'meshy-bridge') }
    'Verify' { Invoke-Compose @('--profile', 'quality', 'build', 'quality') }
}
