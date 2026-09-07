import type {
  ModelTextureAuthoringDocumentV1,
  ModelTextureBindingAuthoringV1,
} from "./types";

export interface ModelTextureEditorSnapshotV1 {
  readonly document: ModelTextureAuthoringDocumentV1;
  readonly files: ReadonlyMap<string, File>;
}

export interface PreparedModelTexturePayloadsV1 {
  readonly authoringJson: string;
  readonly payloadBlob: ArrayBuffer;
  readonly descriptorsJson: string;
}

const sha256 = async (file: File) => {
  const digest = await crypto.subtle.digest("SHA-256", await file.arrayBuffer());
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
};

const mimeType = (file: File): "image/png" | "image/jpeg" => {
  const type = file.type.toLowerCase();
  const name = file.name.toLowerCase();
  if (type === "image/png" || name.endsWith(".png")) return "image/png";
  if (type === "image/jpeg" || type === "image/jpg" || /\.jpe?g$/.test(name)) {
    return "image/jpeg";
  }
  throw new Error("MODEL-TEXTURE-MIME-UNSUPPORTED: Select a PNG or JPEG image.");
};

export function sourceModelTextureBindingV1(
  binding: ModelTextureBindingAuthoringV1,
): ModelTextureBindingAuthoringV1 {
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

export async function overrideModelTextureBindingV1(
  binding: ModelTextureBindingAuthoringV1,
  file: File,
) {
  const canonicalMimeType = mimeType(file);
  const digest = await sha256(file);
  const assetId = `material-texture-${digest.slice(0, 24)}`;
  return {
    assetId,
    binding: {
      ...binding,
      mode: "OVERRIDE" as const,
      overrideAssetId: assetId,
      overrideSha256: digest,
      overrideMimeType: canonicalMimeType,
      overrideByteLength: file.size,
      alphaPolicy: "OPAQUE_ONLY" as const,
    },
  };
}

export function reuseModelTextureOverrideV1(
  target: ModelTextureBindingAuthoringV1,
  source: ModelTextureBindingAuthoringV1,
): ModelTextureBindingAuthoringV1 {
  if (source.mode !== "OVERRIDE" || !source.overrideAssetId || !source.overrideSha256
    || !source.overrideMimeType || source.overrideByteLength === null) {
    throw new Error("MODEL-TEXTURE-REUSE-SOURCE-INVALID: Select an uploaded override texture.");
  }
  return {
    ...target,
    mode: "OVERRIDE",
    overrideAssetId: source.overrideAssetId,
    overrideSha256: source.overrideSha256,
    overrideMimeType: source.overrideMimeType,
    overrideByteLength: source.overrideByteLength,
    alphaPolicy: "OPAQUE_ONLY",
  };
}

export async function prepareModelTexturePayloadsV1(
  snapshot: ModelTextureEditorSnapshotV1,
): Promise<PreparedModelTexturePayloadsV1> {
  const descriptors: Array<{
    schemaVersion: 1;
    assetId: string;
    sha256: string;
    mimeType: string;
    byteOffset: number;
    byteLength: number;
  }> = [];
  const chunks: Uint8Array[] = [];
  const seen = new Set<string>();
  let byteOffset = 0;
  const overrides = snapshot.document.bindings
    .filter((binding) => binding.mode === "OVERRIDE")
    .sort((left, right) => (left.overrideAssetId ?? "").localeCompare(right.overrideAssetId ?? ""));
  for (const binding of overrides) {
    const assetId = binding.overrideAssetId;
    if (!assetId || seen.has(assetId)) continue;
    seen.add(assetId);
    const file = snapshot.files.get(assetId);
    if (!file) throw new Error(`MODEL-TEXTURE-PAYLOAD-MISSING: ${assetId}`);
    const digest = await sha256(file);
    if (digest !== binding.overrideSha256 || file.size !== binding.overrideByteLength) {
      throw new Error(`MODEL-TEXTURE-PAYLOAD-STALE: ${assetId}`);
    }
    const bytes = new Uint8Array(await file.arrayBuffer());
    descriptors.push({
      schemaVersion: 1,
      assetId,
      sha256: digest,
      mimeType: mimeType(file),
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
