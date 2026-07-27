[CmdletBinding()]
param(
    [ValidateSet('Plan', 'Install')]
    [string]$Mode = 'Plan',

    [string]$PacketRoot = 'C:\Projects\meshy2aurora\proof-output\m0-meshy-golem-20260718',

    [string]$HakDirectory = 'C:\Users\enonw\Documents\Neverwinter Nights\hak',

    [string]$ModuleDirectory = 'C:\Users\enonw\Documents\Neverwinter Nights\modules',

    [string]$ReferenceHakSource = 'C:\Users\enonw\Downloads\znd_tortoise\hak\ZND_Tortoise.hak'
)

$ErrorActionPreference = 'Stop'

$referenceHakDestination = Join-Path $HakDirectory 'znd_tortoise.hak'
$expected = @(
    [pscustomobject]@{
        Kind = 'reference HAK (read-only dependency)'
        Source = $ReferenceHakSource
        Destination = $referenceHakDestination
        Sha256 = '3f85946f1e86e20b4d9c6fae311f214587da4009080b64376711f630b0327021'
        Install = $false
    },
    [pscustomobject]@{
        Kind = 'M0 HAK'
        Source = (Join-Path $PacketRoot 'generated\m2a_m0_proof.hak')
        Destination = (Join-Path $HakDirectory 'm2a_m0_proof.hak')
        Sha256 = '2eb0c336c9ba3de7a40e10b653f5e55a43d88ae5b18d8745a80006fac3623294'
        Install = $true
    },
    [pscustomobject]@{
        Kind = 'M0 module'
        Source = (Join-Path $PacketRoot 'generated\m2a_m0_proof.mod')
        Destination = (Join-Path $ModuleDirectory 'm2a_m0_proof.mod')
        Sha256 = '8b3e4f1358d4f788c67d76c4c10a59db138dd8dd47617d74edd3692d7c474c5e'
        Install = $true
    }
)

function Get-Sha256([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

Write-Host "Native M0 runtime-proof installer: $Mode"
Write-Host 'This script never changes the game installation or Toolset configuration.'

foreach ($item in $expected) {
    if (-not (Test-Path -LiteralPath $item.Source -PathType Leaf)) {
        throw "Missing packet/dependency $($item.Kind): $($item.Source)"
    }
    $actual = Get-Sha256 $item.Source
    if ($actual -ne $item.Sha256) {
        throw "Source hash mismatch for $($item.Kind). Expected $($item.Sha256), got $actual."
    }

    $destinationDirectory = Split-Path -Parent $item.Destination
    if (-not (Test-Path -LiteralPath $destinationDirectory -PathType Container)) {
        throw "Missing destination directory for $($item.Kind): $destinationDirectory"
    }

    $targetState = if (Test-Path -LiteralPath $item.Destination -PathType Leaf) {
        $targetHash = Get-Sha256 $item.Destination
        if ($targetHash -ne $item.Sha256) {
            throw "Existing target hash mismatch for $($item.Kind): $($item.Destination)"
        }
        "EXISTS sha256=$targetHash"
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
    Write-Host 'PLAN ONLY: no files were copied.'
    exit 0
}

foreach ($item in $expected | Where-Object { $_.Install }) {
    if (Test-Path -LiteralPath $item.Destination -PathType Leaf) {
        throw "Refusing to overwrite existing target: $($item.Destination)"
    }
}

foreach ($item in $expected | Where-Object { $_.Install }) {
    # File.Copy(..., $false) is a second no-overwrite guard against races.
    [System.IO.File]::Copy($item.Source, $item.Destination, $false)
    $installedHash = Get-Sha256 $item.Destination
    if ($installedHash -ne $item.Sha256) {
        throw "Installed hash mismatch for $($item.Destination). Expected $($item.Sha256), got $installedHash."
    }
    Write-Host "INSTALLED and verified [$($item.Kind)]: $($item.Destination)"
}

Write-Host 'Install succeeded. Required Toolset proof: open m2a_m0_proof.mod, then confirm both fixtures in m2a_m0proof_area before Test Module.'
