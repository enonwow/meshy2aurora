type JsonRecord = Record<string, unknown>;

export interface SourceIdentity {
  format: string;
  byteLength: number;
  sha256: string;
  assetVersion: string;
  generator: string | null;
}

export interface SourceInventory {
  sceneCount: number;
  nodeCount: number;
  meshCount: number;
  primitiveCount: number;
  materialCount: number;
  textureCount: number;
  samplerCount: number;
  imageCount: number;
  skinCount: number;
  jointReferenceCount: number;
  animationCount: number;
  keyframeCount: number;
}

export interface SourceStatistics {
  vertexCount: number;
  indexCount: number;
  triangleCount: number;
  boundsMin: [number, number, number] | null;
  boundsMax: [number, number, number] | null;
  primitivesMissingNormals: number;
  primitivesMissingUv0: number;
  nonTrianglePrimitives: number;
}

export interface SourceAnimationClip {
  id: number;
  name: string | null;
  durationSeconds: number;
  samplerCount: number;
  channelCount: number;
  keyframeCount: number;
  targetNodeIds: number[];
  targetPaths: string[];
}

export interface SourceGate {
  code: string;
  severity: string;
  path: string;
  expected: string;
  actual: string;
  message: string;
}

export interface SourceDiagnostic {
  schemaVersion: 1;
  code: string;
  severity: string;
  byteOffset: number | null;
  jsonPath: string | null;
  message: string;
}

export interface SourceInspectionSnapshot {
  schemaVersion: 1;
  source: SourceIdentity;
  inventory: SourceInventory;
  statistics: SourceStatistics;
  boneCount: number;
  clips: SourceAnimationClip[];
  gates: SourceGate[];
  diagnostics: SourceDiagnostic[];
  conversionEligible: boolean;
}

export interface SourceInspectionFailure {
  schemaVersion: 1;
  code: string;
  message: string;
  byteOffset: number | null;
  jsonPath: string | null;
}

export type SourceInspectionProjection =
  | { kind: "READY"; snapshot: SourceInspectionSnapshot }
  | { kind: "FAILED"; failure: SourceInspectionFailure };

const fail = (path: string): never => {
  throw new Error(`Source inspection field ${path} is missing or has the wrong type`);
};

const record = (value: unknown, path: string): JsonRecord =>
  value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as JsonRecord
    : fail(path);

const array = (value: unknown, path: string): unknown[] =>
  Array.isArray(value) ? value : fail(path);

const string = (value: unknown, path: string): string =>
  typeof value === "string" ? value : fail(path);

const nonEmptyString = (value: unknown, path: string): string => {
  const parsed = string(value, path);
  return parsed.length > 0 ? parsed : fail(path);
};

const boolean = (value: unknown, path: string): boolean =>
  typeof value === "boolean" ? value : fail(path);

const finiteNumber = (value: unknown, path: string): number =>
  typeof value === "number" && Number.isFinite(value) ? value : fail(path);

const nonNegativeInteger = (value: unknown, path: string): number =>
  Number.isSafeInteger(value) && (value as number) >= 0 ? value as number : fail(path);

const nullableString = (value: unknown, path: string): string | null =>
  value === null ? null : string(value, path);

const optionalString = (value: unknown, path: string): string | null =>
  value === undefined || value === null ? null : string(value, path);

const optionalInteger = (value: unknown, path: string): number | null =>
  value === undefined || value === null ? null : nonNegativeInteger(value, path);

const sha256 = (value: unknown, path: string): string => {
  const parsed = nonEmptyString(value, path);
  return /^[0-9a-f]{64}$/.test(parsed) ? parsed : fail(path);
};

const schemaVersion = (value: unknown, path: string): 1 =>
  nonNegativeInteger(value, path) === 1 ? 1 : fail(path);

const vec3OrNull = (value: unknown, path: string): [number, number, number] | null => {
  if (value === null) return null;
  const entries = array(value, path);
  if (entries.length !== 3) fail(path);
  return [
    finiteNumber(entries[0], `${path}[0]`),
    finiteNumber(entries[1], `${path}[1]`),
    finiteNumber(entries[2], `${path}[2]`),
  ];
};

const equal = (actual: unknown, expected: unknown, path: string): void => {
  if (actual !== expected) {
    throw new Error(`Source inspection identity mismatch at ${path}`);
  }
};

function parseJson(ingestJson: string): JsonRecord {
  try {
    return record(JSON.parse(ingestJson), "ingestJson");
  } catch (error) {
    if (error instanceof Error && error.message.startsWith("Source inspection field")) throw error;
    return fail("ingestJson");
  }
}

function projectFailure(root: JsonRecord): SourceInspectionProjection {
  if (root.ir !== undefined || root.report !== undefined) fail("ingestJson");
  return {
    kind: "FAILED",
    failure: {
      schemaVersion: schemaVersion(root.schemaVersion, "ingestJson.schemaVersion"),
      code: nonEmptyString(root.code, "ingestJson.code"),
      message: nonEmptyString(root.message, "ingestJson.message"),
      byteOffset: optionalInteger(root.byteOffset, "ingestJson.byteOffset"),
      jsonPath: optionalString(root.jsonPath, "ingestJson.jsonPath"),
    },
  };
}

function projectInventory(value: unknown, path: string): SourceInventory {
  const inventory = record(value, path);
  return {
    sceneCount: nonNegativeInteger(inventory.sceneCount, `${path}.sceneCount`),
    nodeCount: nonNegativeInteger(inventory.nodeCount, `${path}.nodeCount`),
    meshCount: nonNegativeInteger(inventory.meshCount, `${path}.meshCount`),
    primitiveCount: nonNegativeInteger(inventory.primitiveCount, `${path}.primitiveCount`),
    materialCount: nonNegativeInteger(inventory.materialCount, `${path}.materialCount`),
    textureCount: nonNegativeInteger(inventory.textureCount, `${path}.textureCount`),
    samplerCount: nonNegativeInteger(inventory.samplerCount, `${path}.samplerCount`),
    imageCount: nonNegativeInteger(inventory.imageCount, `${path}.imageCount`),
    skinCount: nonNegativeInteger(inventory.skinCount, `${path}.skinCount`),
    jointReferenceCount: nonNegativeInteger(inventory.jointReferenceCount, `${path}.jointReferenceCount`),
    animationCount: nonNegativeInteger(inventory.animationCount, `${path}.animationCount`),
    keyframeCount: nonNegativeInteger(inventory.keyframeCount, `${path}.keyframeCount`),
  };
}

function projectStatistics(value: unknown, path: string): SourceStatistics {
  const statistics = record(value, path);
  return {
    vertexCount: nonNegativeInteger(statistics.vertexCount, `${path}.vertexCount`),
    indexCount: nonNegativeInteger(statistics.indexCount, `${path}.indexCount`),
    triangleCount: nonNegativeInteger(statistics.triangleCount, `${path}.triangleCount`),
    boundsMin: vec3OrNull(statistics.boundsMin, `${path}.boundsMin`),
    boundsMax: vec3OrNull(statistics.boundsMax, `${path}.boundsMax`),
    primitivesMissingNormals: nonNegativeInteger(statistics.primitivesMissingNormals, `${path}.primitivesMissingNormals`),
    primitivesMissingUv0: nonNegativeInteger(statistics.primitivesMissingUv0, `${path}.primitivesMissingUv0`),
    nonTrianglePrimitives: nonNegativeInteger(statistics.nonTrianglePrimitives, `${path}.nonTrianglePrimitives`),
  };
}

function projectClip(value: unknown, index: number): SourceAnimationClip {
  const path = `ingestJson.ir.animations[${index}]`;
  const animation = record(value, path);
  const samplers = array(animation.samplers, `${path}.samplers`);
  const channels = array(animation.channels, `${path}.channels`);
  const targetNodeIds: number[] = [];
  const targetPaths: string[] = [];

  let keyframeCount = 0;
  samplers.forEach((value, samplerIndex) => {
    const samplerPath = `${path}.samplers[${samplerIndex}]`;
    const sampler = record(value, samplerPath);
    nonNegativeInteger(sampler.id, `${samplerPath}.id`);
    nonEmptyString(sampler.interpolation, `${samplerPath}.interpolation`);
    const inputTimes = array(sampler.inputTimesSeconds, `${samplerPath}.inputTimesSeconds`);
    inputTimes.forEach((time, timeIndex) => finiteNumber(time, `${samplerPath}.inputTimesSeconds[${timeIndex}]`));
    keyframeCount += inputTimes.length;
  });

  channels.forEach((value, channelIndex) => {
    const channelPath = `${path}.channels[${channelIndex}]`;
    const channel = record(value, channelPath);
    nonNegativeInteger(channel.samplerId, `${channelPath}.samplerId`);
    targetNodeIds.push(nonNegativeInteger(channel.targetNodeId, `${channelPath}.targetNodeId`));
    targetPaths.push(nonEmptyString(channel.targetPath, `${channelPath}.targetPath`));
  });

  return {
    id: nonNegativeInteger(animation.id, `${path}.id`),
    name: nullableString(animation.name, `${path}.name`),
    durationSeconds: finiteNumber(animation.durationSeconds, `${path}.durationSeconds`),
    samplerCount: samplers.length,
    channelCount: channels.length,
    keyframeCount,
    targetNodeIds,
    targetPaths,
  };
}

function projectGate(value: unknown, index: number): SourceGate {
  const path = `ingestJson.report.gates[${index}]`;
  const gate = record(value, path);
  return {
    code: nonEmptyString(gate.code, `${path}.code`),
    severity: nonEmptyString(gate.severity, `${path}.severity`),
    path: string(gate.path, `${path}.path`),
    expected: string(gate.expected, `${path}.expected`),
    actual: string(gate.actual, `${path}.actual`),
    message: nonEmptyString(gate.message, `${path}.message`),
  };
}

function projectDiagnostic(value: unknown, index: number): SourceDiagnostic {
  const path = `ingestJson.report.diagnostics[${index}]`;
  const diagnostic = record(value, path);
  return {
    schemaVersion: schemaVersion(diagnostic.schemaVersion, `${path}.schemaVersion`),
    code: nonEmptyString(diagnostic.code, `${path}.code`),
    severity: nonEmptyString(diagnostic.severity, `${path}.severity`),
    byteOffset: optionalInteger(diagnostic.byteOffset, `${path}.byteOffset`),
    jsonPath: optionalString(diagnostic.jsonPath, `${path}.jsonPath`),
    message: nonEmptyString(diagnostic.message, `${path}.message`),
  };
}

function reconcileCount(actual: number, value: unknown, path: string): void {
  equal(actual, array(value, path).length, path);
}

function projectReady(root: JsonRecord): SourceInspectionProjection {
  const ir = record(root.ir, "ingestJson.ir");
  const report = record(root.report, "ingestJson.report");
  const sourceJson = record(ir.source, "ingestJson.ir.source");
  const reportInput = record(report.input, "ingestJson.report.input");
  const source: SourceIdentity = {
    format: nonEmptyString(sourceJson.format, "ingestJson.ir.source.format"),
    byteLength: nonNegativeInteger(sourceJson.byteLength, "ingestJson.ir.source.byteLength"),
    sha256: sha256(sourceJson.sha256, "ingestJson.ir.source.sha256"),
    assetVersion: nonEmptyString(sourceJson.assetVersion, "ingestJson.ir.source.assetVersion"),
    generator: nullableString(sourceJson.generator, "ingestJson.ir.source.generator"),
  };

  schemaVersion(root.schemaVersion, "ingestJson.schemaVersion");
  schemaVersion(ir.schemaVersion, "ingestJson.ir.schemaVersion");
  schemaVersion(report.schemaVersion, "ingestJson.report.schemaVersion");
  equal(source.format, nonEmptyString(report.format, "ingestJson.report.format"), "report.format");
  equal(source.byteLength, nonNegativeInteger(reportInput.byteLength, "ingestJson.report.input.byteLength"), "report.input.byteLength");
  equal(source.sha256, sha256(reportInput.sha256, "ingestJson.report.input.sha256"), "report.input.sha256");

  const inventory = projectInventory(report.inventory, "ingestJson.report.inventory");
  reconcileCount(inventory.sceneCount, ir.scenes, "ingestJson.ir.scenes");
  reconcileCount(inventory.nodeCount, ir.nodes, "ingestJson.ir.nodes");
  reconcileCount(inventory.meshCount, ir.meshes, "ingestJson.ir.meshes");
  reconcileCount(inventory.primitiveCount, ir.primitives, "ingestJson.ir.primitives");
  reconcileCount(inventory.materialCount, ir.materials, "ingestJson.ir.materials");
  reconcileCount(inventory.textureCount, ir.textures, "ingestJson.ir.textures");
  reconcileCount(inventory.samplerCount, ir.samplers, "ingestJson.ir.samplers");
  reconcileCount(inventory.imageCount, ir.images, "ingestJson.ir.images");
  reconcileCount(inventory.skinCount, ir.skins, "ingestJson.ir.skins");

  const animations = array(ir.animations, "ingestJson.ir.animations");
  equal(inventory.animationCount, animations.length, "ir.animations");
  const clips = animations.map(projectClip);
  equal(inventory.keyframeCount, clips.reduce((sum, clip) => sum + clip.keyframeCount, 0), "report.inventory.keyframeCount");

  const skins = array(ir.skins, "ingestJson.ir.skins");
  const uniqueJointNodeIds = new Set<number>();
  const jointReferenceCount = skins.reduce<number>((sum, value, index) => {
    const skin = record(value, `ingestJson.ir.skins[${index}]`);
    const jointNodeIds = array(skin.jointNodeIds, `ingestJson.ir.skins[${index}].jointNodeIds`);
    jointNodeIds.forEach((jointNodeId, jointIndex) => {
      uniqueJointNodeIds.add(nonNegativeInteger(
        jointNodeId,
        `ingestJson.ir.skins[${index}].jointNodeIds[${jointIndex}]`,
      ));
    });
    return sum + jointNodeIds.length;
  }, 0);
  equal(inventory.jointReferenceCount, jointReferenceCount, "report.inventory.jointReferenceCount");

  return {
    kind: "READY",
    snapshot: {
      schemaVersion: 1,
      source,
      inventory,
      statistics: projectStatistics(report.statistics, "ingestJson.report.statistics"),
      boneCount: uniqueJointNodeIds.size,
      clips,
      gates: array(report.gates, "ingestJson.report.gates").map(projectGate),
      diagnostics: array(report.diagnostics, "ingestJson.report.diagnostics").map(projectDiagnostic),
      conversionEligible: boolean(report.conversionEligible, "ingestJson.report.conversionEligible"),
    },
  };
}

/** Projects the exact JSON returned in `SOURCE_INSPECTED.ingestJson`. */
export function projectSourceInspection(ingestJson: string): SourceInspectionProjection {
  const root = parseJson(ingestJson);
  if (root.code !== undefined && (root.ir !== undefined || root.report !== undefined)) {
    return fail("ingestJson");
  }
  if (root.ir !== undefined || root.report !== undefined) return projectReady(root);
  if (root.code !== undefined) return projectFailure(root);
  return fail("ingestJson");
}
