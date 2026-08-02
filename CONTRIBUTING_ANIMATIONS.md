# Contributing animations

The Animation Studio exports declarative `community-animation-contribution-v1`
JSON. A contribution contains portable bone-name tracks and metadata only. It
must never contain a GLB, FBX, MDL, HAK, MOD, executable, local filesystem path,
access token or copied retail/CEP animation payload.

## Local workflow

1. Open an exact model and create or edit a Custom animation.
2. Resolve diagnostics and use **Save to Custom** so the clip is `VALID`.
3. Use **Export contribution**, fill in author, license, tags, label and summary,
   and download the JSON.
4. From the repository root run:

   `node scripts/import-animation-contribution.mjs --input <file.json>`

5. Regenerate the tracked catalog:

   `node scripts/generate-community-animation-catalog.mjs --write`

6. Run the same local gate as CI:

   `node scripts/generate-community-animation-catalog.mjs --check`

The importer refuses to overwrite an existing preset directory. An already
published `presetId + presetVersion` is immutable; corrections require a higher
version.

## Rights and provenance

Only motion that you created or have explicit rights to redistribute may be
submitted. Retail Neverwinter Nights, CEP and third-party animation payloads
are reference-only and are not eligible for export.

The repository owner has not yet selected the repository-wide public license.
Until that decision is recorded in a root `LICENSE`, external public animation
PRs remain closed. Project-generated built-ins use the temporary internal
`LicenseRef-Meshy2Aurora-Project-Generated`; this is not a grant for external
reuse.

## Validation

Core owns the strict schema, tag and license allowlists, rig signature, motion
hash, path checks and size limits. GitHub Actions additionally prevents changes
or removal of published versions. A `PIPELINE_VERIFIED` badge means the preset
passed offline Core/WASM/MDL gates; only an owner-reported runtime result may use
`OWNER_NWN_VERIFIED`.
