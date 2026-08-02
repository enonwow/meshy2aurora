export type PlaceableTextureBindingMode = "SOURCE" | "OVERRIDE";
export type PlaceableTextureAlphaPolicy = "OPAQUE_ONLY";
export type PlaceableTexturePreviewMode = "SOURCE" | "EDITED";

export interface PlaceableMaterialTextureInspection {
  readonly materialSlot: number;
  readonly sourceMaterialId: number;
  readonly sourceMaterialName: string | null;
  readonly primitiveIds: readonly number[];
  readonly sourceTextureId: number;
  readonly sourceImageId: number;
  readonly sourceImageSha256: string;
  readonly sourceMimeType: string;
  readonly sourceImageByteLength: number;
  readonly hasUv0: boolean;
  readonly baseColorFactor: readonly [number, number, number, number];
  readonly alphaMode: string;
  readonly alphaCutoff: number | null;
  readonly normalTexturePresent: boolean;
  readonly metallicRoughnessTexturePresent: boolean;
  readonly emissiveTexturePresent: boolean;
}

export interface PlaceableTextureBindingAuthoring {
  readonly materialSlot: number;
  readonly sourceMaterialId: number;
  readonly sourceMaterialName: string | null;
  readonly sourceImageSha256: string;
  readonly mode: PlaceableTextureBindingMode;
  readonly overrideAssetId: string | null;
  readonly overrideSha256: string | null;
  readonly overrideMimeType: string | null;
  readonly overrideByteLength: number | null;
  readonly alphaPolicy: PlaceableTextureAlphaPolicy;
}

export interface PlaceableTextureAuthoringDocument {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly bindings: readonly PlaceableTextureBindingAuthoring[];
}

export interface PlaceableTextureAuthoringBootstrap {
  readonly schemaVersion: 1;
  readonly inspection: {
    readonly schemaVersion: 1;
    readonly sourceSha256: string;
    readonly materials: readonly PlaceableMaterialTextureInspection[];
  };
  readonly document: PlaceableTextureAuthoringDocument;
}

export interface ResolvedPlaceableTextureBinding {
  readonly materialSlot: number;
  readonly sourceMaterialId: number;
  readonly sourceImageSha256: string;
  readonly mode: PlaceableTextureBindingMode;
  readonly inputSha256: string;
  readonly outputResref: string;
  readonly outputSha256: string;
  readonly sourceAlphaMode: string;
  readonly ignoredSourcePbrMaps: readonly string[];
}

export interface ResolvedPlaceableTextureResource {
  readonly resref: string;
  readonly resourceType: number;
  readonly byteLength: number;
  readonly sha256: string;
  readonly materialSlots: readonly number[];
}

export interface ResolvedPlaceableTextures {
  readonly schemaVersion: 1;
  readonly sourceSha256: string;
  readonly authoringSha256: string;
  readonly alphaPolicy: PlaceableTextureAlphaPolicy;
  readonly bindings: readonly ResolvedPlaceableTextureBinding[];
  readonly resources: readonly ResolvedPlaceableTextureResource[];
}

export interface PlaceableTextureEditorSnapshot {
  readonly document: PlaceableTextureAuthoringDocument;
  readonly files: ReadonlyMap<string, File>;
  readonly preview: PlaceableTexturePreviewMode;
  readonly selectedMaterialSlot: number | null;
}

export interface PreparedPlaceableTexturePayloads {
  readonly authoringJson: string;
  readonly payloadBlob: ArrayBuffer;
  readonly descriptorsJson: string;
}

const SHA256_PATTERN = /^[0-9a-f]{64}$/;

function validBootstrap(value: PlaceableTextureAuthoringBootstrap) {
  return value.schemaVersion === 1
    && value.inspection?.schemaVersion === 1
    && value.document?.schemaVersion === 1
    && value.inspection.sourceSha256 === value.document.sourceSha256
    && SHA256_PATTERN.test(value.document.sourceSha256)
    && Array.isArray(value.inspection.materials)
    && Array.isArray(value.document.bindings)
    && value.inspection.materials.length === value.document.bindings.length;
}

export function parsePlaceableTextureAuthoringBootstrap(
  json: string,
): PlaceableTextureAuthoringBootstrap {
  const value = JSON.parse(json) as PlaceableTextureAuthoringBootstrap;
  if (!validBootstrap(value)) throw new Error("PLACEABLE-TEXTURE-BOOTSTRAP-INVALID");
  return value;
}

export function parseResolvedPlaceableTextures(json: string): ResolvedPlaceableTextures {
  const value = JSON.parse(json) as ResolvedPlaceableTextures;
  if (
    value.schemaVersion !== 1
    || value.alphaPolicy !== "OPAQUE_ONLY"
    || !SHA256_PATTERN.test(value.sourceSha256)
    || !SHA256_PATTERN.test(value.authoringSha256)
    || !Array.isArray(value.bindings)
    || !Array.isArray(value.resources)
  ) throw new Error("PLACEABLE-TEXTURE-RESOLUTION-INVALID");
  return value;
}

export async function sha256FileV1(file: File): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", await file.arrayBuffer());
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function canonicalMimeType(file: File): "image/png" | "image/jpeg" {
  const type = file.type.toLowerCase();
  const name = file.name.toLowerCase();
  if (type === "image/png" || name.endsWith(".png")) return "image/png";
  if (type === "image/jpeg" || type === "image/jpg" || /\.jpe?g$/.test(name)) {
    return "image/jpeg";
  }
  throw new Error("PLACEABLE-TEXTURE-MIME-UNSUPPORTED: Select a PNG or JPEG image.");
}

export async function createTextureOverrideBindingV1(
  binding: PlaceableTextureBindingAuthoring,
  file: File,
): Promise<{ readonly binding: PlaceableTextureBindingAuthoring; readonly assetId: string }> {
  const mimeType = canonicalMimeType(file);
  const sha256 = await sha256FileV1(file);
  const assetId = `texture-${sha256.slice(0, 24)}`;
  return {
    assetId,
    binding: {
      ...binding,
      mode: "OVERRIDE",
      overrideAssetId: assetId,
      overrideSha256: sha256,
      overrideMimeType: mimeType,
      overrideByteLength: file.size,
      alphaPolicy: "OPAQUE_ONLY",
    },
  };
}

export function sourceTextureBindingV1(
  binding: PlaceableTextureBindingAuthoring,
): PlaceableTextureBindingAuthoring {
  return {
    ...binding,
    mode: "SOURCE",
    overrideAssetId: null,
    overrideSha256: null,
    overrideMimeType: null,
    overrideByteLength: null,
    alphaPolicy: "OPAQUE_ONLY",
  };
}

export async function preparePlaceableTexturePayloadsV1(
  snapshot: PlaceableTextureEditorSnapshot,
): Promise<PreparedPlaceableTexturePayloads> {
  const descriptors: {
    schemaVersion: 1;
    assetId: string;
    sha256: string;
    mimeType: string;
    byteOffset: number;
    byteLength: number;
  }[] = [];
  const chunks: Uint8Array[] = [];
  let byteOffset = 0;
  const overrides = [...snapshot.document.bindings]
    .filter((binding) => binding.mode === "OVERRIDE")
    .sort((left, right) => (left.overrideAssetId ?? "").localeCompare(right.overrideAssetId ?? ""));
  const seen = new Set<string>();
  for (const binding of overrides) {
    const assetId = binding.overrideAssetId;
    if (!assetId || seen.has(assetId)) continue;
    seen.add(assetId);
    const file = snapshot.files.get(assetId);
    if (!file) throw new Error(`PLACEABLE-TEXTURE-PAYLOAD-MISSING: ${assetId}`);
    const sha256 = await sha256FileV1(file);
    if (sha256 !== binding.overrideSha256 || file.size !== binding.overrideByteLength) {
      throw new Error(`PLACEABLE-TEXTURE-PAYLOAD-STALE: ${assetId}`);
    }
    const bytes = new Uint8Array(await file.arrayBuffer());
    descriptors.push({
      schemaVersion: 1,
      assetId,
      sha256,
      mimeType: canonicalMimeType(file),
      byteOffset,
      byteLength: bytes.byteLength,
    });
    chunks.push(bytes);
    byteOffset += bytes.byteLength;
  }
  const payload = new Uint8Array(byteOffset);
  let cursor = 0;
  for (const chunk of chunks) {
    payload.set(chunk, cursor);
    cursor += chunk.byteLength;
  }
  return {
    authoringJson: JSON.stringify(snapshot.document),
    payloadBlob: payload.buffer,
    descriptorsJson: JSON.stringify(descriptors),
  };
}
