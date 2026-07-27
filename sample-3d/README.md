# Meshy source model library

This is the only canonical root for owner-selected local Meshy source models.
Each model lives in `sample-3d/<asset-id>/` with a tracked `manifest.yaml`.
Binary model payloads remain local and are ignored by Git.

The binding policy and migration record are in
`documentation/MESHY_ASSET_LAYOUT.md`. Validate the layout from the repository
root with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File assert-meshy-asset-layout.ps1
```
