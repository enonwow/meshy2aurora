import type { WorkerArtifact } from "../../worker/types";
import {
  resolveOwnerRuntimeProofV1,
  type CanonicalRuntimeAcceptance,
} from "./ownerRuntimeProofs";

export interface CanonicalResultSnapshot {
  status: string;
  sourceMetrics: CanonicalModelMetrics;
  convertedMetrics: CanonicalModelMetrics;
  geometry: { vertices: number; triangles: number; joints: number; deformation: string };
  animation: { sourceName: string; outputName: string; durationSeconds: number; hasMotion: boolean };
  texture: { width: number; height: number; pixelFormat: string; byteLength: number };
  resrefs: { model: string; texture: string };
  appearance: { appendedRow: number; sourcePrefixPreserved: boolean; policy: string };
  hak: { byteLength: number; sha256: string; entryCount: number };
  outputs: Record<string, { byteLength: number; sha256: string }>;
  resources: Array<{ role: string; resref: string; type: number; byteLength: number; sha256: string }>;
  semanticEvidence: {
    semanticDiff: string[];
    deviations: Array<{ code: string; path: string; message: string }>;
  };
  conversionEvidence: CanonicalConversionEvidence;
  packageAssemblyEvidence: {
    strictReconciled: true;
    resourceCount: number;
    artifactCount: number;
  };
  runtimeAcceptance: CanonicalRuntimeAcceptance;
  animationEventEvidence?: CanonicalAnimationEventEvidence;
  weaponAnchorAuthoring?: CanonicalWeaponAnchorAuthoringReport;
  skinAccessoryStabilization?: CanonicalSkinAccessoryStabilizationReport;
  materialFidelity?: CanonicalMaterialFidelityReport;
  referenceSupermodelCoverage?: CanonicalReferenceSupermodelCoverage;
  demo?: CanonicalCreatureDemoReport;
  runtimeFixtureContract?: CanonicalM0RuntimeFixtureContract;
  artifacts: WorkerArtifact[];
  reportJson: string;
  summaryJson: string;
  manifestJson: string;
}

export interface CanonicalReferenceSupermodelCoverage {
  fullCarrierCoverage: true;
  requiredJointCoverage: true;
  skinInfluenceCoverage: true;
  inheritedClipCoverage: true;
  visibleMotionCoverage: true;
  carrierNodeCount: number;
  allowedBoneCount: number;
  activeWeightedBoneCount: number;
  passiveUnweightedJointNames: string[];
  requiredClipCount: number;
  sampledClipCount: number;
  jointClipRequiredCount: number;
  jointClipPassCount: number;
  seamViolationCount: 0;
}

export interface CanonicalConversionGate {
  schemaVersion: 1;
  code: string;
  severity: string;
  path: string;
  expected: string;
  actual: string;
  message: string;
}

export interface CanonicalConversionDiagnostic {
  schemaVersion: 1;
  code: string;
  severity: string;
  path: string;
  message: string;
}

export interface CanonicalConversionEvidence {
  schemaVersion: 1;
  conversionEligible: boolean;
  policies: {
    basisStatus: string;
    assetForwardMapping: string;
    orientationParity: string;
    engineFacingProof: string;
    uvRuntimeProof: string;
  };
  gates: CanonicalConversionGate[];
  diagnostics: CanonicalConversionDiagnostic[];
}

export interface CanonicalModelMetrics {
  nodes: number;
  meshes: number;
  vertices: number;
  triangles: number;
  animations: number;
}

export interface CanonicalAnimationEventEvidence {
  schemaVersion: 1;
  profile: string;
  requiredPairCount: number;
  satisfiedPairCount: number;
  totalEventCount: number;
  unknownEventNames: string[];
  missingPairs: string[];
  complete: true;
  authoringCanonical: { byteLength: number; sha256: string };
}

export interface CanonicalWeaponAnchorBinding {
  anchorName: string;
  anchorNodeId: number;
  parentBoneName: string;
  parentBoneNodeId: number;
  localMatrix: number[];
  disposition: "added" | "reused_compatible";
  weightedVertexCount: number;
}

export interface CanonicalWeaponAnchorAuthoringReport {
  schemaVersion: 1;
  status: "weapon_anchors_ready";
  calibration: string;
  profileSha256Before: string;
  profileSha256After: string;
  anchors: CanonicalWeaponAnchorBinding[];
  gripAdjustment?: CanonicalWeaponGripAdjustmentReport;
}

export interface CanonicalWeaponEulerOffset {
  rollDegrees: number;
  pitchDegrees: number;
  yawDegrees: number;
}

export interface CanonicalWeaponGripHandReport {
  requested: CanonicalWeaponEulerOffset;
  automaticLocalMatrix: number[];
  finalLocalMatrix: number[];
}

export interface CanonicalWeaponGripAdjustmentReport {
  schemaVersion: 1;
  mode: "AUTO_PLUS_OFFSETS";
  compositionOrder: "AUTO_X_RZ_YAW_X_RX_PITCH_X_RY_ROLL_LOCAL_ITEM_AXES";
  rightHand: CanonicalWeaponGripHandReport;
  leftHand: CanonicalWeaponGripHandReport;
}

export interface CanonicalSkinAccessoryDeformationMetrics {
  sampledClipCount: number;
  sampledPoseCount: number;
  sampledVertexCount: number;
  vertexSamplingMode: string;
  timeSamplingTruncatedClipCount: number;
  maxPairDistanceRatio: number;
  maxPairDistanceError: number;
  minAxisAlignment: number;
}

export interface CanonicalSkinAccessoryComponentReport {
  segmentIndex: number;
  componentIndex: number;
  triangleCount: number;
  vertexCount: number;
  isPrimaryBody: boolean;
  centroid: [number, number, number];
  activeBoneCount: number;
  dominantBoneName?: string;
  dominantBoneShare: number;
  riskReasons: string[];
  action: string;
  selectedBoneId?: number;
  selectedBoneName?: string;
  changedVertexCount: number;
  before: CanonicalSkinAccessoryDeformationMetrics;
  after: CanonicalSkinAccessoryDeformationMetrics;
}

export interface CanonicalSkinAccessoryStabilizationReport {
  schemaVersion: 2;
  mode: string;
  auditedClipCount: number;
  weldTolerance: number;
  componentCount: number;
  detachedComponentCount: number;
  riskyComponentCount: number;
  stabilizedComponentCount: number;
  changedVertexCount: number;
  components: CanonicalSkinAccessoryComponentReport[];
  warnings: string[];
}

export interface CanonicalCreatureDemoReport {
  schemaVersion: 2;
  moduleResref: string;
  moduleDisplayName: string;
  areaResref: string;
  areaDisplayName: string;
  creatureResref: string;
  hakResref: string;
  appearanceRow: number;
  resourceCount: number;
  byteLength: number;
  sha256: string;
  semanticReadbackStatus: "PASS";
  heldStockWeaponReadback?: CanonicalHeldStockWeaponReadback;
}

export interface CanonicalHeldStockWeaponReadback {
  schemaVersion: 2;
  weapon: {
    resref: string;
    resourceType: 2025;
    resourceScope: "NWN_BASE_GAME";
  };
  fixtures: [{
    hand: "right_hand" | "left_hand";
    equippedItemResref: string;
  }];
}

export interface CanonicalMaterialFidelityReport {
  schemaVersion: 1;
  materialSlot: number;
  sourceMaterialId: number;
  baseColorFactor: [number, number, number, number];
  baseColorFactorBaked: boolean;
  alphaMode: string;
  alphaCutoff?: number;
  alphaChannelPreserved: boolean;
  metallicFactor: number;
  roughnessFactor: number;
  normalTexturePresent: boolean;
  emissiveFactor: [number, number, number];
  emissiveTexturePresent: boolean;
  doubleSided: boolean;
  auroraMaterialProfile: string;
  mappedFields: string[];
  unsupportedFields: string[];
}

export interface CanonicalM0RuntimeResource {
  resref: string;
  byteLength: number;
  sha256: string;
}

export interface CanonicalM0RuntimeFixtureContract {
  schemaVersion: 2;
  lane: "M0_BINARY_VERTICAL_SLICE";
  module: CanonicalM0RuntimeResource;
  hak: CanonicalM0RuntimeResource;
  model: CanonicalM0RuntimeResource;
  texture: CanonicalM0RuntimeResource;
  appearanceTwoDa: CanonicalM0RuntimeResource;
  appearance: { physicalRow: number; label: string; modelType: string; race: string };
  binaryScene: {
    moduleResref: string;
    areaResref: string;
    orderedHakResrefs: string[];
    entryPosition: { x: number; y: number; z: number };
    entryDirection: { x: number; y: number };
    fixture: {
      templateResref: string;
      appearanceRow: number;
      position: { x: number; y: number; z: number };
      orientation: { x: number; y: number };
    };
  };
  meshEligibility: {
    eligible: boolean;
    checkedMeshCount: number;
    eligibleMeshCount: number;
    meshTypes: number[];
    textureResrefs: string[];
    rule: string;
  };
}

type JsonRecord = Record<string, unknown>;

const fail = (path: string): never => { throw new Error(`Canonical result field ${path} is missing or has the wrong type`); };
const record = (value: unknown, path: string): JsonRecord => value !== null && typeof value === "object" && !Array.isArray(value) ? value as JsonRecord : fail(path);
const array = (value: unknown, path: string): unknown[] => Array.isArray(value) ? value : fail(path);
const string = (value: unknown, path: string): string => typeof value === "string" && value.length > 0 ? value : fail(path);
const boolean = (value: unknown, path: string): boolean => typeof value === "boolean" ? value : fail(path);
const number = (value: unknown, path: string): number => typeof value === "number" && Number.isFinite(value) ? value : fail(path);
const integer = (value: unknown, path: string): number => Number.isSafeInteger(value) && (value as number) >= 0 ? value as number : fail(path);
const stringArray = (value: unknown, path: string): string[] => array(value, path).map((entry, index) => string(entry, `${path}[${index}]`));
const sha256 = (value: unknown, path: string): string => {
  const result = string(value, path);
  return /^[0-9a-f]{64}$/.test(result) ? result : fail(path);
};

function identity(value: unknown, path: string) {
  const item = record(value, path);
  return { byteLength: integer(item.byteLength, `${path}.byteLength`), sha256: sha256(item.sha256, `${path}.sha256`) };
}

function parseJson(json: string, path: string) {
  try { return record(JSON.parse(json), path); } catch { return fail(path); }
}

function runtimeResource(value: unknown, path: string): CanonicalM0RuntimeResource {
  const item = record(value, path);
  return { resref: string(item.resref, `${path}.resref`), ...identity(item, path) };
}

function position3(value: unknown, path: string) {
  const item = record(value, path);
  return {
    x: number(item.x, `${path}.x`),
    y: number(item.y, `${path}.y`),
    z: number(item.z, `${path}.z`),
  };
}

function direction2(value: unknown, path: string) {
  const item = record(value, path);
  return { x: number(item.x, `${path}.x`), y: number(item.y, `${path}.y`) };
}

function runtimeFixtureContractParser(value: unknown, path: string): CanonicalM0RuntimeFixtureContract {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 2) fail(`${path}.schemaVersion`);
  if (string(item.lane, `${path}.lane`) !== "M0_BINARY_VERTICAL_SLICE") fail(`${path}.lane`);
  const appearance = record(item.appearance, `${path}.appearance`);
  const binaryScene = record(item.binaryScene, `${path}.binaryScene`);
  const fixture = record(binaryScene.fixture, `${path}.binaryScene.fixture`);
  const meshEligibility = record(item.meshEligibility, `${path}.meshEligibility`);
  const contract: CanonicalM0RuntimeFixtureContract = {
    schemaVersion: 2,
    lane: "M0_BINARY_VERTICAL_SLICE",
    module: runtimeResource(item.module, `${path}.module`),
    hak: runtimeResource(item.hak, `${path}.hak`),
    model: runtimeResource(item.model, `${path}.model`),
    texture: runtimeResource(item.texture, `${path}.texture`),
    appearanceTwoDa: runtimeResource(item.appearanceTwoDa, `${path}.appearanceTwoDa`),
    appearance: {
      physicalRow: integer(appearance.physicalRow, `${path}.appearance.physicalRow`),
      label: string(appearance.label, `${path}.appearance.label`),
      modelType: string(appearance.modelType, `${path}.appearance.modelType`),
      race: string(appearance.race, `${path}.appearance.race`),
    },
    binaryScene: {
      moduleResref: string(binaryScene.moduleResref, `${path}.binaryScene.moduleResref`),
      areaResref: string(binaryScene.areaResref, `${path}.binaryScene.areaResref`),
      orderedHakResrefs: stringArray(binaryScene.orderedHakResrefs, `${path}.binaryScene.orderedHakResrefs`),
      entryPosition: position3(binaryScene.entryPosition, `${path}.binaryScene.entryPosition`),
      entryDirection: direction2(binaryScene.entryDirection, `${path}.binaryScene.entryDirection`),
      fixture: {
        templateResref: string(fixture.templateResref, `${path}.binaryScene.fixture.templateResref`),
        appearanceRow: integer(fixture.appearanceRow, `${path}.binaryScene.fixture.appearanceRow`),
        position: position3(fixture.position, `${path}.binaryScene.fixture.position`),
        orientation: direction2(fixture.orientation, `${path}.binaryScene.fixture.orientation`),
      },
    },
    meshEligibility: {
      eligible: boolean(meshEligibility.eligible, `${path}.meshEligibility.eligible`),
      checkedMeshCount: integer(meshEligibility.checkedMeshCount, `${path}.meshEligibility.checkedMeshCount`),
      eligibleMeshCount: integer(meshEligibility.eligibleMeshCount, `${path}.meshEligibility.eligibleMeshCount`),
      meshTypes: array(meshEligibility.meshTypes, `${path}.meshEligibility.meshTypes`).map((entry, index) => integer(entry, `${path}.meshEligibility.meshTypes[${index}]`)),
      textureResrefs: stringArray(meshEligibility.textureResrefs, `${path}.meshEligibility.textureResrefs`),
      rule: string(meshEligibility.rule, `${path}.meshEligibility.rule`),
    },
  };
  if (contract.appearance.physicalRow !== contract.binaryScene.fixture.appearanceRow) {
    throw new Error(`Canonical result identity mismatch at ${path}.appearance.physicalRow`);
  }
  if (contract.module.resref !== contract.binaryScene.moduleResref) {
    throw new Error(`Canonical result identity mismatch at ${path}.module.resref`);
  }
  if (contract.binaryScene.orderedHakResrefs.length !== 1
    || contract.hak.resref !== contract.binaryScene.orderedHakResrefs[0]) {
    throw new Error(`Canonical result identity mismatch at ${path}.binaryScene.orderedHakResrefs`);
  }
  if (contract.meshEligibility.eligibleMeshCount > contract.meshEligibility.checkedMeshCount) {
    throw new Error(`Canonical result identity mismatch at ${path}.meshEligibility.eligibleMeshCount`);
  }
  return contract;
}

function equal(actual: unknown, expected: unknown, path: string) {
  if (actual !== expected) {
    throw new Error(
      `Canonical result identity mismatch at ${path}: expected ${JSON.stringify(expected)}, received ${JSON.stringify(actual)}`,
    );
  }
}

function conversionGate(value: unknown, path: string): CanonicalConversionGate {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    code: string(item.code, `${path}.code`),
    severity: string(item.severity, `${path}.severity`),
    path: string(item.path, `${path}.path`),
    expected: string(item.expected, `${path}.expected`),
    actual: string(item.actual, `${path}.actual`),
    message: string(item.message, `${path}.message`),
  };
}

function conversionDiagnostic(value: unknown, path: string): CanonicalConversionDiagnostic {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    code: string(item.code, `${path}.code`),
    severity: string(item.severity, `${path}.severity`),
    path: string(item.path, `${path}.path`),
    message: string(item.message, `${path}.message`),
  };
}

function optionalString(value: unknown, path: string): string | undefined {
  return value === undefined || value === null ? undefined : string(value, path);
}

function optionalInteger(value: unknown, path: string): number | undefined {
  return value === undefined || value === null ? undefined : integer(value, path);
}

function optionalNumber(value: unknown, path: string): number | undefined {
  return value === undefined || value === null ? undefined : number(value, path);
}

function weaponAnchorAuthoringParser(
  value: unknown,
  path: string,
): CanonicalWeaponAnchorAuthoringReport {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  const status = string(item.status, `${path}.status`);
  if (status !== "weapon_anchors_ready") fail(`${path}.status`);
  const anchors = array(item.anchors, `${path}.anchors`).map((value, index) => {
    const anchorPath = `${path}.anchors[${index}]`;
    const anchor = record(value, anchorPath);
    const dispositionValue = string(anchor.disposition, `${anchorPath}.disposition`);
    if (dispositionValue !== "added" && dispositionValue !== "reused_compatible") {
      fail(`${anchorPath}.disposition`);
    }
    const disposition = dispositionValue as CanonicalWeaponAnchorBinding["disposition"];
    return {
      anchorName: string(anchor.anchorName, `${anchorPath}.anchorName`),
      anchorNodeId: integer(anchor.anchorNodeId, `${anchorPath}.anchorNodeId`),
      parentBoneName: string(anchor.parentBoneName, `${anchorPath}.parentBoneName`),
      parentBoneNodeId: integer(anchor.parentBoneNodeId, `${anchorPath}.parentBoneNodeId`),
      localMatrix: finiteNumberTuple(anchor.localMatrix, 16, `${anchorPath}.localMatrix`),
      disposition,
      weightedVertexCount: integer(anchor.weightedVertexCount, `${anchorPath}.weightedVertexCount`),
    };
  });
  const normalizedNames = anchors.map(({ anchorName }) => anchorName.toLocaleLowerCase()).sort();
  if (normalizedNames.length !== 2 || normalizedNames[0] !== "lhand" || normalizedNames[1] !== "rhand") {
    fail(`${path}.anchors`);
  }
  if (anchors.some(({ weightedVertexCount }) => weightedVertexCount !== 0)) {
    fail(`${path}.anchors.weightedVertexCount`);
  }
  const gripAdjustment = item.gripAdjustment === undefined
    ? undefined
    : weaponGripAdjustmentParser(item.gripAdjustment, `${path}.gripAdjustment`);
  if (gripAdjustment) {
    for (const [anchorName, hand] of [
      ["rhand", gripAdjustment.rightHand],
      ["lhand", gripAdjustment.leftHand],
    ] as const) {
      const anchor = anchors.find((candidate) => candidate.anchorName.toLocaleLowerCase() === anchorName);
      if (!anchor || anchor.localMatrix.some((value, index) => Math.abs(value - hand.finalLocalMatrix[index]!) > 1e-5)) {
        fail(`${path}.gripAdjustment.${anchorName === "rhand" ? "rightHand" : "leftHand"}.finalLocalMatrix`);
      }
    }
  }
  return {
    schemaVersion: 1,
    status: "weapon_anchors_ready",
    calibration: string(item.calibration, `${path}.calibration`),
    profileSha256Before: sha256(item.profileSha256Before, `${path}.profileSha256Before`),
    profileSha256After: sha256(item.profileSha256After, `${path}.profileSha256After`),
    anchors,
    ...(gripAdjustment ? { gripAdjustment } : {}),
  };
}

function weaponGripAdjustmentParser(
  value: unknown,
  path: string,
): CanonicalWeaponGripAdjustmentReport {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  if (string(item.mode, `${path}.mode`) !== "AUTO_PLUS_OFFSETS") fail(`${path}.mode`);
  const compositionOrder = string(item.compositionOrder, `${path}.compositionOrder`);
  const expectedComposition = "AUTO_X_RZ_YAW_X_RX_PITCH_X_RY_ROLL_LOCAL_ITEM_AXES" as const;
  if (compositionOrder !== expectedComposition) {
    fail(`${path}.compositionOrder`);
  }
  const hand = (handValue: unknown, handPath: string): CanonicalWeaponGripHandReport => {
    const handItem = record(handValue, handPath);
    const requested = record(handItem.requested, `${handPath}.requested`);
    return {
      requested: {
        rollDegrees: number(requested.rollDegrees, `${handPath}.requested.rollDegrees`),
        pitchDegrees: number(requested.pitchDegrees, `${handPath}.requested.pitchDegrees`),
        yawDegrees: number(requested.yawDegrees, `${handPath}.requested.yawDegrees`),
      },
      automaticLocalMatrix: finiteNumberTuple(
        handItem.automaticLocalMatrix,
        16,
        `${handPath}.automaticLocalMatrix`,
      ),
      finalLocalMatrix: finiteNumberTuple(
        handItem.finalLocalMatrix,
        16,
        `${handPath}.finalLocalMatrix`,
      ),
    };
  };
  return {
    schemaVersion: 1,
    mode: "AUTO_PLUS_OFFSETS",
    compositionOrder: expectedComposition,
    rightHand: hand(item.rightHand, `${path}.rightHand`),
    leftHand: hand(item.leftHand, `${path}.leftHand`),
  };
}

function skinAccessoryMetrics(
  value: unknown,
  path: string,
): CanonicalSkinAccessoryDeformationMetrics {
  const item = record(value, path);
  return {
    sampledClipCount: integer(item.sampledClipCount, `${path}.sampledClipCount`),
    sampledPoseCount: integer(item.sampledPoseCount, `${path}.sampledPoseCount`),
    sampledVertexCount: integer(item.sampledVertexCount, `${path}.sampledVertexCount`),
    vertexSamplingMode: string(item.vertexSamplingMode, `${path}.vertexSamplingMode`),
    timeSamplingTruncatedClipCount: integer(
      item.timeSamplingTruncatedClipCount,
      `${path}.timeSamplingTruncatedClipCount`,
    ),
    maxPairDistanceRatio: number(item.maxPairDistanceRatio, `${path}.maxPairDistanceRatio`),
    maxPairDistanceError: number(item.maxPairDistanceError, `${path}.maxPairDistanceError`),
    minAxisAlignment: number(item.minAxisAlignment, `${path}.minAxisAlignment`),
  };
}

function skinAccessoryStabilizationParser(
  value: unknown,
  path: string,
): CanonicalSkinAccessoryStabilizationReport {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 2) fail(`${path}.schemaVersion`);
  const components = array(item.components, `${path}.components`).map((value, index) => {
    const componentPath = `${path}.components[${index}]`;
    const component = record(value, componentPath);
    const centroid = array(component.centroid, `${componentPath}.centroid`);
    if (centroid.length !== 3) fail(`${componentPath}.centroid`);
    return {
      segmentIndex: integer(component.segmentIndex, `${componentPath}.segmentIndex`),
      componentIndex: integer(component.componentIndex, `${componentPath}.componentIndex`),
      triangleCount: integer(component.triangleCount, `${componentPath}.triangleCount`),
      vertexCount: integer(component.vertexCount, `${componentPath}.vertexCount`),
      isPrimaryBody: boolean(component.isPrimaryBody, `${componentPath}.isPrimaryBody`),
      centroid: [
        number(centroid[0], `${componentPath}.centroid[0]`),
        number(centroid[1], `${componentPath}.centroid[1]`),
        number(centroid[2], `${componentPath}.centroid[2]`),
      ] as [number, number, number],
      activeBoneCount: integer(component.activeBoneCount, `${componentPath}.activeBoneCount`),
      dominantBoneName: optionalString(component.dominantBoneName, `${componentPath}.dominantBoneName`),
      dominantBoneShare: number(component.dominantBoneShare, `${componentPath}.dominantBoneShare`),
      riskReasons: stringArray(component.riskReasons, `${componentPath}.riskReasons`),
      action: string(component.action, `${componentPath}.action`),
      selectedBoneId: optionalInteger(component.selectedBoneId, `${componentPath}.selectedBoneId`),
      selectedBoneName: optionalString(component.selectedBoneName, `${componentPath}.selectedBoneName`),
      changedVertexCount: integer(component.changedVertexCount, `${componentPath}.changedVertexCount`),
      before: skinAccessoryMetrics(component.before, `${componentPath}.before`),
      after: skinAccessoryMetrics(component.after, `${componentPath}.after`),
    };
  });
  return {
    schemaVersion: 2,
    mode: string(item.mode, `${path}.mode`),
    auditedClipCount: integer(item.auditedClipCount, `${path}.auditedClipCount`),
    weldTolerance: number(item.weldTolerance, `${path}.weldTolerance`),
    componentCount: integer(item.componentCount, `${path}.componentCount`),
    detachedComponentCount: integer(item.detachedComponentCount, `${path}.detachedComponentCount`),
    riskyComponentCount: integer(item.riskyComponentCount, `${path}.riskyComponentCount`),
    stabilizedComponentCount: integer(item.stabilizedComponentCount, `${path}.stabilizedComponentCount`),
    changedVertexCount: integer(item.changedVertexCount, `${path}.changedVertexCount`),
    components,
    warnings: stringArray(item.warnings, `${path}.warnings`),
  };
}

function creatureDemoParser(value: unknown, path: string): CanonicalCreatureDemoReport {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 2) fail(`${path}.schemaVersion`);
  if (item.heldWeaponReadback !== undefined) fail(`${path}.heldWeaponReadback`);
  const semanticReadbackStatus = string(
    item.semanticReadbackStatus,
    `${path}.semanticReadbackStatus`,
  );
  if (semanticReadbackStatus !== "PASS") fail(`${path}.semanticReadbackStatus`);
  let heldStockWeaponReadback: CanonicalHeldStockWeaponReadback | undefined;
  if (item.heldStockWeaponReadback !== undefined) {
    const held = record(item.heldStockWeaponReadback, `${path}.heldStockWeaponReadback`);
    if (integer(held.schemaVersion, `${path}.heldStockWeaponReadback.schemaVersion`) !== 2) {
      fail(`${path}.heldStockWeaponReadback.schemaVersion`);
    }
    const weapon = record(held.weapon, `${path}.heldStockWeaponReadback.weapon`);
    const resref = string(weapon.resref, `${path}.heldStockWeaponReadback.weapon.resref`);
    const resourceType = integer(
      weapon.resourceType,
      `${path}.heldStockWeaponReadback.weapon.resourceType`,
    );
    const resourceScope = string(
      weapon.resourceScope,
      `${path}.heldStockWeaponReadback.weapon.resourceScope`,
    );
    if (resourceType !== 2025) fail(`${path}.heldStockWeaponReadback.weapon.resourceType`);
    if (resourceScope !== "NWN_BASE_GAME") fail(`${path}.heldStockWeaponReadback.weapon.resourceScope`);
    const fixtures = array(held.fixtures, `${path}.heldStockWeaponReadback.fixtures`);
    if (fixtures.length !== 1) fail(`${path}.heldStockWeaponReadback.fixtures`);
    const fixture = record(fixtures[0], `${path}.heldStockWeaponReadback.fixtures[0]`);
    const handValue = string(fixture.hand, `${path}.heldStockWeaponReadback.fixtures[0].hand`);
    const hand: "right_hand" | "left_hand" = handValue === "right_hand"
      ? "right_hand"
      : handValue === "left_hand"
        ? "left_hand"
        : fail(`${path}.heldStockWeaponReadback.fixtures[0].hand`);
    const equippedItemResref = string(
      fixture.equippedItemResref,
      `${path}.heldStockWeaponReadback.fixtures[0].equippedItemResref`,
    );
    if (equippedItemResref !== resref) {
      fail(`${path}.heldStockWeaponReadback.fixtures[0].equippedItemResref`);
    }
    heldStockWeaponReadback = {
      schemaVersion: 2,
      weapon: {
        resref,
        resourceType: 2025,
        resourceScope: "NWN_BASE_GAME",
      },
      fixtures: [{ hand, equippedItemResref }],
    };
  }
  return {
    schemaVersion: 2,
    moduleResref: string(item.moduleResref, `${path}.moduleResref`),
    moduleDisplayName: string(item.moduleDisplayName, `${path}.moduleDisplayName`),
    areaResref: string(item.areaResref, `${path}.areaResref`),
    areaDisplayName: string(item.areaDisplayName, `${path}.areaDisplayName`),
    creatureResref: string(item.creatureResref, `${path}.creatureResref`),
    hakResref: string(item.hakResref, `${path}.hakResref`),
    appearanceRow: integer(item.appearanceRow, `${path}.appearanceRow`),
    resourceCount: integer(item.resourceCount, `${path}.resourceCount`),
    byteLength: integer(item.byteLength, `${path}.byteLength`),
    sha256: sha256(item.sha256, `${path}.sha256`),
    semanticReadbackStatus: "PASS",
    ...(heldStockWeaponReadback ? { heldStockWeaponReadback } : {}),
  };
}

function finiteNumberTuple<const Length extends number>(
  value: unknown,
  length: Length,
  path: string,
): number[] {
  const values = array(value, path);
  if (values.length !== length) fail(path);
  return values.map((entry, index) => number(entry, `${path}[${index}]`));
}

function materialFidelityParser(value: unknown, path: string): CanonicalMaterialFidelityReport {
  const item = record(value, path);
  if (integer(item.schemaVersion, `${path}.schemaVersion`) !== 1) fail(`${path}.schemaVersion`);
  return {
    schemaVersion: 1,
    materialSlot: integer(item.materialSlot, `${path}.materialSlot`),
    sourceMaterialId: integer(item.sourceMaterialId, `${path}.sourceMaterialId`),
    baseColorFactor: finiteNumberTuple(
      item.baseColorFactor,
      4,
      `${path}.baseColorFactor`,
    ) as [number, number, number, number],
    baseColorFactorBaked: boolean(
      item.baseColorFactorBaked,
      `${path}.baseColorFactorBaked`,
    ),
    alphaMode: string(item.alphaMode, `${path}.alphaMode`),
    alphaCutoff: optionalNumber(item.alphaCutoff, `${path}.alphaCutoff`),
    alphaChannelPreserved: boolean(
      item.alphaChannelPreserved,
      `${path}.alphaChannelPreserved`,
    ),
    metallicFactor: number(item.metallicFactor, `${path}.metallicFactor`),
    roughnessFactor: number(item.roughnessFactor, `${path}.roughnessFactor`),
    normalTexturePresent: boolean(
      item.normalTexturePresent,
      `${path}.normalTexturePresent`,
    ),
    emissiveFactor: finiteNumberTuple(
      item.emissiveFactor,
      3,
      `${path}.emissiveFactor`,
    ) as [number, number, number],
    emissiveTexturePresent: boolean(
      item.emissiveTexturePresent,
      `${path}.emissiveTexturePresent`,
    ),
    doubleSided: boolean(item.doubleSided, `${path}.doubleSided`),
    auroraMaterialProfile: string(
      item.auroraMaterialProfile,
      `${path}.auroraMaterialProfile`,
    ),
    mappedFields: stringArray(item.mappedFields, `${path}.mappedFields`),
    unsupportedFields: stringArray(item.unsupportedFields, `${path}.unsupportedFields`),
  };
}

export function projectCanonicalResult(
  reportJson: string,
  summaryJson: string,
  manifestJson: string,
  artifacts: readonly WorkerArtifact[],
  demoReportJson?: string,
): CanonicalResultSnapshot {
  const report = parseJson(reportJson, "reportJson");
  const summary = parseJson(summaryJson, "summaryJson");
  const manifest = parseJson(manifestJson, "manifestJson");
  const status = string(summary.status, "summary.status");
  const productOnly = status === "PROCEDURAL_CREATURE_PRODUCT_MATERIALIZED";
  if (integer(report.schemaVersion, "report.schemaVersion") !== (productOnly ? 4 : 1)) {
    fail("report.schemaVersion");
  }
  for (const [value, path] of [[summary, "summary"], [manifest, "manifest"]] as const) {
    if (integer(value.schemaVersion, `${path}.schemaVersion`) !== (productOnly ? 3 : 1)) {
      fail(`${path}.schemaVersion`);
    }
  }
  if (
    status !== "M6_MODEL_PACKAGE_MATERIALIZED"
    && status !== "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED"
    && !productOnly
  ) {
    fail("summary.status");
  }
  equal(string(manifest.status, "manifest.status"), status, "manifest.status");

  const ingest = record(report.ingest, "report.ingest");
  if (integer(ingest.schemaVersion, "report.ingest.schemaVersion") !== 1) fail("report.ingest.schemaVersion");
  const sourceInventory = record(ingest.inventory, "report.ingest.inventory");
  const sourceStatistics = record(ingest.statistics, "report.ingest.statistics");
  const sourceMetrics: CanonicalModelMetrics = {
    nodes: integer(sourceInventory.nodeCount, "report.ingest.inventory.nodeCount"),
    meshes: integer(sourceInventory.meshCount, "report.ingest.inventory.meshCount"),
    vertices: integer(sourceStatistics.vertexCount, "report.ingest.statistics.vertexCount"),
    triangles: integer(sourceStatistics.triangleCount, "report.ingest.statistics.triangleCount"),
    animations: integer(sourceInventory.animationCount, "report.ingest.inventory.animationCount"),
  };

  const conversion = record(report.conversion, "report.conversion");
  if (integer(conversion.schemaVersion, "report.conversion.schemaVersion") !== 1) fail("report.conversion.schemaVersion");
  const conversionPolicies = record(conversion.policies, "report.conversion.policies");
  const conversionEvidence: CanonicalConversionEvidence = {
    schemaVersion: 1,
    conversionEligible: boolean(conversion.conversionEligible, "report.conversion.conversionEligible"),
    policies: {
      basisStatus: string(conversionPolicies.basisStatus, "report.conversion.policies.basisStatus"),
      assetForwardMapping: string(conversionPolicies.assetForwardMapping, "report.conversion.policies.assetForwardMapping"),
      orientationParity: string(conversionPolicies.orientationParity, "report.conversion.policies.orientationParity"),
      engineFacingProof: string(conversionPolicies.engineFacingProof, "report.conversion.policies.engineFacingProof"),
      uvRuntimeProof: string(conversionPolicies.uvRuntimeProof, "report.conversion.policies.uvRuntimeProof"),
    },
    gates: array(conversion.gates, "report.conversion.gates").map((value, index) =>
      conversionGate(value, `report.conversion.gates[${index}]`)),
    diagnostics: array(conversion.diagnostics, "report.conversion.diagnostics").map((value, index) =>
      conversionDiagnostic(value, `report.conversion.diagnostics[${index}]`)),
  };

  const geometryJson = record(report.geometry, "report.geometry");
  const geometry = {
    vertices: integer(geometryJson.vertexCount, "report.geometry.vertexCount"),
    triangles: integer(geometryJson.triangleCount, "report.geometry.triangleCount"),
    joints: integer(geometryJson.activeJointCount, "report.geometry.activeJointCount"),
    deformation: string(geometryJson.outputSegmentDeformation, "report.geometry.outputSegmentDeformation"),
  };
  const animationJson = record(summary.animation, "summary.animation");
  const animation = {
    sourceName: string(animationJson.sourceName, "summary.animation.sourceName"),
    outputName: string(animationJson.outputName, "summary.animation.outputName"),
    durationSeconds: number(animationJson.durationSeconds, "summary.animation.durationSeconds"),
    hasMotion: boolean(animationJson.hasMotion, "summary.animation.hasMotion"),
  };
  const textureJson = record(report.texture, "report.texture");
  const texture = {
    width: integer(textureJson.width, "report.texture.width"),
    height: integer(textureJson.height, "report.texture.height"),
    pixelFormat: string(textureJson.pixelFormat, "report.texture.pixelFormat"),
    byteLength: integer(textureJson.byteLength, "report.texture.byteLength"),
  };
  const appearanceJson = record(report.appearance, "report.appearance");
  const appendedRow = integer(appearanceJson.appendedRowIndex, "report.appearance.appendedRowIndex");
  equal(integer(summary.appendedPhysicalRow, "summary.appendedPhysicalRow"), appendedRow, "summary.appendedPhysicalRow");
  equal(integer(manifest.appendedPhysicalRow, "manifest.appendedPhysicalRow"), appendedRow, "manifest.appendedPhysicalRow");
  const policy = string(summary.appearancePayloadPolicy, "summary.appearancePayloadPolicy");
  equal(string(manifest.appearancePayloadPolicy, "manifest.appearancePayloadPolicy"), policy, "manifest.appearancePayloadPolicy");

  const outputJson = record(summary.outputs, "summary.outputs");
  const outputNames = productOnly
    ? ["model", "texture", "appearanceTwoDa", "hak", "report"]
    : ["model", "texture", "appearanceTwoDa", "hak", "proofModule", "report"];
  const outputs = Object.fromEntries(
    outputNames.map((name) => [name, identity(outputJson[name], `summary.outputs.${name}`)]),
  );
  const hakJson = record(report.hak, "report.hak");
  const hak = {
    byteLength: integer(hakJson.byteLength, "report.hak.byteLength"),
    sha256: sha256(hakJson.archiveSha256, "report.hak.archiveSha256"),
    entryCount: integer(hakJson.entryCount, "report.hak.entryCount"),
  };
  equal(outputs.hak.byteLength, hak.byteLength, "summary.outputs.hak.byteLength");
  equal(outputs.hak.sha256, hak.sha256, "summary.outputs.hak.sha256");
  equal(texture.byteLength, outputs.texture.byteLength, "report.texture.byteLength");
  equal(sha256(textureJson.outputSha256, "report.texture.outputSha256"), outputs.texture.sha256, "report.texture.outputSha256");
  const modelJson = record(report.model, "report.model");
  const projection = record(modelJson.projection, "report.model.projection");
  const convertedMetrics: CanonicalModelMetrics = {
    nodes: integer(projection.rigNodeCount, "report.model.projection.rigNodeCount")
      + integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
    meshes: integer(projection.meshNodeCount, "report.model.projection.meshNodeCount"),
    vertices: geometry.vertices,
    triangles: integer(projection.triangleCount, "report.model.projection.triangleCount"),
    animations: integer(projection.animationCount, "report.model.projection.animationCount"),
  };
  equal(convertedMetrics.triangles, geometry.triangles, "report.model.projection.triangleCount");
  const semanticDiff = stringArray(modelJson.semanticDiff, "report.model.semanticDiff");
  const deviations = array(modelJson.deviations, "report.model.deviations").map((value, index) => {
    const item = record(value, `report.model.deviations[${index}]`);
    return {
      code: string(item.code, `report.model.deviations[${index}].code`),
      path: string(item.path, `report.model.deviations[${index}].path`),
      message: string(item.message, `report.model.deviations[${index}].message`),
    };
  });
  equal(sha256(modelJson.payloadSha256, "report.model.payloadSha256"), outputs.model.sha256, "report.model.payloadSha256");
  equal(integer(record(modelJson.layout, "report.model.layout").fileLength, "report.model.layout.fileLength"), outputs.model.byteLength, "report.model.layout.fileLength");
  equal(integer(appearanceJson.outputByteLength, "report.appearance.outputByteLength"), outputs.appearanceTwoDa.byteLength, "report.appearance.outputByteLength");
  equal(sha256(appearanceJson.outputSha256, "report.appearance.outputSha256"), outputs.appearanceTwoDa.sha256, "report.appearance.outputSha256");
  let embeddedDemo: CanonicalCreatureDemoReport | undefined;
  if (!productOnly) {
    const proofModuleJson = record(report.proofModule, "report.proofModule");
    if (demoReportJson !== undefined) {
      embeddedDemo = creatureDemoParser(proofModuleJson, "report.proofModule");
    }
    equal(integer(proofModuleJson.byteLength, "report.proofModule.byteLength"), outputs.proofModule.byteLength, "report.proofModule.byteLength");
    equal(sha256(proofModuleJson.sha256, "report.proofModule.sha256"), outputs.proofModule.sha256, "report.proofModule.sha256");
    equal(integer(proofModuleJson.appearanceRow, "report.proofModule.appearanceRow"), appendedRow, "report.proofModule.appearanceRow");
    if (string(proofModuleJson.semanticReadbackStatus, "report.proofModule.semanticReadbackStatus") !== "PASS") fail("report.proofModule.semanticReadbackStatus");
  } else if (report.proofModule !== undefined || outputJson.proofModule !== undefined) {
    fail("report.proofModule");
  }

  let animationEventEvidence: CanonicalAnimationEventEvidence | undefined;
  if (
    report.animationEventConformance !== undefined
    || report.animationEventAuthoringCanonical !== undefined
  ) {
    const eventConformance = record(
      report.animationEventConformance,
      "report.animationEventConformance",
    );
    if (
      integer(
        eventConformance.schemaVersion,
        "report.animationEventConformance.schemaVersion",
      ) !== 1
    ) {
      fail("report.animationEventConformance.schemaVersion");
    }
    const requiredPairCount = integer(
      eventConformance.requiredPairCount,
      "report.animationEventConformance.requiredPairCount",
    );
    const satisfiedPairCount = integer(
      eventConformance.satisfiedPairCount,
      "report.animationEventConformance.satisfiedPairCount",
    );
    const missingPairs = stringArray(
      eventConformance.missingPairs,
      "report.animationEventConformance.missingPairs",
    );
    if (
      !boolean(eventConformance.complete, "report.animationEventConformance.complete")
      || satisfiedPairCount !== requiredPairCount
      || missingPairs.length !== 0
    ) {
      throw new Error("Canonical result identity mismatch at report.animationEventConformance");
    }
    animationEventEvidence = {
      schemaVersion: 1,
      profile: string(
        eventConformance.profile,
        "report.animationEventConformance.profile",
      ),
      requiredPairCount,
      satisfiedPairCount,
      totalEventCount: integer(
        eventConformance.totalEventCount,
        "report.animationEventConformance.totalEventCount",
      ),
      unknownEventNames: stringArray(
        eventConformance.unknownEventNames,
        "report.animationEventConformance.unknownEventNames",
      ),
      missingPairs,
      complete: true,
      authoringCanonical: identity(
        report.animationEventAuthoringCanonical,
        "report.animationEventAuthoringCanonical",
      ),
    };
  }

  const skinAccessoryStabilization = report.skinAccessoryStabilization === undefined
    ? undefined
    : skinAccessoryStabilizationParser(
      report.skinAccessoryStabilization,
      "report.skinAccessoryStabilization",
    );
  const weaponAnchorAuthoring = report.weaponAnchorAuthoring === undefined
    ? undefined
    : weaponAnchorAuthoringParser(
      report.weaponAnchorAuthoring,
      "report.weaponAnchorAuthoring",
    );
  const materialFidelity = report.materialFidelity === undefined
    ? undefined
    : materialFidelityParser(report.materialFidelity, "report.materialFidelity");

  const packageManifest = record(manifest.packageManifest, "manifest.packageManifest");
  equal(sha256(packageManifest.packageSha256, "manifest.packageManifest.packageSha256"), hak.sha256, "manifest.packageManifest.packageSha256");
  const resources = array(packageManifest.resources, "manifest.packageManifest.resources").map((value, index) => {
    const item = record(value, `manifest.packageManifest.resources[${index}]`);
    return {
      role: string(item.role, `manifest.packageManifest.resources[${index}].role`),
      resref: string(item.resref, `manifest.packageManifest.resources[${index}].resref`),
      type: integer(item.type, `manifest.packageManifest.resources[${index}].type`),
      byteLength: integer(item.byteLength, `manifest.packageManifest.resources[${index}].byteLength`),
      sha256: sha256(item.sha256, `manifest.packageManifest.resources[${index}].sha256`),
    };
  });
  if (resources.length !== 3 || resources.length !== hak.entryCount) throw new Error("Canonical result identity mismatch at HAK resource count");
  const resourcesByRole = new Map(resources.map((resource) => [resource.role, resource]));
  if (resourcesByRole.size !== resources.length) throw new Error("Canonical result identity mismatch at duplicate resource role");
  const resource = (role: string) => resourcesByRole.get(role) ?? fail(`manifest.packageManifest.resources.${role}`);
  const modelResource = resource("MODEL");
  const textureResource = resource("TEXTURE");
  const appearanceResource = resource("APPEARANCE_TABLE");
  if (resourcesByRole.size !== 3) fail("manifest.packageManifest.resources.roles");
  const reconcile = (actual: { byteLength: number; sha256: string }, expected: { byteLength: number; sha256: string }, path: string) => {
    equal(actual.byteLength, expected.byteLength, `${path}.byteLength`);
    equal(actual.sha256, expected.sha256, `${path}.sha256`);
  };
  reconcile(modelResource, outputs.model, "manifest.packageManifest.resources.MODEL");
  reconcile(textureResource, outputs.texture, "manifest.packageManifest.resources.TEXTURE");
  reconcile(appearanceResource, outputs.appearanceTwoDa, "manifest.packageManifest.resources.APPEARANCE_TABLE");
  const productIdentity = productOnly ? record(summary.identity, "summary.identity") : undefined;
  const modelResref = productOnly
    ? string(productIdentity?.modelResref, "summary.identity.modelResref")
    : string(summary.modelResref, "summary.modelResref");
  const textureResref = productOnly
    ? string(productIdentity?.textureResref, "summary.identity.textureResref")
    : string(summary.textureResref, "summary.textureResref");
  equal(modelResource.resref, modelResref, "manifest.packageManifest.resources.MODEL.resref");
  equal(string(projection.modelResourceResref, "report.model.projection.modelResourceResref"), modelResource.resref, "report.model.projection.modelResourceResref");
  equal(textureResource.resref, textureResref, "manifest.packageManifest.resources.TEXTURE.resref");
  equal(appearanceResource.resref, "appearance", "manifest.packageManifest.resources.APPEARANCE_TABLE.resref");

  const demo = demoReportJson === undefined
    ? undefined
    : creatureDemoParser(parseJson(demoReportJson, "demoReportJson"), "demo");
  if (demo) {
    if (productOnly) {
      equal(
        demo.hakResref,
        string(productIdentity?.hakResref, "summary.identity.hakResref"),
        "demo.hakResref",
      );
    } else {
      if (status !== "M6_MODEL_PACKAGE_MATERIALIZED" || embeddedDemo === undefined) {
        fail("demoReportJson");
      }
      if (JSON.stringify(demo) !== JSON.stringify(embeddedDemo)) {
        throw new Error("Canonical result identity mismatch at embedded demo report");
      }
      reconcile(demo, outputs.proofModule, "demo");
    }
    equal(demo.appearanceRow, appendedRow, "demo.appearanceRow");
  }

  let runtimeFixtureContract: CanonicalM0RuntimeFixtureContract | undefined;
  if (status === "M0_MESHY_STATIC_RIGID_PACKAGE_MATERIALIZED") {
    const reportContract = runtimeFixtureContractParser(report.m0RuntimeFixtureContract, "report.m0RuntimeFixtureContract");
    const summaryContract = runtimeFixtureContractParser(summary.m0RuntimeFixtureContract, "summary.m0RuntimeFixtureContract");
    const manifestContract = runtimeFixtureContractParser(manifest.m0RuntimeFixtureContract, "manifest.m0RuntimeFixtureContract");
    if (JSON.stringify(reportContract) !== JSON.stringify(summaryContract)
      || JSON.stringify(reportContract) !== JSON.stringify(manifestContract)) {
      throw new Error("Canonical result identity mismatch at m0RuntimeFixtureContract");
    }
    reconcile(reportContract.module, outputs.proofModule, "m0RuntimeFixtureContract.module");
    reconcile(reportContract.hak, outputs.hak, "m0RuntimeFixtureContract.hak");
    reconcile(reportContract.model, outputs.model, "m0RuntimeFixtureContract.model");
    reconcile(reportContract.texture, outputs.texture, "m0RuntimeFixtureContract.texture");
    reconcile(reportContract.appearanceTwoDa, outputs.appearanceTwoDa, "m0RuntimeFixtureContract.appearanceTwoDa");
    equal(reportContract.model.resref, modelResource.resref, "m0RuntimeFixtureContract.model.resref");
    equal(reportContract.texture.resref, textureResource.resref, "m0RuntimeFixtureContract.texture.resref");
    equal(reportContract.appearanceTwoDa.resref, appearanceResource.resref, "m0RuntimeFixtureContract.appearanceTwoDa.resref");
    equal(reportContract.appearance.physicalRow, appendedRow, "m0RuntimeFixtureContract.appearance.physicalRow");
    if (!reportContract.meshEligibility.eligible) {
      throw new Error("Canonical result identity mismatch at m0RuntimeFixtureContract.meshEligibility.eligible");
    }
    runtimeFixtureContract = reportContract;
  } else if (report.m0RuntimeFixtureContract !== undefined
    || summary.m0RuntimeFixtureContract !== undefined
    || manifest.m0RuntimeFixtureContract !== undefined) {
    fail("m0RuntimeFixtureContract");
  }

  const requiresTextureArtifact = productOnly
    || report.textureArtifactCleanup !== undefined
    || (demo !== undefined && !productOnly);
  const expectedArtifactCount = productOnly
    ? (demo ? 8 : 6)
    : (requiresTextureArtifact ? 7 : 6) + (demo ? 1 : 0);
  if (artifacts.length !== expectedArtifactCount || new Set(artifacts.map(({ artifactId }) => artifactId)).size !== artifacts.length) {
    throw new Error("Canonical result identity mismatch at artifact inventory");
  }
  const requiredArtifacts = [
    ["package-hak", "HAK", outputs.hak],
    ["model-mdl", "MODEL", outputs.model],
    ...(requiresTextureArtifact ? [["texture-tga", "TEXTURE", outputs.texture] as const] : []),
    ...(demo && productOnly ? [["proof-module", "MODULE", demo] as const] : []),
    ["report-json", "JSON_REPORT", outputs.report],
    ...(!productOnly ? [["proof-module", "MODULE", outputs.proofModule] as const] : []),
  ] as const;
  for (const [artifactId, kind, expected] of requiredArtifacts) {
    const artifact = artifacts.find((item) => item.artifactId === artifactId)
      ?? fail(`artifacts.${artifactId}`);
    if (artifact.kind !== kind) fail(`artifacts.${artifactId}.kind`);
    equal(artifact.byteLength, expected.byteLength, `artifacts.${artifactId}.byteLength`);
    equal(artifact.sha256, expected.sha256, `artifacts.${artifactId}.sha256`);
  }
  if (demo) {
    const moduleArtifact = artifacts.find((item) => item.artifactId === "proof-module")
      ?? fail("artifacts.proof-module");
    equal(moduleArtifact.fileName, `${demo.moduleResref}.mod`, "artifacts.proof-module.fileName");
    if (!productOnly) {
      const hakArtifact = artifacts.find((item) => item.artifactId === "package-hak")
        ?? fail("artifacts.package-hak");
      equal(hakArtifact.fileName, `${demo.hakResref}.hak`, "artifacts.package-hak.fileName");
    }
  }
  for (const [artifactId, exactJson] of [
    ["report-json", reportJson],
    ["manifest-json", manifestJson],
    ["summary-json", summaryJson],
    ...(demoReportJson === undefined
      ? []
      : [["demo-report-json", demoReportJson] as const]),
  ] as const) {
    const artifact = artifacts.find((item) => item.artifactId === artifactId)
      ?? fail(`artifacts.${artifactId}`);
    if (artifact.kind !== "JSON_REPORT") fail(`artifacts.${artifactId}.kind`);
    if (new TextDecoder().decode(artifact.bytes) !== exactJson) {
      throw new Error(`Canonical result identity mismatch at artifacts.${artifactId}.bytes`);
    }
  }
  for (const artifact of artifacts) {
    string(artifact.fileName, "artifacts.fileName");
    integer(artifact.byteLength, "artifacts.byteLength");
    sha256(artifact.sha256, "artifacts.sha256");
    if (artifact.provenance !== "M2A_WASM_WORKER") fail(`artifacts.${artifact.artifactId}.provenance`);
    if (artifact.bytes.byteLength !== artifact.byteLength) throw new Error(`Canonical result identity mismatch at artifact ${artifact.artifactId} bytes`);
  }

  return {
    status,
    sourceMetrics,
    convertedMetrics,
    geometry,
    animation,
    texture,
    resrefs: {
      model: modelResref,
      texture: textureResref,
    },
    appearance: {
      appendedRow,
      sourcePrefixPreserved: boolean(appearanceJson.sourcePrefixPreserved, "report.appearance.sourcePrefixPreserved"),
      policy,
    },
    hak,
    outputs,
    resources,
    semanticEvidence: { semanticDiff, deviations },
    conversionEvidence,
    packageAssemblyEvidence: {
      strictReconciled: true,
      resourceCount: resources.length,
      artifactCount: artifacts.length,
    },
    runtimeAcceptance: resolveOwnerRuntimeProofV1(outputs),
    animationEventEvidence,
    weaponAnchorAuthoring,
    skinAccessoryStabilization,
    materialFidelity,
    demo,
    runtimeFixtureContract,
    artifacts: [...artifacts],
    reportJson,
    summaryJson,
    manifestJson,
  };
}
