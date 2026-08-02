import type { StudioWorkerClient } from "../../worker/client";
import type { AuthoredAnimationClipV1 } from "../animation-studio/types";
import { loadAnimationPresetAssetsV1 } from "./catalog";
import type {
  AnimationContributionExportV1,
  AnimationContributionMetadataV1,
  AnimationPresetCatalogEntryV1,
  AnimationPresetCompatibilityV1,
  AnimationPresetInstantiationV1,
} from "./types";

export async function inspectAnimationLibraryPresetV1(
  worker: StudioWorkerClient,
  sourceFile: File,
  preset: AnimationPresetCatalogEntryV1,
) {
  const assets = await loadAnimationPresetAssetsV1(
    preset.presetId,
    preset.presetVersion,
  );
  const response = await worker.inspectAnimationPresetCompatibility(
    assets.manifestJson,
    assets.animationJson,
    assets.catalogSha256,
    await sourceFile.arrayBuffer(),
  );
  if (
    !response.ok
    || response.type !== "ANIMATION_PRESET_COMPATIBILITY_INSPECTED"
  ) {
    throw new Error("The exact Core rig compatibility check failed.");
  }
  return JSON.parse(response.compatibilityJson) as AnimationPresetCompatibilityV1;
}

export async function instantiateAnimationLibraryPresetV1(
  worker: StudioWorkerClient,
  sourceFile: File,
  preset: AnimationPresetCatalogEntryV1,
  newId: string,
  newName: string,
) {
  const assets = await loadAnimationPresetAssetsV1(
    preset.presetId,
    preset.presetVersion,
  );
  const response = await worker.instantiateAnimationPreset(
    assets.manifestJson,
    assets.animationJson,
    assets.catalogSha256,
    await sourceFile.arrayBuffer(),
    newId,
    newName,
  );
  if (!response.ok || response.type !== "ANIMATION_PRESET_INSTANTIATED") {
    throw new Error("The exact Core preset instantiation failed.");
  }
  const result = JSON.parse(response.instantiationJson) as AnimationPresetInstantiationV1;
  if (result.status !== "READY" || !result.clip) {
    throw new Error(diagnosticMessageV1(result.diagnostics, "Preset is incompatible."));
  }
  return result.clip;
}

export async function exportAnimationLibraryContributionV1(
  worker: StudioWorkerClient,
  sourceFile: File,
  clip: AuthoredAnimationClipV1,
  metadata: AnimationContributionMetadataV1,
) {
  const response = await worker.exportAnimationContribution(
    JSON.stringify(clip),
    await sourceFile.arrayBuffer(),
    JSON.stringify(metadata),
  );
  if (!response.ok || response.type !== "ANIMATION_CONTRIBUTION_EXPORTED") {
    throw new Error("The exact Core contribution export failed.");
  }
  const result = JSON.parse(response.contributionJson) as AnimationContributionExportV1;
  if (result.status !== "READY" || !result.contribution) {
    throw new Error(diagnosticMessageV1(result.diagnostics, "Contribution is invalid."));
  }
  return result.contribution;
}

function diagnosticMessageV1(
  diagnostics: readonly { message: string; action: string }[],
  fallback: string,
) {
  return diagnostics.map(({ message, action }) => `${message} ${action}`).join(" ")
    || fallback;
}
