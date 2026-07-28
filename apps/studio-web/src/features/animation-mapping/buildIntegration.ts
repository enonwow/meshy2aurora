import type { StudioSessionState } from "../../app/studioSession";
import type {
  CanonicalAnimationMappingEvidenceV1,
  CanonicalResultSnapshot,
} from "../results/projectCanonicalResult";
import type { BinaryMdlInspectionReport } from "../preview/types";
import { serializeCreatureAnimationAuthoringV1 } from "./persistence";
import type {
  CreatureAnimationAuthoringV1,
  CreatureAnimationMappingDiagnosticV1,
} from "./types";
import { FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1 } from "../source/directCreatureAnimationProfile";

export interface CreatureAnimationBuildInputV1 {
  schemaVersion: 1;
  sessionRevision: number;
  sourceRevision: string;
  authoringRevision: number;
  authoringFingerprintSha256: string;
  animationAuthoringJson: string;
  authoring: CreatureAnimationAuthoringV1;
}

export interface BuiltAnimationCoverageRowV1 {
  targetSlot: string;
  resolvedSourceSlot: string;
  sourceKind: string;
  sourceClipName: string | null;
  provider: string;
  assetId: string;
  ownership: string;
  viaFallbackSlots: string[];
}

export interface AuthoredBuiltAnimationReconciliationV1 {
  schemaVersion: 1;
  matches: boolean;
  expectedClipNames: string[];
  actualClipNames: string[];
  missingClipNames: string[];
  unexpectedClipNames: string[];
  diagnostics: CreatureAnimationMappingDiagnosticV1[];
}

export function createCreatureAnimationBuildInputV1(
  session: Pick<
    StudioSessionState,
    "revision" | "target" | "source" | "animationMapping" | "animationMappingValidation"
  >,
): CreatureAnimationBuildInputV1 {
  if (
    session.target !== "CREATURE"
    || !session.source?.sha256
    || session.animationMapping?.revision !== session.revision
    || session.animationMapping.value.sourceRevision !== session.source.sha256
    || session.animationMappingValidation?.revision !== session.revision
    || session.animationMappingValidation.value.authoringRevision
      !== session.animationMapping.value.authoringRevision
    || session.animationMappingValidation.value.status !== "READY"
    || !/^[0-9a-f]{64}$/.test(
      session.animationMappingValidation.value.authoringFingerprintSha256 ?? "",
    )
  ) {
    throw new RangeError("Current ready creature animation mapping is required for build");
  }
  const animationAuthoringJson = serializeCreatureAnimationAuthoringV1(
    session.animationMapping.value,
  );
  return {
    schemaVersion: 1,
    sessionRevision: session.revision,
    sourceRevision: session.source.sha256,
    authoringRevision: session.animationMapping.value.authoringRevision,
    authoringFingerprintSha256:
      session.animationMappingValidation.value.authoringFingerprintSha256!,
    animationAuthoringJson,
    authoring: JSON.parse(animationAuthoringJson) as CreatureAnimationAuthoringV1,
  };
}

export function assertCreatureAnimationBuildInputCurrentV1(
  session: Pick<
    StudioSessionState,
    "revision" | "target" | "source" | "animationMapping" | "animationMappingValidation"
  >,
  input: CreatureAnimationBuildInputV1,
) {
  const current = createCreatureAnimationBuildInputV1(session);
  if (
    input.schemaVersion !== current.schemaVersion
    || input.sessionRevision !== current.sessionRevision
    || input.sourceRevision !== current.sourceRevision
    || input.authoringRevision !== current.authoringRevision
    || input.authoringFingerprintSha256 !== current.authoringFingerprintSha256
    || input.animationAuthoringJson !== current.animationAuthoringJson
  ) {
    throw new RangeError("Creature animation build input is stale");
  }
}

export function projectBuiltAnimationCoverageV1(
  result: Pick<CanonicalResultSnapshot, "animationMappingEvidence">,
): BuiltAnimationCoverageRowV1[] {
  const evidence = requireBuiltMappingEvidence(result.animationMappingEvidence);
  return evidence.baseAnimations.map((animation) => ({
    targetSlot: animation.targetSlot,
    resolvedSourceSlot: animation.resolvedSourceSlot,
    sourceKind: animation.sourceKind,
    sourceClipName: animation.sourceClipName,
    provider: animation.provider,
    assetId: animation.assetId,
    ownership: animation.ownership,
    viaFallbackSlots: [...animation.viaFallbackSlots],
  }));
}

export function reconcileAuthoredAndBuiltAnimationsV1(
  authoring: CreatureAnimationAuthoringV1,
  readback: BinaryMdlInspectionReport,
): AuthoredBuiltAnimationReconciliationV1 {
  const expectedClipNames = [
    ...FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
    ...authoring.customAnimations.flatMap((custom) => (
      custom.playback === "ONE_SHOT"
        ? [custom.name]
        : custom.phases.map(({ phase }) => (
            phase === "START"
              ? `${custom.name}_s`
              : phase === "END"
                ? `${custom.name}_e`
                : custom.name
          ))
    )),
  ].filter((name, index, values) => (
    values.findIndex((candidate) => equalName(candidate, name)) === index
  ));
  const actualClipNames = readback.animations.map(({ name }) => name);
  const missingClipNames = expectedClipNames.filter((expected) => (
    !actualClipNames.some((actual) => equalName(actual, expected))
  ));
  const expectedFolded = new Set(expectedClipNames.map((name) => name.toLowerCase()));
  const unexpectedClipNames = actualClipNames.filter((actual) => (
    !expectedFolded.has(actual.toLowerCase())
  ));
  const actualFolded = actualClipNames.map((name) => name.toLowerCase());
  const duplicateActualClipNames = actualClipNames.filter((name, index) => (
    actualFolded.indexOf(name.toLowerCase()) !== index
  ));
  const orderMatches = expectedClipNames.length === actualClipNames.length
    && expectedClipNames.every((expected, index) => (
      equalName(expected, actualClipNames[index])
    ));
  const diagnostics: CreatureAnimationMappingDiagnosticV1[] = [
    ...missingClipNames.map((name) => diagnostic(
      "M2A-ANIMATION-READBACK-EXPECTED-MISSING",
      `readback.animations.${name}`,
      `Binary MDL readback is missing authored animation ${name}.`,
      "Rebuild from the current mapping and inspect the canonical writer report.",
    )),
    ...unexpectedClipNames.map((name) => diagnostic(
      "M2A-ANIMATION-READBACK-UNEXPECTED",
      `readback.animations.${name}`,
      `Binary MDL readback contains unexpected animation ${name}.`,
      "Review the materialized mapping before download.",
    )),
    ...duplicateActualClipNames.map((name) => diagnostic(
      "M2A-ANIMATION-READBACK-DUPLICATE",
      `readback.animations.${name}`,
      `Binary MDL readback contains duplicate animation ${name}.`,
      "Inspect the writer output; every Base 42 and custom clip name must be unique.",
    )),
    ...(!orderMatches && missingClipNames.length === 0 && unexpectedClipNames.length === 0
      ? [diagnostic(
          "M2A-ANIMATION-READBACK-ORDER",
          "readback.animations",
          "Binary MDL animation order differs from the canonical Base 42 plus custom order.",
          "Rebuild with the canonical authored animation materializer.",
        )]
      : []),
  ];
  return {
    schemaVersion: 1,
    matches: diagnostics.length === 0,
    expectedClipNames,
    actualClipNames,
    missingClipNames,
    unexpectedClipNames,
    diagnostics,
  };
}

function requireBuiltMappingEvidence(
  evidence: CanonicalAnimationMappingEvidenceV1 | undefined,
) {
  if (!evidence) {
    throw new RangeError("Canonical build result has no authored animation mapping evidence");
  }
  return evidence;
}

function equalName(left: string, right: string) {
  return left.localeCompare(right, undefined, { sensitivity: "base" }) === 0;
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
): CreatureAnimationMappingDiagnosticV1 {
  return {
    schemaVersion: 1,
    code,
    path,
    level: "BLOCKING",
    message,
    action,
  };
}
