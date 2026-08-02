[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$expectedRoot = "C:\Projects\meshy2aurora"
$activeRoot = (Resolve-Path -LiteralPath $PSScriptRoot).Path.TrimEnd("\")

& (Join-Path $activeRoot "assert-canonical-workspace.ps1")

$sampleRoot = Join-Path $expectedRoot "sample-3d"
$retiredRoots = @(
    (Join-Path $expectedRoot "test-assets\meshy"),
    (Join-Path $activeRoot "test-assets\meshy")
) | Select-Object -Unique
$errors = [System.Collections.Generic.List[string]]::new()

if (-not (Test-Path -LiteralPath $sampleRoot -PathType Container)) {
    $errors.Add("missing canonical Meshy asset root: $sampleRoot")
}

foreach ($retiredRoot in $retiredRoots) {
    if (Test-Path -LiteralPath $retiredRoot) {
        $errors.Add("retired competing asset root still exists: $retiredRoot")
    }
}

if (Test-Path -LiteralPath $sampleRoot -PathType Container) {
    $assetDirectories = @(Get-ChildItem -LiteralPath $sampleRoot -Directory -Force)

    foreach ($assetDirectory in $assetDirectories) {
        $manifestPath = Join-Path $assetDirectory.FullName "manifest.yaml"
        if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
            $errors.Add("sample has no manifest.yaml: $($assetDirectory.FullName)")
            continue
        }

        $manifest = Get-Content -LiteralPath $manifestPath -Raw
        $assetIdMatch = [regex]::Match(
            $manifest,
            "(?m)^asset_id:\s*['""]?([^'""\r\n]+)['""]?\s*$"
        )
        if (-not $assetIdMatch.Success) {
            $errors.Add("manifest has no asset_id: $manifestPath")
        }
        elseif ($assetIdMatch.Groups[1].Value.Trim() -cne $assetDirectory.Name) {
            $errors.Add(
                "manifest asset_id '$($assetIdMatch.Groups[1].Value.Trim())' " +
                "does not match directory '$($assetDirectory.Name)': $manifestPath"
            )
        }

        $payloadFiles = @(
            Get-ChildItem -LiteralPath $assetDirectory.FullName -Recurse -File -Force |
                Where-Object { $_.Extension -in @(".glb", ".gltf", ".fbx", ".zip") }
        )

        foreach ($payloadFile in $payloadFiles) {
            if ($payloadFile.DirectoryName -cne $assetDirectory.FullName) {
                $errors.Add(
                    "payload must be directly inside sample-3d/<asset-id>/: " +
                    $payloadFile.FullName
                )
                continue
            }

            if ($manifest -notmatch "(?m)^\s*path:\s*['""]?$([regex]::Escape($payloadFile.Name))['""]?\s*$") {
                $errors.Add("payload is not declared in manifest: $($payloadFile.FullName)")
            }

            $hash = (Get-FileHash -LiteralPath $payloadFile.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($manifest -notmatch [regex]::Escape($hash)) {
                $errors.Add("payload SHA-256 is absent from manifest: $($payloadFile.FullName)")
            }

            if ($manifest -notmatch "(?m)^\s*size_bytes:\s*$($payloadFile.Length)\s*$") {
                $errors.Add("payload size is absent from manifest: $($payloadFile.FullName)")
            }

            $relativePayload = $payloadFile.FullName.Substring($expectedRoot.Length).TrimStart("\")
            & git -C $expectedRoot check-ignore --quiet -- $relativePayload
            if ($LASTEXITCODE -ne 0) {
                $errors.Add("local Meshy payload is not ignored by Git: $relativePayload")
            }
        }
    }

    $rootPayloads = @(
        Get-ChildItem -LiteralPath $sampleRoot -File -Force |
            Where-Object { $_.Extension -in @(".glb", ".gltf", ".fbx", ".zip") }
    )
    foreach ($rootPayload in $rootPayloads) {
        $errors.Add("payload must be inside sample-3d/<asset-id>/: $($rootPayload.FullName)")
    }
}

$productRoots = @("apps", "crates", "tools")
$legacyPattern = "test-assets[\\/]meshy"
foreach ($relativeProductRoot in $productRoots) {
    $productRoot = Join-Path $activeRoot $relativeProductRoot
    if (-not (Test-Path -LiteralPath $productRoot -PathType Container)) {
        continue
    }

    $sourceFiles = @(
        Get-ChildItem -LiteralPath $productRoot -Recurse -File -Force |
            Where-Object {
                $_.FullName -notmatch "[\\/](target|node_modules|dist|pkg)[\\/]" -and
                $_.Extension -in @(
                    ".c", ".cc", ".cpp", ".cs", ".h", ".hpp", ".js", ".jsx",
                    ".mjs", ".rs", ".ts", ".tsx", ".json", ".toml", ".yaml", ".yml"
                )
            }
    )

    foreach ($sourceFile in $sourceFiles) {
        $match = Select-String -LiteralPath $sourceFile.FullName -Pattern $legacyPattern -AllMatches
        if ($match) {
            $relativePath = $sourceFile.FullName.Substring($activeRoot.Length).TrimStart("\")
            $errors.Add("product code references retired test-assets/meshy root: $relativePath")
        }
    }
}

$documentationRoot = Join-Path $activeRoot "documentation"
$legacyDocumentationPayloadPattern =
    "test-assets[\\/]meshy[\\/].*\.(glb|gltf|fbx|zip)"
if (Test-Path -LiteralPath $documentationRoot -PathType Container) {
    $documentationFiles = @(
        Get-ChildItem -LiteralPath $documentationRoot -Recurse -File -Force |
            Where-Object { $_.Extension -in @(".md", ".yaml", ".yml", ".json") }
    )

    foreach ($documentationFile in $documentationFiles) {
        $match = Select-String -LiteralPath $documentationFile.FullName `
            -Pattern $legacyDocumentationPayloadPattern -AllMatches
        if ($match) {
            $relativePath = $documentationFile.FullName.Substring($activeRoot.Length).TrimStart("\")
            $errors.Add(
                "documentation references a concrete payload under retired " +
                "test-assets/meshy root: $relativePath"
            )
        }
    }
}

if ($errors.Count -gt 0) {
    $message = "meshy-asset-layout: FAILED`n - " + ($errors -join "`n - ")
    throw $message
}

Write-Output "meshy-asset-layout-ok: $sampleRoot (workspace: $activeRoot)"
