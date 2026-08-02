import type { AuthoredAnimationClipV1, AuthoredAnimationTrackPathV1 } from "../animation-studio/types";

export interface AnimationLibraryDiagnosticV1 {
  schemaVersion: 1;
  code: string;
  path: string;
  level?: "INFO" | "WARNING" | "BLOCKING";
  message: string;
  action: string;
}

export type AnimationPresetSourceV1 = "BUILT_IN" | "COMMUNITY";
export type AnimationPresetPlaybackV1 = "ONE_SHOT" | "LOOP";
export type AnimationLibraryValidationStatusV1 =
  | "PIPELINE_VERIFIED"
  | "OWNER_NWN_VERIFIED";

export interface AnimationPresetAuthorV1 {
  name: string;
}

export interface AnimationPresetCatalogEntryV1 {
  schemaVersion: 1;
  presetId: string;
  presetVersion: number;
  outputName: string;
  label: string;
  summary: string;
  source: AnimationPresetSourceV1;
  authors: AnimationPresetAuthorV1[];
  license: string;
  tags: string[];
  playback: AnimationPresetPlaybackV1;
  durationSeconds: number;
  rigProfile: string;
  rigSignatureSha256: string;
  requiredBones: string[];
  animationPath: "animation.json";
  animationByteLength: number;
  animationSha256: string;
  motionSha256: string;
  previewPath: "preview.webp" | null;
  previewByteLength: number | null;
  previewSha256: string | null;
  validationStatus: AnimationLibraryValidationStatusV1;
}

export interface CommunityAnimationCatalogV1 {
  schemaVersion: 1;
  source: "EMBEDDED_RELEASE";
  entries: AnimationPresetCatalogEntryV1[];
  catalogSha256: string;
}

export interface AnimationPresetKeyframeV1 {
  timeSeconds: number;
  value: number[];
}

export interface AnimationPresetPayloadV1 {
  schemaVersion: 1;
  durationSeconds: number;
  animationRootBoneName: string;
  tracks: Array<{
    targetBoneName: string;
    path: AuthoredAnimationTrackPathV1;
    interpolation: "LINEAR";
    keyframes: AnimationPresetKeyframeV1[];
  }>;
  events: Array<{ timeSeconds: number; name: string }>;
}

export interface LoadedAnimationPresetAssetsV1 {
  entry: AnimationPresetCatalogEntryV1;
  manifestJson: string;
  animationJson: string;
  catalogSha256: string;
  previewUrl: string | null;
}

export interface AnimationPresetCompatibilityV1 {
  schemaVersion: 1;
  status: "COMPATIBLE" | "INCOMPATIBLE";
  expectedRigSignatureSha256: string;
  actualRigSignatureSha256: string | null;
  missingBones: string[];
  diagnostics: AnimationLibraryDiagnosticV1[];
}

export interface AnimationPresetInstantiationV1 {
  schemaVersion: 1;
  status: "READY" | "BLOCKED";
  clip: AuthoredAnimationClipV1 | null;
  diagnostics: AnimationLibraryDiagnosticV1[];
}

export interface AnimationContributionExportV1 {
  schemaVersion: 1;
  status: "READY" | "BLOCKED";
  contribution: unknown | null;
  diagnostics: AnimationLibraryDiagnosticV1[];
}

export interface AnimationContributionMetadataV1 {
  presetId: string;
  presetVersion: number;
  outputName: string;
  label: string;
  summary: string;
  authors: AnimationPresetAuthorV1[];
  license: "CC0-1.0" | "CC-BY-4.0" | "LicenseRef-Meshy2Aurora-Project-Generated";
  tags: string[];
  playback: AnimationPresetPlaybackV1;
  rigProfile: string;
  validationStatus: AnimationLibraryValidationStatusV1;
}
