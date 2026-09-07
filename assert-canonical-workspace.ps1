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
    throw "HARD STOP: current location is not a Git worktree: $currentLocation"
}
$normalizedGitRoot = [System.IO.Path]::GetFullPath($gitRoot).TrimEnd('\')

$gitCommonDir = (& git -c "safe.directory=$safeDirectory" -C $currentLocation rev-parse --path-format=absolute --git-common-dir).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: cannot resolve Git common directory for '$currentLocation'"
}
$normalizedGitCommonDir = [System.IO.Path]::GetFullPath($gitCommonDir).TrimEnd('\')
$expectedGitCommonDir = [System.IO.Path]::GetFullPath((Join-Path $expected '.git')).TrimEnd('\')

if ($normalizedGitCommonDir -ine $expectedGitCommonDir) {
    throw "HARD STOP: Git worktree '$normalizedGitRoot' is not registered by canonical repository '$expected'"
}

$registeredWorktrees = @(
    & git -c "safe.directory=$safeDirectory" -C $expected worktree list --porcelain |
        Where-Object { $_ -like 'worktree *' } |
        ForEach-Object {
            [System.IO.Path]::GetFullPath($_.Substring('worktree '.Length)).TrimEnd('\')
        }
)
if ($LASTEXITCODE -ne 0 -or $registeredWorktrees -inotcontains $normalizedGitRoot) {
    throw "HARD STOP: Git worktree '$normalizedGitRoot' is not present in canonical worktree registry"
}

Write-Output "canonical-worktree-ok: $normalizedGitRoot"
