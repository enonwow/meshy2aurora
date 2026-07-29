[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$canonical = [System.IO.Path]::GetFullPath('C:\Projects\meshy2aurora').TrimEnd('\')
$forbidden = [System.IO.Path]::GetFullPath('C:\Users\enonw\Documents\meshy2aurora').TrimEnd('\')
$scriptRoot = [System.IO.Path]::GetFullPath($PSScriptRoot).TrimEnd('\')
$currentLocation = [System.IO.Path]::GetFullPath((Get-Location).Path).TrimEnd('\')

if (
    $currentLocation -ieq $forbidden -or
    $currentLocation.StartsWith($forbidden + '\', [System.StringComparison]::OrdinalIgnoreCase)
) {
    throw "HARD STOP: forbidden Meshy2Aurora workspace: $currentLocation"
}

$safeDirectory = $scriptRoot.Replace('\', '/')
$gitRoot = (& git -c "safe.directory=$safeDirectory" -C $currentLocation rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: current location is not inside an approved Git worktree: $currentLocation"
}
$normalizedGitRoot = [System.IO.Path]::GetFullPath($gitRoot).TrimEnd('\')

$isPrimaryRoot = $normalizedGitRoot -ieq $canonical
$isNestedWorktree = $normalizedGitRoot.StartsWith(
    $canonical + '\',
    [System.StringComparison]::OrdinalIgnoreCase
)
if (-not $isPrimaryRoot -and -not $isNestedWorktree) {
    throw "HARD STOP: Git root '$normalizedGitRoot' is outside canonical root '$canonical'"
}

if ($scriptRoot -ine $normalizedGitRoot) {
    throw "HARD STOP: guard root '$scriptRoot' does not match Git root '$normalizedGitRoot'"
}

$gitCommonDir = (& git -c "safe.directory=$safeDirectory" -C $currentLocation rev-parse --path-format=absolute --git-common-dir).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: cannot resolve Git common directory for '$normalizedGitRoot'"
}
$normalizedGitCommonDir = [System.IO.Path]::GetFullPath($gitCommonDir).TrimEnd('\')
$expectedGitCommonDir = [System.IO.Path]::GetFullPath(
    (Join-Path $canonical '.git')
).TrimEnd('\')
if ($normalizedGitCommonDir -ine $expectedGitCommonDir) {
    throw "HARD STOP: worktree is not linked to canonical Git metadata '$expectedGitCommonDir'"
}

$registeredWorktrees = @(
    & git -c "safe.directory=$($canonical.Replace('\', '/'))" -C $canonical worktree list --porcelain |
        Where-Object { $_ -like 'worktree *' } |
        ForEach-Object {
            [System.IO.Path]::GetFullPath($_.Substring('worktree '.Length)).TrimEnd('\')
        }
)
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: cannot resolve registered worktrees for '$canonical'"
}
if (-not ($registeredWorktrees | Where-Object { $_ -ieq $normalizedGitRoot })) {
    throw "HARD STOP: Git root '$normalizedGitRoot' is not a registered canonical worktree"
}

Write-Output "canonical-workspace-ok: $normalizedGitRoot"
