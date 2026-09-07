import {
  effectiveCatalogModelV1,
  loadSupermodelModelBytesV1,
  type SupermodelCatalogSessionV1,
} from "./filesystem";
import type {
  AppliedSupermodelExactChainResourceV2,
  ReferenceSupermodelFormatV2,
} from "./appliedPreview";

export interface ReferenceSupermodelChainBlobDescriptorV2 {
  readonly resref: string;
  readonly supermodelResref: string;
  readonly format: ReferenceSupermodelFormatV2;
  readonly sha256: string;
  readonly byteOffset: number;
  readonly byteLength: number;
}

export interface ExactReferenceSupermodelChainV2 {
  readonly selectedSupermodelResref: string;
  readonly blob: ArrayBuffer;
  readonly descriptorsJson: string;
  readonly descriptors: readonly ReferenceSupermodelChainBlobDescriptorV2[];
}

async function sha256Hex(bytes: ArrayBuffer) {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)].map((value) => value.toString(16).padStart(2, "0")).join("");
}

export async function loadExactReferenceSupermodelChainV2(
  session: SupermodelCatalogSessionV1,
  selectedSupermodelResref: string,
  expected?: readonly AppliedSupermodelExactChainResourceV2[],
): Promise<ExactReferenceSupermodelChainV2> {
  const selected = session.catalog.entries.find(
    (entry) => entry.resref.toLocaleLowerCase() === selectedSupermodelResref.toLocaleLowerCase(),
  );
  if (!selected || selected.status !== "RESOLVED" || selected.chain.length === 0) {
    throw new Error("BLOCKED_EXACT_CHAIN_UNVERIFIED: selected supermodel chain is not fully resolved");
  }
  if (expected && expected.length !== selected.chain.length) {
    throw new Error("BLOCKED_EXACT_CHAIN_UNVERIFIED: catalog chain length changed after analysis");
  }

  const payloads: ArrayBuffer[] = [];
  const descriptors: ReferenceSupermodelChainBlobDescriptorV2[] = [];
  let byteOffset = 0;
  for (const [index, chainResref] of selected.chain.entries()) {
    const model = effectiveCatalogModelV1(session, chainResref);
    if (!model) {
      throw new Error(`BLOCKED_EXACT_CHAIN_UNVERIFIED: missing exact MDL ${chainResref}`);
    }
    const payload = await loadSupermodelModelBytesV1(session, model);
    const digest = await sha256Hex(payload);
    const descriptor = {
      resref: model.resref.toLocaleLowerCase(),
      supermodelResref: model.header.supermodelName,
      format: model.header.format,
      sha256: digest,
      byteOffset,
      byteLength: payload.byteLength,
    } satisfies ReferenceSupermodelChainBlobDescriptorV2;
    const expectedResource = expected?.[index];
    if (expectedResource && (
      expectedResource.resref.toLocaleLowerCase() !== descriptor.resref
      || expectedResource.supermodelResref.toLocaleLowerCase() !== descriptor.supermodelResref.toLocaleLowerCase()
      || expectedResource.format !== descriptor.format
      || expectedResource.sha256 !== descriptor.sha256
      || expectedResource.byteLength !== descriptor.byteLength
    )) {
      throw new Error(`M2A-REFERENCE-SUPERMODEL-CHAIN-IDENTITY-MISMATCH: ${chainResref} changed after analysis`);
    }
    const expectedParent = selected.chain[index + 1] ?? "NULL";
    if (descriptor.supermodelResref.toLocaleLowerCase() !== expectedParent.toLocaleLowerCase()) {
      throw new Error(`BLOCKED_EXACT_CHAIN_UNVERIFIED: ${chainResref} parent is ${descriptor.supermodelResref}, expected ${expectedParent}`);
    }
    payloads.push(payload);
    descriptors.push(descriptor);
    byteOffset += payload.byteLength;
  }

  const blob = new Uint8Array(byteOffset);
  let cursor = 0;
  for (const payload of payloads) {
    blob.set(new Uint8Array(payload), cursor);
    cursor += payload.byteLength;
  }
  return {
    selectedSupermodelResref: selected.resref.toLocaleLowerCase(),
    blob: blob.buffer,
    descriptorsJson: JSON.stringify(descriptors),
    descriptors,
  };
}
