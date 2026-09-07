export type SupermodelSourceKindV1 = "BASE_KEY_BIF" | "HAK" | "OVERRIDE" | "LOOSE_MDL";
export type MdlCatalogFormatV1 = "BINARY" | "ASCII";
export type SupermodelCatalogEntryStatusV1 = "RESOLVED" | "MISSING" | "CYCLIC";
export type SupermodelCatalogCompletenessV1 = "COMPLETE" | "PARTIAL";
export type SupermodelCatalogFilterV1 = "ALL" | "WITH_ANIMATIONS" | SupermodelCatalogEntryStatusV1;

export interface NwnKeyBifV1 {
  index: number;
  logicalName: string;
}

export interface NwnKeyModelLocatorV1 {
  keyIndex: number;
  resref: string;
  bifIndex: number;
  resourceIndex: number;
}

export interface NwnKeyModelIndexV1 {
  schemaVersion: 1;
  resourceCount: number;
  modelResourceCount: number;
  bifs: NwnKeyBifV1[];
  models: NwnKeyModelLocatorV1[];
}

export interface NwnBifIndexPlanV1 {
  schemaVersion: 1;
  resourceCount: number;
  tableOffset: number;
  tableByteLength: number;
}

export interface NwnBifResourceV1 {
  resourceIndex: number;
  resourceId: number;
  payloadOffset: number;
  payloadSize: number;
  resourceType: number;
}

export interface NwnBifIndexV1 {
  schemaVersion: 1;
  resources: NwnBifResourceV1[];
}

export interface MdlCatalogHeaderV1 {
  schemaVersion: 1;
  format: MdlCatalogFormatV1;
  modelName: string;
  supermodelName: string;
  classification: number | null;
  animationScale: number;
  localAnimationCount: number;
}

export interface MdlCatalogHeaderBatchDescriptorV1 {
  itemId: string;
  byteOffset: number;
  byteLength: number;
  declaredPayloadSize: number;
}

export interface MdlCatalogHeaderBatchItemV1 {
  itemId: string;
  header?: MdlCatalogHeaderV1;
  error?: { schemaVersion: 1; code: string; offset: number; context: string };
}

export interface MdlCatalogHeaderBatchV1 {
  schemaVersion: 1;
  items: MdlCatalogHeaderBatchItemV1[];
}

export interface HakModelCatalogItemV1 {
  resourceIndex: number;
  resref: string;
  payloadOffset: number;
  payloadSize: number;
  header?: MdlCatalogHeaderV1;
  error?: { schemaVersion: 1; code: string; offset: number; context: string };
}

export interface HakModelIndexV1 {
  schemaVersion: 1;
  resourceCount: number;
  modelResourceCount: number;
  scannedModelCount: number;
  failedModelCount: number;
  models: HakModelCatalogItemV1[];
}

export interface SupermodelModelV1 {
  resref: string;
  sourceId: string;
  sourceKind: SupermodelSourceKindV1;
  containerName: string;
  sourcePriority: number;
  resourceIndex: number;
  payloadOffset: number;
  payloadSize: number;
  header: MdlCatalogHeaderV1;
}

export interface SupermodelCatalogBuildInputV1 {
  declaredModelCount: number;
  failedModelCount: number;
  models: SupermodelModelV1[];
}

export interface SupermodelCatalogEntryV1 {
  resref: string;
  status: SupermodelCatalogEntryStatusV1;
  resource?: SupermodelModelV1;
  definitions: SupermodelModelV1[];
  children: string[];
  chain: string[];
}

export interface SupermodelCatalogV1 {
  schemaVersion: 1;
  completeness: SupermodelCatalogCompletenessV1;
  declaredModelCount: number;
  scannedModelCount: number;
  failedModelCount: number;
  supermodelCount: number;
  entries: SupermodelCatalogEntryV1[];
}

function fail(path: string): never {
  throw new Error(`Invalid supermodel catalog data at ${path}`);
}

function record(value: unknown, path: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) return fail(path);
  return value as Record<string, unknown>;
}

function parseEnvelope(json: string, path: string) {
  let parsed: unknown;
  try { parsed = JSON.parse(json); } catch { return fail(path); }
  const value = record(parsed, path);
  if (typeof value.code === "string") {
    throw new Error(`${value.code}: ${typeof value.context === "string" ? value.context : "operation failed"}`);
  }
  return value;
}

function integer(value: unknown, path: string) {
  if (!Number.isSafeInteger(value) || (value as number) < 0) return fail(path);
  return value as number;
}

function finite(value: unknown, path: string) {
  if (typeof value !== "number" || !Number.isFinite(value)) return fail(path);
  return value;
}

function string(value: unknown, path: string) {
  if (typeof value !== "string") return fail(path);
  return value;
}

function array(value: unknown, path: string) {
  if (!Array.isArray(value)) return fail(path);
  return value;
}

function literal<T extends string>(value: unknown, allowed: readonly T[], path: string): T {
  if (typeof value !== "string" || !allowed.includes(value as T)) return fail(path);
  return value as T;
}

function schema(value: Record<string, unknown>, path: string) {
  if (integer(value.schemaVersion, `${path}.schemaVersion`) !== 1) return fail(`${path}.schemaVersion`);
}

export function parseNwnKeyModelIndexV1(json: string): NwnKeyModelIndexV1 {
  const value = parseEnvelope(json, "keyIndex");
  schema(value, "keyIndex");
  return {
    schemaVersion: 1,
    resourceCount: integer(value.resourceCount, "keyIndex.resourceCount"),
    modelResourceCount: integer(value.modelResourceCount, "keyIndex.modelResourceCount"),
    bifs: array(value.bifs, "keyIndex.bifs").map((entry, index) => {
      const item = record(entry, `keyIndex.bifs[${index}]`);
      return {
        index: integer(item.index, `keyIndex.bifs[${index}].index`),
        logicalName: string(item.logicalName, `keyIndex.bifs[${index}].logicalName`),
      };
    }),
    models: array(value.models, "keyIndex.models").map((entry, index) => {
      const item = record(entry, `keyIndex.models[${index}]`);
      return {
        keyIndex: integer(item.keyIndex, `keyIndex.models[${index}].keyIndex`),
        resref: string(item.resref, `keyIndex.models[${index}].resref`),
        bifIndex: integer(item.bifIndex, `keyIndex.models[${index}].bifIndex`),
        resourceIndex: integer(item.resourceIndex, `keyIndex.models[${index}].resourceIndex`),
      };
    }),
  };
}

export function parseNwnBifIndexPlanV1(json: string): NwnBifIndexPlanV1 {
  const value = parseEnvelope(json, "bifPlan");
  schema(value, "bifPlan");
  return {
    schemaVersion: 1,
    resourceCount: integer(value.resourceCount, "bifPlan.resourceCount"),
    tableOffset: integer(value.tableOffset, "bifPlan.tableOffset"),
    tableByteLength: integer(value.tableByteLength, "bifPlan.tableByteLength"),
  };
}

export function parseNwnBifIndexV1(json: string): NwnBifIndexV1 {
  const value = parseEnvelope(json, "bifIndex");
  schema(value, "bifIndex");
  return {
    schemaVersion: 1,
    resources: array(value.resources, "bifIndex.resources").map((entry, index) => {
      const item = record(entry, `bifIndex.resources[${index}]`);
      return {
        resourceIndex: integer(item.resourceIndex, `bifIndex.resources[${index}].resourceIndex`),
        resourceId: integer(item.resourceId, `bifIndex.resources[${index}].resourceId`),
        payloadOffset: integer(item.payloadOffset, `bifIndex.resources[${index}].payloadOffset`),
        payloadSize: integer(item.payloadSize, `bifIndex.resources[${index}].payloadSize`),
        resourceType: integer(item.resourceType, `bifIndex.resources[${index}].resourceType`),
      };
    }),
  };
}

function parseMdlHeader(value: unknown, path: string): MdlCatalogHeaderV1 {
  const item = record(value, path);
  schema(item, path);
  const classification = item.classification;
  return {
    schemaVersion: 1,
    format: literal(item.format, ["BINARY", "ASCII"], `${path}.format`),
    modelName: string(item.modelName, `${path}.modelName`),
    supermodelName: string(item.supermodelName, `${path}.supermodelName`),
    classification: classification === null ? null : integer(classification, `${path}.classification`),
    animationScale: finite(item.animationScale, `${path}.animationScale`),
    localAnimationCount: integer(item.localAnimationCount, `${path}.localAnimationCount`),
  };
}

export function parseMdlCatalogHeaderBatchV1(json: string): MdlCatalogHeaderBatchV1 {
  const value = parseEnvelope(json, "headerBatch");
  schema(value, "headerBatch");
  return {
    schemaVersion: 1,
    items: array(value.items, "headerBatch.items").map((entry, index) => {
      const item = record(entry, `headerBatch.items[${index}]`);
      const errorValue = item.error;
      const error = errorValue === null || errorValue === undefined ? undefined : record(errorValue, `headerBatch.items[${index}].error`);
      return {
        itemId: string(item.itemId, `headerBatch.items[${index}].itemId`),
        ...(item.header === null || item.header === undefined ? {} : { header: parseMdlHeader(item.header, `headerBatch.items[${index}].header`) }),
        ...(error ? { error: {
          schemaVersion: 1 as const,
          code: string(error.code, `headerBatch.items[${index}].error.code`),
          offset: integer(error.offset, `headerBatch.items[${index}].error.offset`),
          context: string(error.context, `headerBatch.items[${index}].error.context`),
        } } : {}),
      };
    }),
  };
}

export function parseHakModelIndexV1(json: string): HakModelIndexV1 {
  const value = parseEnvelope(json, "hakIndex");
  schema(value, "hakIndex");
  return {
    schemaVersion: 1,
    resourceCount: integer(value.resourceCount, "hakIndex.resourceCount"),
    modelResourceCount: integer(value.modelResourceCount, "hakIndex.modelResourceCount"),
    scannedModelCount: integer(value.scannedModelCount, "hakIndex.scannedModelCount"),
    failedModelCount: integer(value.failedModelCount, "hakIndex.failedModelCount"),
    models: array(value.models, "hakIndex.models").map((entry, index) => {
      const item = record(entry, `hakIndex.models[${index}]`);
      const errorValue = item.error;
      const error = errorValue === null || errorValue === undefined
        ? undefined
        : record(errorValue, `hakIndex.models[${index}].error`);
      return {
        resourceIndex: integer(item.resourceIndex, `hakIndex.models[${index}].resourceIndex`),
        resref: string(item.resref, `hakIndex.models[${index}].resref`),
        payloadOffset: integer(item.payloadOffset, `hakIndex.models[${index}].payloadOffset`),
        payloadSize: integer(item.payloadSize, `hakIndex.models[${index}].payloadSize`),
        ...(item.header === null || item.header === undefined ? {} : { header: parseMdlHeader(item.header, `hakIndex.models[${index}].header`) }),
        ...(error ? { error: {
          schemaVersion: 1 as const,
          code: string(error.code, `hakIndex.models[${index}].error.code`),
          offset: integer(error.offset, `hakIndex.models[${index}].error.offset`),
          context: string(error.context, `hakIndex.models[${index}].error.context`),
        } } : {}),
      };
    }),
  };
}

function parseModel(value: unknown, path: string): SupermodelModelV1 {
  const item = record(value, path);
  return {
    resref: string(item.resref, `${path}.resref`),
    sourceId: string(item.sourceId, `${path}.sourceId`),
    sourceKind: literal(item.sourceKind, ["BASE_KEY_BIF", "HAK", "OVERRIDE", "LOOSE_MDL"], `${path}.sourceKind`),
    containerName: string(item.containerName, `${path}.containerName`),
    sourcePriority: integer(item.sourcePriority, `${path}.sourcePriority`),
    resourceIndex: integer(item.resourceIndex, `${path}.resourceIndex`),
    payloadOffset: integer(item.payloadOffset, `${path}.payloadOffset`),
    payloadSize: integer(item.payloadSize, `${path}.payloadSize`),
    header: parseMdlHeader(item.header, `${path}.header`),
  };
}

export function parseSupermodelCatalogV1(json: string): SupermodelCatalogV1 {
  const value = parseEnvelope(json, "catalog");
  schema(value, "catalog");
  const entries = array(value.entries, "catalog.entries").map((entry, index) => {
    const item = record(entry, `catalog.entries[${index}]`);
    return {
      resref: string(item.resref, `catalog.entries[${index}].resref`),
      status: literal(item.status, ["RESOLVED", "MISSING", "CYCLIC"], `catalog.entries[${index}].status`),
      ...(item.resource === null || item.resource === undefined ? {} : { resource: parseModel(item.resource, `catalog.entries[${index}].resource`) }),
      definitions: array(item.definitions, `catalog.entries[${index}].definitions`).map((definition, definitionIndex) => parseModel(definition, `catalog.entries[${index}].definitions[${definitionIndex}]`)),
      children: array(item.children, `catalog.entries[${index}].children`).map((child, childIndex) => string(child, `catalog.entries[${index}].children[${childIndex}]`)),
      chain: array(item.chain, `catalog.entries[${index}].chain`).map((link, linkIndex) => string(link, `catalog.entries[${index}].chain[${linkIndex}]`)),
    } satisfies SupermodelCatalogEntryV1;
  });
  return {
    schemaVersion: 1,
    completeness: literal(value.completeness, ["COMPLETE", "PARTIAL"], "catalog.completeness"),
    declaredModelCount: integer(value.declaredModelCount, "catalog.declaredModelCount"),
    scannedModelCount: integer(value.scannedModelCount, "catalog.scannedModelCount"),
    failedModelCount: integer(value.failedModelCount, "catalog.failedModelCount"),
    supermodelCount: integer(value.supermodelCount, "catalog.supermodelCount"),
    entries,
  };
}

export function filterSupermodelCatalogEntriesV1(
  entries: readonly SupermodelCatalogEntryV1[],
  query: string,
  filter: SupermodelCatalogFilterV1,
) {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  return entries.filter((entry) => {
    if (normalizedQuery && ![
      entry.resref,
      ...entry.children,
      ...entry.chain,
    ].some((value) => value.toLocaleLowerCase().includes(normalizedQuery))) return false;
    if (filter === "WITH_ANIMATIONS") return (entry.resource?.header.localAnimationCount ?? 0) > 0;
    if (filter !== "ALL") return entry.status === filter;
    return true;
  });
}
