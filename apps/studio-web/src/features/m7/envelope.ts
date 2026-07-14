export interface M7SourceSelection {
  role: "SOURCE";
  relativePath: string;
  file: File;
}

export interface M7AppearanceSelection {
  role: "RIGGED_HUMANOID_APPEARANCE_2DA";
  sampleId: string;
  file: File;
}

export interface M7PayloadEnvelope {
  payloadBlob: ArrayBuffer;
  descriptorsJson: string;
}

type Descriptor =
  | {
      role: "SOURCE";
      relativePath: string;
      payloadOffset: number;
      payloadSize: number;
    }
  | {
      role: "RIGGED_HUMANOID_APPEARANCE_2DA";
      sampleId: string;
      payloadOffset: number;
      payloadSize: number;
    };

export async function buildM7PayloadEnvelope(
  sources: readonly M7SourceSelection[],
  appearances: readonly M7AppearanceSelection[],
): Promise<M7PayloadEnvelope> {
  const selections = [...sources, ...appearances];
  let byteLength = 0;
  for (const { file } of selections) {
    if (!Number.isSafeInteger(file.size) || file.size <= 0 || file.size > 0xffff_ffff) {
      throw new Error(`M7 payload file ${file.name} has an invalid u32 byte size`);
    }
    byteLength += file.size;
    if (!Number.isSafeInteger(byteLength) || byteLength > 0xffff_ffff) {
      throw new Error("M7 payload envelope exceeds u32 offsets");
    }
  }

  const payloadBlob = new ArrayBuffer(byteLength);
  const blob = new Uint8Array(payloadBlob);
  const descriptors: Descriptor[] = [];
  let offset = 0;
  for (const selection of selections) {
    const bytes = await selection.file.arrayBuffer();
    if (bytes.byteLength !== selection.file.size) {
      throw new Error(`M7 payload file ${selection.file.name} changed while being read`);
    }
    const chunk = new Uint8Array(bytes);
    blob.set(chunk, offset);
    descriptors.push(selection.role === "SOURCE"
      ? {
          role: selection.role,
          relativePath: selection.relativePath,
          payloadOffset: offset,
          payloadSize: chunk.byteLength,
        }
      : {
          role: selection.role,
          sampleId: selection.sampleId,
          payloadOffset: offset,
          payloadSize: chunk.byteLength,
        });
    offset += chunk.byteLength;
  }

  return {
    payloadBlob,
    descriptorsJson: JSON.stringify({ schemaVersion: 1, payloads: descriptors }),
  };
}
