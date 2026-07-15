[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$expected = [System.IO.Path]::GetFullPath('C:\Projects\meshy2aurora').TrimEnd('\')
$forbidden = [System.IO.Path]::GetFullPath('C:\Users\enonw\Documents\meshy2aurora').TrimEnd('\')
$scriptRoot = [System.IO.Path]::GetFullPath($PSScriptRoot).TrimEnd('\')
$currentLocation = [System.IO.Path]::GetFullPath((Get-Location).Path).TrimEnd('\')

if (
    $currentLocation -ieq $forbidden -or
    $currentLocation.StartsWith($forbidden + '\', [System.StringComparison]::OrdinalIgnoreCase)
) {
    throw "HARD STOP: forbidden Meshy2Aurora workspace: $currentLocation"
}

if ($scriptRoot -ine $expected) {
    throw "HARD STOP: workspace guard is not located in canonical repo '$expected': $scriptRoot"
}

$safeDirectory = $expected.Replace('\', '/')
$gitRoot = (& git -c "safe.directory=$safeDirectory" -C $currentLocation rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: current location is not inside the canonical Git worktree: $currentLocation"
}
$normalizedGitRoot = [System.IO.Path]::GetFullPath($gitRoot).TrimEnd('\')

if ($normalizedGitRoot -ine $expected) {
    throw "HARD STOP: Git root '$normalizedGitRoot' is not canonical '$expected'"
}

Write-Output "canonical-workspace-ok: $expected"
