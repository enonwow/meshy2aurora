[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$expected = [System.IO.Path]::GetFullPath('C:\Projects\meshy2aurora').TrimEnd('\')
$expectedWorktreeRoot = [System.IO.Path]::GetFullPath(
    (Join-Path $expected '.worktrees')
).TrimEnd('\')
$expectedGitCommonDir = [System.IO.Path]::GetFullPath(
    (Join-Path $expected '.git')
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

if (
    $scriptRoot -ieq $forbidden -or
    $scriptRoot.StartsWith($forbidden + '\', [System.StringComparison]::OrdinalIgnoreCase)
) {
    throw "HARD STOP: workspace guard is inside forbidden Meshy2Aurora workspace: $scriptRoot"
}

$safeExpected = $expected.Replace('\', '/')
$safeScriptRoot = $scriptRoot.Replace('\', '/')
$gitRoot = (& git `
    -c "safe.directory=$safeExpected" `
    -c "safe.directory=$safeScriptRoot" `
    -C $currentLocation `
    rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: current location is not inside the canonical Git worktree: $currentLocation"
}
$normalizedGitRoot = [System.IO.Path]::GetFullPath($gitRoot).TrimEnd('\')

if ($scriptRoot -ine $normalizedGitRoot) {
    throw "HARD STOP: workspace guard root '$scriptRoot' differs from active Git root '$normalizedGitRoot'"
}

if (
    $currentLocation -ine $normalizedGitRoot -and
    -not $currentLocation.StartsWith(
        $normalizedGitRoot + '\',
        [System.StringComparison]::OrdinalIgnoreCase
    )
) {
    throw "HARD STOP: current location '$currentLocation' escapes active Git root '$normalizedGitRoot'"
}

if ($normalizedGitRoot -ieq $expected) {
    Write-Output "canonical-workspace-ok: $expected"
    exit 0
}

if (
    -not $normalizedGitRoot.StartsWith(
        $expectedWorktreeRoot + '\',
        [System.StringComparison]::OrdinalIgnoreCase
    )
) {
    throw "HARD STOP: Git root '$normalizedGitRoot' is neither the canonical checkout nor an in-repository linked worktree"
}

$gitCommonDir = (& git `
    -c "safe.directory=$safeExpected" `
    -c "safe.directory=$safeScriptRoot" `
    -C $normalizedGitRoot `
    rev-parse --git-common-dir).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: cannot resolve Git common directory for '$normalizedGitRoot'"
}
$gitCommonPath = if ([System.IO.Path]::IsPathRooted($gitCommonDir)) {
    $gitCommonDir
} else {
    Join-Path $normalizedGitRoot $gitCommonDir
}
$normalizedGitCommonDir = [System.IO.Path]::GetFullPath($gitCommonPath).TrimEnd('\')
if ($normalizedGitCommonDir -ine $expectedGitCommonDir) {
    throw "HARD STOP: linked worktree '$normalizedGitRoot' does not belong to canonical repo '$expected'"
}

$registered = (& git `
    -c "safe.directory=$safeExpected" `
    --git-dir=$expectedGitCommonDir `
    worktree list --porcelain) |
    Where-Object { $_ -like 'worktree *' } |
    ForEach-Object {
        [System.IO.Path]::GetFullPath($_.Substring('worktree '.Length)).TrimEnd('\')
    }
if ($LASTEXITCODE -ne 0) {
    throw "HARD STOP: cannot enumerate registered worktrees for '$expected'"
}
if (-not ($registered | Where-Object { $_ -ieq $normalizedGitRoot })) {
    throw "HARD STOP: linked worktree '$normalizedGitRoot' is not registered in canonical repo '$expected'"
}

Write-Output "canonical-worktree-ok: $normalizedGitRoot (repository: $expected)"
