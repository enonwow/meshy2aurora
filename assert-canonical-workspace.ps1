[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$canonical = [System.IO.Path]::GetFullPath('C:\Projects\meshy2aurora').TrimEnd('\')
$animationWorktree = [System.IO.Path]::GetFullPath(
    'C:\Projects\meshy2aurora\.worktrees\animation'
).TrimEnd('\')
$forbidden = [System.IO.Path]::GetFullPath('C:\Users\enonw\Documents\meshy2aurora').TrimEnd('\')
$scriptRoot = [System.IO.Path]::GetFullPath($PSScriptRoot).TrimEnd('\')
$currentLocation = [System.IO.Path]::GetFullPath((Get-Location).Path).TrimEnd('\')

if (
    $currentLocation -ieq $forbidden -or
    $currentLocation.StartsWith($forbidden + '\', [System.StringComparison]::OrdinalIgnoreCase)
) {
    throw "HARD STOP: forbidden Meshy2Aurora workspace: $currentLocation"
}

if ($scriptRoot -ine $canonical -and $scriptRoot -ine $animationWorktree) {
    throw "HARD STOP: workspace guard is outside the approved Meshy2Aurora roots: $scriptRoot"
}

$safeDirectory = $scriptRoot.Replace('\', '/')
$gitRoot = (& git -c "safe.directory=$safeDirectory" -C $currentLocation rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: current location is not inside an approved Git worktree: $currentLocation"
}
$normalizedGitRoot = [System.IO.Path]::GetFullPath($gitRoot).TrimEnd('\')

if ($normalizedGitRoot -ine $canonical -and $normalizedGitRoot -ine $animationWorktree) {
    throw "HARD STOP: Git root '$normalizedGitRoot' is not approved"
}

if ($scriptRoot -ine $normalizedGitRoot) {
    throw "HARD STOP: guard root '$scriptRoot' does not match Git root '$normalizedGitRoot'"
}

if ($normalizedGitRoot -ieq $animationWorktree) {
    $branch = (& git -c "safe.directory=$safeDirectory" -C $currentLocation symbolic-ref --short HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $branch -cne 'animation') {
        throw "HARD STOP: approved animation worktree must use branch 'animation', found '$branch'"
    }

    $gitCommonDir = (& git -c "safe.directory=$safeDirectory" -C $currentLocation rev-parse --path-format=absolute --git-common-dir).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "HARD STOP: cannot resolve Git common directory for animation worktree"
    }
    $normalizedGitCommonDir = [System.IO.Path]::GetFullPath($gitCommonDir).TrimEnd('\')
    $expectedGitCommonDir = [System.IO.Path]::GetFullPath(
        (Join-Path $canonical '.git')
    ).TrimEnd('\')
    if ($normalizedGitCommonDir -ine $expectedGitCommonDir) {
        throw "HARD STOP: animation worktree is not linked to canonical Git metadata '$expectedGitCommonDir'"
    }
}

Write-Output "canonical-workspace-ok: $normalizedGitRoot"
