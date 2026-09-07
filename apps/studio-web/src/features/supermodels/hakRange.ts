const HAK_HEADER_BYTES = 160;
const HAK_KEY_ENTRY_BYTES = 24;
const HAK_RESOURCE_ENTRY_BYTES = 8;
const HAK_MDL_RESOURCE_TYPE = 2002;
const HAK_MAX_ENTRY_COUNT = 262_144;

export interface HakRangeIndexPlanV1 {
  readonly entryCount: number;
  readonly keyTableOffset: number;
  readonly keyTableByteLength: number;
  readonly resourceTableOffset: number;
  readonly resourceTableByteLength: number;
}

export interface HakModelResourceLocatorV1 {
  readonly resref: string;
  readonly resourceId: number;
  readonly payloadOffset: number;
  readonly payloadSize: number;
}

export interface HakResourceLocatorV1 extends HakModelResourceLocatorV1 {
  readonly resourceType: number;
}

function exactBytes(buffer: ArrayBuffer, expected: number, context: string) {
  if (buffer.byteLength !== expected) {
    throw new Error(`${context} requires exactly ${expected} bytes; received ${buffer.byteLength}`);
  }
  return new Uint8Array(buffer);
}

function checkedTableLength(count: number, stride: number, context: string) {
  const length = count * stride;
  if (!Number.isSafeInteger(length)) throw new Error(`${context} byte length overflows`);
  return length;
}

export function parseHakRangeIndexPlanV1(headerBuffer: ArrayBuffer): HakRangeIndexPlanV1 {
  const bytes = exactBytes(headerBuffer, HAK_HEADER_BYTES, "HAK header");
  if (new TextDecoder("ascii").decode(bytes.subarray(0, 8)) !== "HAK V1.0") {
    throw new Error("Expected a HAK V1.0 archive");
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const entryCount = view.getUint32(16, true);
  if (entryCount > HAK_MAX_ENTRY_COUNT) {
    throw new Error(`HAK entry count ${entryCount} exceeds the product guardrail`);
  }
  const keyTableOffset = view.getUint32(24, true);
  const resourceTableOffset = view.getUint32(28, true);
  const keyTableByteLength = checkedTableLength(entryCount, HAK_KEY_ENTRY_BYTES, "HAK key table");
  const resourceTableByteLength = checkedTableLength(entryCount, HAK_RESOURCE_ENTRY_BYTES, "HAK resource table");
  if (keyTableOffset < HAK_HEADER_BYTES || resourceTableOffset < HAK_HEADER_BYTES) {
    throw new Error("HAK index table starts inside the fixed header");
  }
  return {
    entryCount,
    keyTableOffset,
    keyTableByteLength,
    resourceTableOffset,
    resourceTableByteLength,
  };
}

function parseResref(bytes: Uint8Array) {
  const terminator = bytes.indexOf(0);
  const end = terminator < 0 ? bytes.length : terminator;
  if (end === 0 || bytes.subarray(end).some((byte) => byte !== 0)) {
    throw new Error("HAK key contains an invalid padded resref");
  }
  const value = new TextDecoder("ascii", { fatal: true }).decode(bytes.subarray(0, end));
  if (!/^[A-Za-z0-9_-]{1,16}$/.test(value)) throw new Error("HAK key contains an invalid resref");
  return value.toLocaleLowerCase();
}

export function locateHakModelResourceV1(
  keyTableBuffer: ArrayBuffer,
  resourceTableBuffer: ArrayBuffer,
  entryCount: number,
  requestedResref: string,
): HakModelResourceLocatorV1 {
  const { resourceType: _resourceType, ...locator } = locateHakResourceV1(
    keyTableBuffer,
    resourceTableBuffer,
    entryCount,
    requestedResref,
    HAK_MDL_RESOURCE_TYPE,
  );
  return locator;
}

export function locateHakResourceV1(
  keyTableBuffer: ArrayBuffer,
  resourceTableBuffer: ArrayBuffer,
  entryCount: number,
  requestedResref: string,
  requestedResourceType: number,
): HakResourceLocatorV1 {
  if (!Number.isInteger(entryCount) || entryCount < 0 || entryCount > HAK_MAX_ENTRY_COUNT) {
    throw new Error("Invalid HAK entry count");
  }
  const normalized = requestedResref.trim().toLocaleLowerCase();
  if (!/^[a-z0-9_-]{1,16}$/.test(normalized)) throw new Error("Invalid HAK model resref");
  if (!Number.isInteger(requestedResourceType) || requestedResourceType < 0 || requestedResourceType > 0xffff) {
    throw new Error("Invalid HAK resource type");
  }
  const keys = exactBytes(
    keyTableBuffer,
    checkedTableLength(entryCount, HAK_KEY_ENTRY_BYTES, "HAK key table"),
    "HAK key table",
  );
  const resources = exactBytes(
    resourceTableBuffer,
    checkedTableLength(entryCount, HAK_RESOURCE_ENTRY_BYTES, "HAK resource table"),
    "HAK resource table",
  );
  const keyView = new DataView(keys.buffer, keys.byteOffset, keys.byteLength);
  for (let index = 0; index < entryCount; index += 1) {
    const offset = index * HAK_KEY_ENTRY_BYTES;
    const resourceType = keyView.getUint16(offset + 20, true);
    if (resourceType !== requestedResourceType) continue;
    const resref = parseResref(keys.subarray(offset, offset + 16));
    if (resref !== normalized) continue;
    const resourceId = keyView.getUint32(offset + 16, true);
    if (resourceId >= entryCount) throw new Error(`HAK model ${resref} has an invalid resource id`);
    const resourceOffset = resourceId * HAK_RESOURCE_ENTRY_BYTES;
    const resourceView = new DataView(resources.buffer, resources.byteOffset + resourceOffset, HAK_RESOURCE_ENTRY_BYTES);
    const payloadOffset = resourceView.getUint32(0, true);
    const payloadSize = resourceView.getUint32(4, true);
    if (payloadSize === 0) throw new Error(`HAK model ${resref} has an empty payload`);
    return { resref, resourceId, resourceType, payloadOffset, payloadSize };
  }
  throw new Error(`HAK resource ${normalized} type ${requestedResourceType} is absent from the archive`);
}
