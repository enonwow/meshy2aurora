import type { StudioTarget } from "../../app/studioSession";
import {
  parseAnimationStudioDocumentV1,
} from "../animation-studio/schema";
import {
  parseCreatureAnimationAuthoringV2,
} from "../animation-studio/mappingV2Persistence";
import type {
  AnimationStudioDocumentV1,
  CreatureAnimationAuthoringV2,
} from "../animation-studio/types";
import type { PlaceableAuthoringDocument } from "../placeable-authoring/types";
import type { TileAuthoringOptions } from "../source/InputsPanel";

export const MESHY2AURORA_PROJECT_SCHEMA_VERSION = 1 as const;

export interface ProjectIdentityV1 {
  readonly schemaVersion: 1;
  readonly projectId: string;
  readonly name: string;
  readonly createdAt: string;
  readonly updatedAt: string;
}

export interface ProjectBuildIdentityV1 {
  readonly schemaVersion: 1;
  readonly projectId: string;
  readonly projectName: string;
  readonly projectRevision: number;
}

export interface ProjectFileReferenceV1 {
  readonly name: string;
  readonly byteLength: number;
  readonly lastModified: number;
  readonly sha256: string;
}

export interface Meshy2AuroraProjectFilesV1 {
  readonly sourceGlb: ProjectFileReferenceV1 | null;
  readonly baseTwoDa: ProjectFileReferenceV1 | null;
  readonly animationEvents: ProjectFileReferenceV1 | null;
}

export interface Meshy2AuroraProjectV1 {
  readonly schemaVersion: 1;
  readonly identity: ProjectIdentityV1;
  readonly revision: number;
  readonly target: StudioTarget;
  readonly files: Meshy2AuroraProjectFilesV1;
  readonly animationMappingV2: CreatureAnimationAuthoringV2 | null;
  readonly animationStudio: AnimationStudioDocumentV1 | null;
  readonly placeableAuthoring: PlaceableAuthoringDocument | null;
  readonly tileOptions: TileAuthoringOptions;
}

export interface ProjectDiagnosticV1 {
  readonly schemaVersion: 1;
  readonly code: string;
  readonly path: string;
  readonly message: string;
  readonly action: string;
}

export type ProjectParseResultV1 =
  | { readonly kind: "VALID"; readonly value: Meshy2AuroraProjectV1 }
  | { readonly kind: "INVALID"; readonly diagnostics: readonly ProjectDiagnosticV1[] };

export interface CreateProjectOptionsV1 {
  readonly projectId?: string;
  readonly name?: string;
  readonly target?: StudioTarget;
  readonly now?: string;
}

const DEFAULT_TILE_OPTIONS: TileAuthoringOptions = {
  terrainName: "Grass",
  surface: "GRASS",
  interior: false,
};

export function createMeshy2AuroraProjectV1(
  options: CreateProjectOptionsV1 = {},
): Meshy2AuroraProjectV1 {
  const now = options.now ?? new Date().toISOString();
  const projectId = options.projectId ?? crypto.randomUUID();
  requireProjectId(projectId, "$.identity.projectId");
  requireDate(now, "$.identity.createdAt");
  return {
    schemaVersion: MESHY2AURORA_PROJECT_SCHEMA_VERSION,
    identity: {
      schemaVersion: 1,
      projectId,
      name: normalizedProjectName(options.name ?? "Untitled project"),
      createdAt: now,
      updatedAt: now,
    },
    revision: 1,
    target: options.target ?? "CREATURE",
    files: {
      sourceGlb: null,
      baseTwoDa: null,
      animationEvents: null,
    },
    animationMappingV2: null,
    animationStudio: null,
    placeableAuthoring: null,
    tileOptions: { ...DEFAULT_TILE_OPTIONS },
  };
}

export function reviseMeshy2AuroraProjectV1(
  project: Meshy2AuroraProjectV1,
  patch: Partial<Omit<Meshy2AuroraProjectV1, "schemaVersion" | "identity" | "revision">>,
  now = new Date().toISOString(),
): Meshy2AuroraProjectV1 {
  requireDate(now, "$.identity.updatedAt");
  return {
    ...project,
    ...patch,
    schemaVersion: 1,
    identity: {
      ...project.identity,
      updatedAt: now,
    },
    revision: project.revision + 1,
  };
}

export function renameMeshy2AuroraProjectV1(
  project: Meshy2AuroraProjectV1,
  name: string,
  now = new Date().toISOString(),
): Meshy2AuroraProjectV1 {
  return {
    ...project,
    identity: {
      ...project.identity,
      name: normalizedProjectName(name),
      updatedAt: now,
    },
    revision: project.revision + 1,
  };
}

export function duplicateMeshy2AuroraProjectV1(
  project: Meshy2AuroraProjectV1,
  options: Pick<CreateProjectOptionsV1, "projectId" | "name" | "now"> = {},
): Meshy2AuroraProjectV1 {
  const now = options.now ?? new Date().toISOString();
  const projectId = options.projectId ?? crypto.randomUUID();
  requireProjectId(projectId, "$.identity.projectId");
  return {
    ...structuredClone(project),
    identity: {
      schemaVersion: 1,
      projectId,
      name: normalizedProjectName(options.name ?? `${project.identity.name} copy`),
      createdAt: now,
      updatedAt: now,
    },
    revision: 1,
  };
}

export function projectFileReferenceV1(
  file: Pick<File, "name" | "size" | "lastModified">,
  sha256: string,
): ProjectFileReferenceV1 {
  requireSha256(sha256, "$.sha256");
  return {
    name: file.name,
    byteLength: file.size,
    lastModified: file.lastModified,
    sha256: sha256.toLowerCase(),
  };
}

export function serializeMeshy2AuroraProjectV1(
  project: Meshy2AuroraProjectV1,
): string {
  const parsed = parseMeshy2AuroraProjectV1(project);
  if (parsed.kind === "INVALID") {
    throw new Error(parsed.diagnostics.map(({ message }) => message).join(" "));
  }
  return JSON.stringify(parsed.value, null, 2);
}

export function projectBuildIdentityV1(
  project: Meshy2AuroraProjectV1,
): ProjectBuildIdentityV1 {
  requireProjectId(project.identity.projectId, "$.identity.projectId");
  requirePositiveInteger(project.revision, "$.revision");
  return {
    schemaVersion: 1,
    projectId: project.identity.projectId,
    projectName: normalizedProjectName(project.identity.name),
    projectRevision: project.revision,
  };
}

export function serializeProjectBuildIdentityV1(
  project: Meshy2AuroraProjectV1,
): string {
  return JSON.stringify(projectBuildIdentityV1(project));
}

export function sameProjectBuildIdentityV1(
  left: ProjectBuildIdentityV1 | undefined,
  right: ProjectBuildIdentityV1 | undefined,
) {
  return left !== undefined
    && right !== undefined
    && left.schemaVersion === right.schemaVersion
    && left.projectId === right.projectId
    && left.projectName === right.projectName
    && left.projectRevision === right.projectRevision;
}

export function parseMeshy2AuroraProjectV1(
  input: string | unknown,
): ProjectParseResultV1 {
  let value: unknown;
  try {
    value = typeof input === "string" ? JSON.parse(input) : input;
  } catch (error) {
    return invalid(
      "M2A-PROJECT-JSON",
      "$",
      `Project backup is not valid JSON: ${errorMessage(error)}`,
      "Choose an unmodified Meshy2Aurora project JSON file.",
    );
  }
  try {
    assertExactObject(value, [
      "schemaVersion",
      "identity",
      "revision",
      "target",
      "files",
      "animationMappingV2",
      "animationStudio",
      "placeableAuthoring",
      "tileOptions",
    ], "$");
    if (value.schemaVersion !== 1) {
      const code = typeof value.schemaVersion === "number" && value.schemaVersion > 1
        ? "M2A-PROJECT-NEWER-SCHEMA"
        : "M2A-PROJECT-SCHEMA";
      throw projectError(
        code,
        "$.schemaVersion",
        `Unsupported project schemaVersion: ${String(value.schemaVersion)}.`,
        "Open the backup with a compatible Studio version. Unknown fields are never rewritten.",
      );
    }
    const identity = parseIdentity(value.identity);
    requirePositiveInteger(value.revision, "$.revision");
    requireEnum(value.target, ["CREATURE", "PLACEABLE", "TILE"], "$.target");
    const files = parseFiles(value.files);
    const animationMappingV2 = parseAnimationMapping(value.animationMappingV2);
    const animationStudio = parseAnimationStudio(value.animationStudio);
    const placeableAuthoring = parsePlaceableAuthoring(value.placeableAuthoring);
    const tileOptions = parseTileOptions(value.tileOptions);
    if (
      files.sourceGlb === null
      && (
        animationMappingV2 !== null
        || animationStudio !== null
        || placeableAuthoring !== null
      )
    ) {
      throw projectError(
        "M2A-PROJECT-SOURCE-REFERENCE-MISSING",
        "$.files.sourceGlb",
        "Source-bound authoring requires a source GLB SHA-256 reference.",
        "Restore the source reference from the same project lineage.",
      );
    }
    if (
      value.target !== "CREATURE"
      && (animationMappingV2 !== null || animationStudio !== null)
    ) {
      throw projectError(
        "M2A-PROJECT-TARGET-MISMATCH",
        "$.target",
        "Creature animation authoring cannot be attached to this project target.",
        "Use a Creature project or remove the Creature authoring documents.",
      );
    }
    if (value.target !== "PLACEABLE" && placeableAuthoring !== null) {
      throw projectError(
        "M2A-PROJECT-TARGET-MISMATCH",
        "$.target",
        "Placeable authoring cannot be attached to this project target.",
        "Use a Placeable project or remove the Placeable authoring document.",
      );
    }
    if (
      animationMappingV2 !== null
      && files.sourceGlb !== null
      && animationMappingV2.sourceRevision !== files.sourceGlb.sha256
    ) {
      throw projectError(
        "M2A-PROJECT-SOURCE-MISMATCH",
        "$.animationMappingV2.sourceRevision",
        "Animation mapping sourceRevision does not match the referenced source GLB.",
        "Restore the exact source-bound project backup.",
      );
    }
    if (
      animationStudio !== null
      && files.sourceGlb !== null
      && animationStudio.sourceRevision !== files.sourceGlb.sha256
    ) {
      throw projectError(
        "M2A-PROJECT-SOURCE-MISMATCH",
        "$.animationStudio.sourceRevision",
        "Animation Studio sourceRevision does not match the referenced source GLB.",
        "Restore the exact source-bound project backup.",
      );
    }
    if (
      placeableAuthoring !== null
      && files.sourceGlb !== null
      && placeableAuthoring.sourceSha256 !== files.sourceGlb.sha256
    ) {
      throw projectError(
        "M2A-PROJECT-SOURCE-MISMATCH",
        "$.placeableAuthoring.sourceSha256",
        "Placeable authoring sourceSha256 does not match the referenced source GLB.",
        "Restore the exact source-bound project backup.",
      );
    }
    return {
      kind: "VALID",
      value: {
        schemaVersion: 1,
        identity,
        revision: value.revision as number,
        target: value.target as StudioTarget,
        files,
        animationMappingV2,
        animationStudio,
        placeableAuthoring,
        tileOptions,
      },
    };
  } catch (error) {
    const parsedDiagnostic: ProjectDiagnosticV1 = isProjectError(error)
      ? error.diagnostic
      : diagnostic(
          "M2A-PROJECT-SCHEMA",
          "$",
          errorMessage(error),
          "Use a valid Meshy2AuroraProjectV1 backup without unknown fields.",
        );
    return { kind: "INVALID", diagnostics: [parsedDiagnostic] };
  }
}

export function projectExportFileNameV1(project: Meshy2AuroraProjectV1): string {
  const slug = project.identity.name
    .normalize("NFKD")
    .replace(/[^\w-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .toLowerCase() || "meshy2aurora-project";
  return `${slug}.m2a-project.json`;
}

function parseIdentity(value: unknown): ProjectIdentityV1 {
  assertExactObject(value, [
    "schemaVersion",
    "projectId",
    "name",
    "createdAt",
    "updatedAt",
  ], "$.identity");
  if (value.schemaVersion !== 1) throw new Error("$.identity.schemaVersion must be 1.");
  requireProjectId(value.projectId, "$.identity.projectId");
  requireString(value.name, "$.identity.name");
  requireDate(value.createdAt, "$.identity.createdAt");
  requireDate(value.updatedAt, "$.identity.updatedAt");
  return value as unknown as ProjectIdentityV1;
}

function parseFiles(value: unknown): Meshy2AuroraProjectFilesV1 {
  assertExactObject(value, [
    "sourceGlb",
    "baseTwoDa",
    "animationEvents",
  ], "$.files");
  return {
    sourceGlb: parseFileReference(value.sourceGlb, "$.files.sourceGlb"),
    baseTwoDa: parseFileReference(value.baseTwoDa, "$.files.baseTwoDa"),
    animationEvents: parseFileReference(
      value.animationEvents,
      "$.files.animationEvents",
    ),
  };
}

function parseFileReference(
  value: unknown,
  path: string,
): ProjectFileReferenceV1 | null {
  if (value === null) return null;
  assertExactObject(value, [
    "name",
    "byteLength",
    "lastModified",
    "sha256",
  ], path);
  requireString(value.name, `${path}.name`);
  requireNonNegativeInteger(value.byteLength, `${path}.byteLength`);
  requireNonNegativeInteger(value.lastModified, `${path}.lastModified`);
  requireSha256(value.sha256, `${path}.sha256`);
  if (
    /[\\/]/.test(value.name as string)
    || value.name === "."
    || value.name === ".."
  ) {
    throw new Error(`${path}.name must be a basename, not an absolute path.`);
  }
  return {
    name: value.name as string,
    byteLength: value.byteLength as number,
    lastModified: value.lastModified as number,
    sha256: (value.sha256 as string).toLowerCase(),
  };
}

function parseAnimationMapping(value: unknown): CreatureAnimationAuthoringV2 | null {
  if (value === null) return null;
  return parseCreatureAnimationAuthoringV2(JSON.stringify(value));
}

function parseAnimationStudio(value: unknown): AnimationStudioDocumentV1 | null {
  if (value === null) return null;
  const parsed = parseAnimationStudioDocumentV1(JSON.stringify(value));
  if (parsed.kind === "INVALID") {
    throw new Error(parsed.diagnostics.map(({ message }) => message).join(" "));
  }
  return parsed.value;
}

function parsePlaceableAuthoring(value: unknown): PlaceableAuthoringDocument | null {
  if (value === null) return null;
  assertExactObject(value, ["schemaVersion", "sourceSha256", "elements"], "$.placeableAuthoring");
  if (value.schemaVersion !== 1) {
    throw new Error("$.placeableAuthoring.schemaVersion must be 1.");
  }
  requireSha256(value.sourceSha256, "$.placeableAuthoring.sourceSha256");
  if (!Array.isArray(value.elements)) {
    throw new Error("$.placeableAuthoring.elements must be an array.");
  }
  value.elements.forEach((element, index) => {
    const path = `$.placeableAuthoring.elements[${index}]`;
    assertExactObject(element, [
      "id",
      "name",
      "kind",
      "source",
      "parentId",
      "transform",
      "flags",
      "deleted",
    ], path);
    requireString(element.id, `${path}.id`);
    requireString(element.name, `${path}.name`);
    requireEnum(
      element.kind,
      ["SOURCE_NODE", "SOURCE_COMPONENT", "COPY", "GROUP"],
      `${path}.kind`,
    );
    if (element.source !== null) {
      assertExactObject(
        element.source,
        ["nodeId", "primitiveId", "componentIndex"],
        `${path}.source`,
      );
      requireNonNegativeInteger(element.source.nodeId, `${path}.source.nodeId`);
      requireNullableNonNegativeInteger(
        element.source.primitiveId,
        `${path}.source.primitiveId`,
      );
      requireNullableNonNegativeInteger(
        element.source.componentIndex,
        `${path}.source.componentIndex`,
      );
    }
    if (element.parentId !== null) requireString(element.parentId, `${path}.parentId`);
    assertExactObject(
      element.transform,
      ["translation", "rotationXyzw", "scale", "pivot"],
      `${path}.transform`,
    );
    requireNumberTuple(element.transform.translation, 3, `${path}.transform.translation`);
    requireNumberTuple(element.transform.rotationXyzw, 4, `${path}.transform.rotationXyzw`);
    requireNumberTuple(element.transform.scale, 3, `${path}.transform.scale`);
    requireNumberTuple(element.transform.pivot, 3, `${path}.transform.pivot`);
    const flags = element.flags;
    assertExactObject(
      flags,
      [
        "hidden",
        "locked",
        "renderable",
        "includeInCollision",
        "castShadow",
      ],
      `${path}.flags`,
    );
    [
      "hidden",
      "locked",
      "renderable",
      "includeInCollision",
      "castShadow",
    ].forEach((key) => {
      if (typeof flags[key] !== "boolean") {
        throw new Error(`${path}.flags.${key} must be boolean.`);
      }
    });
    if (typeof element.deleted !== "boolean") {
      throw new Error(`${path}.deleted must be boolean.`);
    }
  });
  // Canonical geometry/source validation remains the worker/Core gate after
  // the exact source file is rebound.
  return structuredClone(value) as unknown as PlaceableAuthoringDocument;
}

function parseTileOptions(value: unknown): TileAuthoringOptions {
  assertExactObject(value, ["terrainName", "surface", "interior"], "$.tileOptions");
  requireString(value.terrainName, "$.tileOptions.terrainName");
  requireEnum(value.surface, ["DIRT", "GRASS", "STONE", "WOOD"], "$.tileOptions.surface");
  if (typeof value.interior !== "boolean") {
    throw new Error("$.tileOptions.interior must be boolean.");
  }
  return value as unknown as TileAuthoringOptions;
}

function normalizedProjectName(name: string): string {
  const normalized = name.trim();
  if (!normalized) throw new RangeError("Project name is required.");
  if (normalized.length > 120) throw new RangeError("Project name is limited to 120 characters.");
  return normalized;
}

function assertExactObject(
  value: unknown,
  keys: readonly string[],
  path: string,
): asserts value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${path} must be an object.`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (
    actual.length !== expected.length
    || actual.some((key, index) => key !== expected[index])
  ) {
    throw new Error(`${path} has missing or unknown fields.`);
  }
}

function requireString(value: unknown, path: string): asserts value is string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${path} must be a non-empty string.`);
  }
}

function requireProjectId(value: unknown, path: string): asserts value is string {
  requireString(value, path);
  if (!/^[a-z0-9][a-z0-9._:-]{2,127}$/i.test(value)) {
    throw new Error(`${path} has an invalid format.`);
  }
}

function requireSha256(value: unknown, path: string): asserts value is string {
  if (typeof value !== "string" || !/^[a-f0-9]{64}$/i.test(value)) {
    throw new Error(`${path} must be a SHA-256.`);
  }
}

function requireDate(value: unknown, path: string): asserts value is string {
  if (typeof value !== "string" || !Number.isFinite(Date.parse(value))) {
    throw new Error(`${path} must be an ISO date.`);
  }
}

function requirePositiveInteger(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || Number(value) < 1) {
    throw new Error(`${path} must be a positive integer.`);
  }
}

function requireNonNegativeInteger(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || Number(value) < 0) {
    throw new Error(`${path} must be a non-negative integer.`);
  }
}

function requireNullableNonNegativeInteger(value: unknown, path: string) {
  if (value === null) return;
  requireNonNegativeInteger(value, path);
}

function requireNumberTuple(value: unknown, length: number, path: string) {
  if (
    !Array.isArray(value)
    || value.length !== length
    || value.some((item) => typeof item !== "number" || !Number.isFinite(item))
  ) {
    throw new Error(`${path} must contain ${length} finite numbers.`);
  }
}

function requireEnum<const T extends string>(
  value: unknown,
  allowed: readonly T[],
  path: string,
): asserts value is T {
  if (typeof value !== "string" || !allowed.includes(value as T)) {
    throw new Error(`${path} must be one of ${allowed.join(", ")}.`);
  }
}

function invalid(
  code: string,
  path: string,
  message: string,
  action: string,
): ProjectParseResultV1 {
  return { kind: "INVALID", diagnostics: [diagnostic(code, path, message, action)] };
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
): ProjectDiagnosticV1 {
  return { schemaVersion: 1, code, path, message, action };
}

interface ProjectSchemaError extends Error {
  readonly diagnostic: ProjectDiagnosticV1;
}

function projectError(
  code: string,
  path: string,
  message: string,
  action: string,
): ProjectSchemaError {
  const error = new Error(message) as ProjectSchemaError;
  Object.defineProperty(error, "diagnostic", {
    value: diagnostic(code, path, message, action),
    enumerable: true,
  });
  return error;
}

function isProjectError(error: unknown): error is ProjectSchemaError {
  return error instanceof Error
    && "diagnostic" in error
    && typeof (error as Partial<ProjectSchemaError>).diagnostic === "object";
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
