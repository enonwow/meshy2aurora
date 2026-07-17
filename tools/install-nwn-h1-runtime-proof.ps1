[CmdletBinding()]
param(
    [ValidateSet('Plan', 'Install')]
    [string]$Mode = 'Plan',

    [string]$PacketRoot = 'C:\Projects\meshy2aurora\proof-output\meshy-h1-codex-animation-proof-v2',

    [string]$HakDirectory = 'C:\Users\enonw\Documents\Neverwinter Nights\hak',

    [string]$ModuleDirectory = 'C:\Users\enonw\Documents\Neverwinter Nights\modules'
)

$ErrorActionPreference = 'Stop'

$expected = @(
    [pscustomobject]@{
        Kind = 'HAK'
        Source = (Join-Path $PacketRoot 'generated\m2a_codex_aproof.hak')
        Destination = (Join-Path $HakDirectory 'm2a_codex_aproof.hak')
        Sha256 = 'da4cde270a2ab7fb24a3d570f55868583a029b4226ecc260bcf39d11704a7756'
    },
    [pscustomobject]@{
        Kind = 'module'
        Source = (Join-Path $PacketRoot 'generated\m2a_codex_aproof.mod')
        Destination = (Join-Path $ModuleDirectory 'm2a_codex_aproof.mod')
        Sha256 = '5c35c82bc37f451d2703c4bab20104e812b7e073c50b2aa5f0d8068aa6ca2e87'
    }
)

function Get-Sha256([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

Write-Host "Native H1 runtime-proof installer: $Mode"
Write-Host 'This script never changes the game installation. It only targets the user hak/modules folders.'

foreach ($item in $expected) {
    if (-not (Test-Path -LiteralPath $item.Source -PathType Leaf)) {
        throw "Missing packet $($item.Kind): $($item.Source)"
    }

    $actual = Get-Sha256 $item.Source
    if ($actual -ne $item.Sha256) {
        throw "Packet hash mismatch for $($item.Source). Expected $($item.Sha256), got $actual."
    }

    $destinationDirectory = Split-Path -Parent $item.Destination
    if (-not (Test-Path -LiteralPath $destinationDirectory -PathType Container)) {
        throw "Missing destination directory for $($item.Kind): $destinationDirectory"
    }

    $targetState = if (Test-Path -LiteralPath $item.Destination -PathType Leaf) {
        "EXISTS sha256=$(Get-Sha256 $item.Destination)"
    } else {
        'ABSENT'
    }
    Write-Host "[$($item.Kind)]"
    Write-Host "  source:      $($item.Source)"
    Write-Host "  source hash: $actual (verified)"
    Write-Host "  target:      $($item.Destination)"
    Write-Host "  target:      $targetState"
}

if ($Mode -eq 'Plan') {
    Write-Host 'PLAN ONLY: no files were copied. Re-run with -Mode Install only after explicit approval.'
    exit 0
}

foreach ($item in $expected) {
    if (Test-Path -LiteralPath $item.Destination -PathType Leaf) {
        throw "Refusing to overwrite existing target: $($item.Destination)"
    }
}

foreach ($item in $expected) {
    # File.Copy(..., $false) is a second no-overwrite guard against races.
    [System.IO.File]::Copy($item.Source, $item.Destination, $false)
    $installedHash = Get-Sha256 $item.Destination
    if ($installedHash -ne $item.Sha256) {
        throw "Installed hash mismatch for $($item.Destination). Expected $($item.Sha256), got $installedHash."
    }
    Write-Host "INSTALLED and verified [$($item.Kind)]: $($item.Destination)"
}

Write-Host 'Install succeeded. Next manual proof: open m2a_codex_aproof.mod in Aurora Toolset, enter the area, and capture a visible animation transition.'
