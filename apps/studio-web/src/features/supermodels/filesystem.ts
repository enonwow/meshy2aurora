import type { StudioWorkerRequest, StudioWorkerResponse } from "../../worker/types";
import {
  parseMdlCatalogHeaderBatchV1,
  parseHakModelIndexV1,
  parseNwnBifIndexPlanV1,
  parseNwnBifIndexV1,
  parseNwnKeyModelIndexV1,
  parseSupermodelCatalogV1,
  type MdlCatalogHeaderBatchDescriptorV1,
  type NwnKeyModelLocatorV1,
  type SupermodelCatalogBuildInputV1,
  type SupermodelCatalogV1,
  type SupermodelModelV1,
} from "./types";

const MDL_CATALOG_PREFIX_BYTES = 12 + 0xe8;
const HEADER_BATCH_SIZE = 128;
const MAX_ASCII_CATALOG_BYTES = 64 * 1024 * 1024;

export interface SupermodelWorkerClientV1 {
  request(
    request: StudioWorkerRequest,
    transfer?: Transferable[],
  ): Promise<StudioWorkerResponse>;
}

export interface SupermodelDirectoryFileHandleV1 {
  getFile(): Promise<File>;
}

export interface SupermodelDirectoryHandleV1 {
  readonly name: string;
  getDirectoryHandle(name: string): Promise<SupermodelDirectoryHandleV1>;
  getFileHandle(name: string): Promise<SupermodelDirectoryFileHandleV1>;
}

export interface NwnResourceSourceV1 {
  readonly label: string;
  keyFile(): Promise<File>;
  bifFile(logicalName: string): Promise<File>;
}

export interface SupermodelCatalogProgressV1 {
  stage: "KEY" | "BIF_INDEX" | "MDL_HEADERS" | "CATALOG";
  scannedModels: number;
  totalModels: number;
  currentContainer?: string;
}

export interface SupermodelCatalogSessionV1 {
  sourceId: string;
  sourceLabel: string;
  catalog: SupermodelCatalogV1;
  models: SupermodelModelV1[];
  filesByContainer: Map<string, File>;
}

function resourceFileKey(sourceId: string, containerName: string) {
  return `${sourceId}\u0000${containerName}`;
}

interface HeaderSliceV1 {
  itemId: string;
  declaredPayloadSize: number;
  bytes: ArrayBuffer;
}

interface PendingModelV1 {
  locator: NwnKeyModelLocatorV1;
  containerName: string;
  payloadOffset: number;
  payloadSize: number;
  file: File;
}

let requestSequence = 0;
function requestId() {
  requestSequence += 1;
  return `supermodel-catalog-${requestSequence}`;
}

function normalizedRelativePath(value: string) {
  const path = value.replaceAll("\\", "/").replace(/^\.\//, "");
  const parts = path.split("/").filter(Boolean);
  if (parts.length === 0 || parts.some((part) => part === "." || part === "..")) {
    throw new Error(`Unsafe local resource path: ${value}`);
  }
  return parts;
}

async function fileAtPath(root: SupermodelDirectoryHandleV1, parts: readonly string[]) {
  let directory = root;
  for (const part of parts.slice(0, -1)) {
    directory = await directory.getDirectoryHandle(part);
  }
  return (await directory.getFileHandle(parts.at(-1)!)).getFile();
}

async function firstFileAtPaths(root: SupermodelDirectoryHandleV1, candidates: readonly string[][]) {
  let lastError: unknown;
  for (const parts of candidates) {
    try { return await fileAtPath(root, parts); } catch (error) { lastError = error; }
  }
  throw new Error(`Required NWN resource was not selected: ${candidates[0]?.join("/") ?? "unknown"}${lastError instanceof Error ? ` (${lastError.message})` : ""}`);
}

export function directoryNwnResourceSourceV1(root: SupermodelDirectoryHandleV1): NwnResourceSourceV1 {
  return {
    label: root.name,
    keyFile: () => firstFileAtPaths(root, [["nwn_base.key"], ["data", "nwn_base.key"]]),
    bifFile: (logicalName) => {
      const parts = normalizedRelativePath(logicalName);
      const withoutData = parts[0]?.toLocaleLowerCase() === "data" ? parts.slice(1) : parts;
      const withData = parts[0]?.toLocaleLowerCase() === "data" ? parts : ["data", ...parts];
      return firstFileAtPaths(root, [parts, withoutData, withData]);
    },
  };
}

export function fileListNwnResourceSourceV1(files: Iterable<File>): NwnResourceSourceV1 {
  const indexed = new Map<string, File[]>();
  let label = "Selected NWN files";
  for (const file of files) {
    const relative = (file.webkitRelativePath || file.name).replaceAll("\\", "/");
    const parts = normalizedRelativePath(relative);
    if (label === "Selected NWN files" && parts.length > 1) label = parts[0]!;
    const normalized = parts.join("/").toLocaleLowerCase();
    const candidates = new Set([
      normalized,
      parts.slice(1).join("/").toLocaleLowerCase(),
    ]);
    for (const candidate of candidates) {
      const current = indexed.get(candidate) ?? [];
      current.push(file);
      indexed.set(candidate, current);
    }
  }
  const resolve = (logicalName: string) => {
    const normalized = normalizedRelativePath(logicalName).join("/").toLocaleLowerCase();
    const matches = [
      ...(indexed.get(normalized) ?? []),
      ...(indexed.get(`data/${normalized}`) ?? []),
      ...(normalized.startsWith("data/") ? indexed.get(normalized.slice(5)) ?? [] : []),
    ].filter((file, index, all) => all.indexOf(file) === index);
    if (matches.length !== 1) {
      throw new Error(matches.length === 0
        ? `Required NWN resource was not selected: ${logicalName}`
        : `Selected NWN resource is ambiguous: ${logicalName}`);
    }
    return matches[0]!;
  };
  return {
    label,
    keyFile: async () => {
      const candidates = ["data/nwn_base.key", "nwn_base.key"]
        .flatMap((path) => indexed.get(path) ?? [])
        .filter((file, index, all) => all.indexOf(file) === index);
      if (candidates.length !== 1) {
        throw new Error(candidates.length === 0
          ? "nwn_base.key was not selected"
          : "More than one nwn_base.key was selected");
      }
      return candidates[0]!;
    },
    bifFile: async (logicalName) => resolve(logicalName),
  };
}

export function mergeCatalogHeaderSlicesV1(slices: readonly HeaderSliceV1[]) {
  const total = slices.reduce((sum, item) => sum + item.bytes.byteLength, 0);
  const payload = new Uint8Array(total);
  const descriptors: MdlCatalogHeaderBatchDescriptorV1[] = [];
  let byteOffset = 0;
  for (const item of slices) {
    const bytes = new Uint8Array(item.bytes);
    payload.set(bytes, byteOffset);
    descriptors.push({
      itemId: item.itemId,
      byteOffset,
      byteLength: bytes.byteLength,
      declaredPayloadSize: item.declaredPayloadSize,
    });
    byteOffset += bytes.byteLength;
  }
  return { payloadBlob: payload.buffer, descriptors };
}

async function exactFileRange(file: File, offset: number, length: number) {
  if (!Number.isSafeInteger(offset) || !Number.isSafeInteger(length) || offset < 0 || length < 0 || offset + length > file.size) {
    throw new Error(`Resource range ${offset}..${offset + length} escapes ${file.name} (${file.size} bytes)`);
  }
  const bytes = await file.slice(offset, offset + length).arrayBuffer();
  if (bytes.byteLength !== length) throw new Error(`Short local read from ${file.name}`);
  return bytes;
}

function requireResponse<T extends StudioWorkerResponse["type"]>(
  response: StudioWorkerResponse,
  expected: T,
): Extract<StudioWorkerResponse, { ok: true; type: T }> {
  if (!response.ok) throw new Error(response.message);
  if (response.type !== expected) throw new Error(`Unexpected Studio Worker response: ${response.type}`);
  return response as Extract<StudioWorkerResponse, { ok: true; type: T }>;
}

function assertNotAborted(signal?: AbortSignal) {
  if (signal?.aborted) throw new DOMException("Supermodel catalog scan was cancelled", "AbortError");
}

async function inspectHeaderSlices(
  worker: SupermodelWorkerClientV1,
  slices: readonly HeaderSliceV1[],
) {
  const merged = mergeCatalogHeaderSlicesV1(slices);
  const response = requireResponse(await worker.request({
    requestId: requestId(),
    type: "INSPECT_MDL_CATALOG_HEADERS",
    payloadBlob: merged.payloadBlob,
    descriptorsJson: JSON.stringify(merged.descriptors),
  }, [merged.payloadBlob]), "MDL_CATALOG_HEADERS_INSPECTED");
  return parseMdlCatalogHeaderBatchV1(response.reportJson);
}

async function inspectPendingBatch(
  worker: SupermodelWorkerClientV1,
  pending: readonly PendingModelV1[],
  sourceId: string,
  signal?: AbortSignal,
) {
  const slices = await Promise.all(pending.map(async (item) => ({
    itemId: String(item.locator.keyIndex),
    declaredPayloadSize: item.payloadSize,
    bytes: await exactFileRange(
      item.file,
      item.payloadOffset,
      Math.min(item.payloadSize, MDL_CATALOG_PREFIX_BYTES),
    ),
  })));
  assertNotAborted(signal);
  const firstPass = await inspectHeaderSlices(worker, slices);
  const byId = new Map(firstPass.items.map((item) => [item.itemId, item]));
  const models: SupermodelModelV1[] = [];
  let failures = 0;
  for (const item of pending) {
    const itemId = String(item.locator.keyIndex);
    let inspected = byId.get(itemId);
    if (inspected?.error?.code === "M2A-SUPERMODEL-ASCII-FULL-PAYLOAD-REQUIRED") {
      if (item.payloadSize > MAX_ASCII_CATALOG_BYTES) {
        failures += 1;
        continue;
      }
      const full = await exactFileRange(item.file, item.payloadOffset, item.payloadSize);
      const retry = await inspectHeaderSlices(worker, [{
        itemId,
        declaredPayloadSize: item.payloadSize,
        bytes: full,
      }]);
      inspected = retry.items[0];
    }
    if (!inspected?.header) {
      failures += 1;
      continue;
    }
    models.push({
      resref: item.locator.resref,
      sourceId,
      sourceKind: "BASE_KEY_BIF",
      containerName: item.containerName,
      sourcePriority: 1_000,
      resourceIndex: item.locator.resourceIndex,
      payloadOffset: item.payloadOffset,
      payloadSize: item.payloadSize,
      header: inspected.header,
    });
  }
  return { models, failures };
}

export async function scanNwnSupermodelsV1(
  source: NwnResourceSourceV1,
  worker: SupermodelWorkerClientV1,
  onProgress?: (progress: SupermodelCatalogProgressV1) => void,
  signal?: AbortSignal,
): Promise<SupermodelCatalogSessionV1> {
  assertNotAborted(signal);
  const keyFile = await source.keyFile();
  onProgress?.({ stage: "KEY", scannedModels: 0, totalModels: 0 });
  const keyBytes = await keyFile.arrayBuffer();
  const keyResponse = requireResponse(await worker.request({
    requestId: requestId(),
    type: "INDEX_NWN_KEY_MODELS",
    keyBytes,
  }, [keyBytes]), "NWN_KEY_MODELS_INDEXED");
  const keyIndex = parseNwnKeyModelIndexV1(keyResponse.indexJson);
  const sourceId = `nwn-key:${keyFile.name}:${keyFile.size}:${keyFile.lastModified}`;
  const byBif = new Map<number, NwnKeyModelLocatorV1[]>();
  for (const model of keyIndex.models) {
    const current = byBif.get(model.bifIndex) ?? [];
    current.push(model);
    byBif.set(model.bifIndex, current);
  }

  const filesByContainer = new Map<string, File>();
  const models: SupermodelModelV1[] = [];
  let failures = 0;
  for (const [bifIndex, locators] of [...byBif.entries()].sort((left, right) => left[0] - right[0])) {
    assertNotAborted(signal);
    const bif = keyIndex.bifs[bifIndex];
    if (!bif) {
      failures += locators.length;
      continue;
    }
    onProgress?.({
      stage: "BIF_INDEX",
      scannedModels: models.length + failures,
      totalModels: keyIndex.modelResourceCount,
      currentContainer: bif.logicalName,
    });
    let file: File;
    try {
      file = await source.bifFile(bif.logicalName);
    } catch {
      failures += locators.length;
      continue;
    }
    filesByContainer.set(resourceFileKey(sourceId, bif.logicalName), file);
    const headerBytes = await exactFileRange(file, 0, 20);
    const planResponse = requireResponse(await worker.request({
      requestId: requestId(),
      type: "PLAN_NWN_BIF_INDEX",
      headerBytes,
    }), "NWN_BIF_INDEX_PLANNED");
    const plan = parseNwnBifIndexPlanV1(planResponse.planJson);
    const tableBytes = await exactFileRange(file, plan.tableOffset, plan.tableByteLength);
    const indexResponse = requireResponse(await worker.request({
      requestId: requestId(),
      type: "INDEX_NWN_BIF_TABLE",
      headerBytes,
      tableBytes,
    }, [headerBytes, tableBytes]), "NWN_BIF_TABLE_INDEXED");
    const bifResources = parseNwnBifIndexV1(indexResponse.indexJson).resources;
    const pending = locators.flatMap((locator) => {
      const resource = bifResources[locator.resourceIndex];
      if (!resource || resource.resourceIndex !== locator.resourceIndex || resource.resourceType !== 2002) {
        failures += 1;
        return [];
      }
      return [{
        locator,
        containerName: bif.logicalName,
        payloadOffset: resource.payloadOffset,
        payloadSize: resource.payloadSize,
        file,
      } satisfies PendingModelV1];
    });
    for (let offset = 0; offset < pending.length; offset += HEADER_BATCH_SIZE) {
      assertNotAborted(signal);
      const inspected = await inspectPendingBatch(
        worker,
        pending.slice(offset, offset + HEADER_BATCH_SIZE),
        sourceId,
        signal,
      );
      models.push(...inspected.models);
      failures += inspected.failures;
      onProgress?.({
        stage: "MDL_HEADERS",
        scannedModels: models.length + failures,
        totalModels: keyIndex.modelResourceCount,
        currentContainer: bif.logicalName,
      });
    }
  }
  assertNotAborted(signal);
  onProgress?.({
    stage: "CATALOG",
    scannedModels: models.length + failures,
    totalModels: keyIndex.modelResourceCount,
  });
  const input: SupermodelCatalogBuildInputV1 = {
    declaredModelCount: keyIndex.modelResourceCount,
    failedModelCount: failures,
    models,
  };
  const catalogResponse = requireResponse(await worker.request({
    requestId: requestId(),
    type: "BUILD_SUPERMODEL_CATALOG",
    inputJson: JSON.stringify(input),
  }), "SUPERMODEL_CATALOG_BUILT");
  return {
    sourceId,
    sourceLabel: source.label,
    catalog: parseSupermodelCatalogV1(catalogResponse.catalogJson),
    models,
    filesByContainer,
  };
}

export async function loadSupermodelModelBytesV1(
  session: SupermodelCatalogSessionV1,
  model: SupermodelModelV1,
) {
  const file = session.filesByContainer.get(resourceFileKey(model.sourceId, model.containerName))
    ?? session.filesByContainer.get(model.containerName);
  if (!file) throw new Error(`Supermodel container is unavailable: ${model.containerName}`);
  return exactFileRange(file, model.payloadOffset, model.payloadSize);
}

async function rebuildCatalogSessionV1(
  session: SupermodelCatalogSessionV1,
  worker: SupermodelWorkerClientV1,
  additions: {
    models: SupermodelModelV1[];
    declaredModelCount: number;
    failedModelCount: number;
    files: Map<string, File>;
    label: string;
  },
) {
  const models = [...session.models, ...additions.models];
  const input: SupermodelCatalogBuildInputV1 = {
    declaredModelCount: session.catalog.declaredModelCount + additions.declaredModelCount,
    failedModelCount: session.catalog.failedModelCount + additions.failedModelCount,
    models,
  };
  const response = requireResponse(await worker.request({
    requestId: requestId(),
    type: "BUILD_SUPERMODEL_CATALOG",
    inputJson: JSON.stringify(input),
  }), "SUPERMODEL_CATALOG_BUILT");
  return {
    sourceId: session.sourceId,
    sourceLabel: `${session.sourceLabel} + ${additions.label}`,
    catalog: parseSupermodelCatalogV1(response.catalogJson),
    models,
    filesByContainer: new Map([...session.filesByContainer, ...additions.files]),
  } satisfies SupermodelCatalogSessionV1;
}

export async function addHakFilesToSupermodelCatalogV1(
  session: SupermodelCatalogSessionV1,
  files: readonly File[],
  worker: SupermodelWorkerClientV1,
) {
  const additions: SupermodelModelV1[] = [];
  const catalogFiles = new Map<string, File>();
  let declaredModelCount = 0;
  let failedModelCount = 0;
  const existingHakSources = new Set(
    session.models.filter((model) => model.sourceKind === "HAK").map((model) => model.sourceId),
  ).size;
  for (const [fileIndex, file] of files.entries()) {
    const sourceId = `hak:${file.name}:${file.size}:${file.lastModified}`;
    const bytes = await file.arrayBuffer();
    const response = requireResponse(await worker.request({
      requestId: requestId(),
      type: "INDEX_HAK_MODELS",
      hakBytes: bytes,
    }, [bytes]), "HAK_MODELS_INDEXED");
    const index = parseHakModelIndexV1(response.indexJson);
    declaredModelCount += index.modelResourceCount;
    failedModelCount += index.failedModelCount;
    catalogFiles.set(resourceFileKey(sourceId, file.name), file);
    for (const model of index.models) {
      if (!model.header) continue;
      additions.push({
        resref: model.resref,
        sourceId,
        sourceKind: "HAK",
        containerName: file.name,
        sourcePriority: 100 + existingHakSources + fileIndex,
        resourceIndex: model.resourceIndex,
        payloadOffset: model.payloadOffset,
        payloadSize: model.payloadSize,
        header: model.header,
      });
    }
  }
  return rebuildCatalogSessionV1(session, worker, {
    models: additions,
    declaredModelCount,
    failedModelCount,
    files: catalogFiles,
    label: `${files.length} HAK`,
  });
}

function looseModelResref(file: File) {
  const stem = file.name.replace(/\.mdl$/i, "");
  return /^[A-Za-z0-9_-]{1,16}$/.test(stem) ? stem : undefined;
}

export async function addLooseMdlFilesToSupermodelCatalogV1(
  session: SupermodelCatalogSessionV1,
  files: readonly File[],
  worker: SupermodelWorkerClientV1,
) {
  const valid = files.filter((file) => file.name.toLocaleLowerCase().endsWith(".mdl"));
  const additions: SupermodelModelV1[] = [];
  const catalogFiles = new Map<string, File>();
  let failedModelCount = files.length - valid.length;
  for (let offset = 0; offset < valid.length; offset += HEADER_BATCH_SIZE) {
    const batch = valid.slice(offset, offset + HEADER_BATCH_SIZE);
    const slices = await Promise.all(batch.map(async (file, index) => ({
      itemId: String(offset + index),
      declaredPayloadSize: file.size,
      bytes: await file.slice(0, Math.min(file.size, MDL_CATALOG_PREFIX_BYTES)).arrayBuffer(),
    })));
    const report = await inspectHeaderSlices(worker, slices);
    for (const [index, file] of batch.entries()) {
      const itemId = String(offset + index);
      let inspected = report.items.find((item) => item.itemId === itemId);
      if (inspected?.error?.code === "M2A-SUPERMODEL-ASCII-FULL-PAYLOAD-REQUIRED"
        && file.size <= MAX_ASCII_CATALOG_BYTES) {
        const retry = await inspectHeaderSlices(worker, [{
          itemId,
          declaredPayloadSize: file.size,
          bytes: await file.arrayBuffer(),
        }]);
        inspected = retry.items[0];
      }
      const resref = looseModelResref(file);
      if (!resref || !inspected?.header) {
        failedModelCount += 1;
        continue;
      }
      const relativePath = file.webkitRelativePath || file.name;
      const sourceId = `loose:${relativePath}:${file.size}:${file.lastModified}`;
      catalogFiles.set(resourceFileKey(sourceId, relativePath), file);
      additions.push({
        resref,
        sourceId,
        sourceKind: "OVERRIDE",
        containerName: relativePath,
        sourcePriority: 0,
        resourceIndex: offset + index,
        payloadOffset: 0,
        payloadSize: file.size,
        header: inspected.header,
      });
    }
  }
  return rebuildCatalogSessionV1(session, worker, {
    models: additions,
    declaredModelCount: files.length,
    failedModelCount,
    files: catalogFiles,
    label: `${files.length} loose/override MDL`,
  });
}

export function effectiveCatalogModelV1(session: SupermodelCatalogSessionV1, resref: string) {
  const normalized = resref.trim().toLocaleLowerCase();
  return session.models
    .filter((model) => model.resref.toLocaleLowerCase() === normalized)
    .sort((left, right) => left.sourcePriority - right.sourcePriority)[0];
}

declare global {
  interface Window {
    showDirectoryPicker?: (options?: { mode?: "read" | "readwrite" }) => Promise<SupermodelDirectoryHandleV1>;
  }
}
