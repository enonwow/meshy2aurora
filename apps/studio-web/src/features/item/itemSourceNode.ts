export interface ItemPreviewSourceNodeContract {
  readonly rootNames: readonly string[];
  readonly selectedName: string | null;
}

interface GlbDocument {
  readonly scene?: number;
  readonly scenes?: readonly { readonly nodes?: readonly number[] }[];
  readonly nodes?: readonly { readonly name?: string }[];
}

function parseGlbDocument(bytes: ArrayBuffer): GlbDocument {
  if (bytes.byteLength < 20) throw new Error("GLB is shorter than its JSON chunk header");
  const view = new DataView(bytes);
  if (
    view.getUint32(0, true) !== 0x4654_6c67
    || view.getUint32(4, true) !== 2
    || view.getUint32(8, true) !== bytes.byteLength
    || view.getUint32(16, true) !== 0x4e4f_534a
  ) {
    throw new Error("sourceNode preview requires one exact GLB 2.0 document");
  }
  const jsonLength = view.getUint32(12, true);
  if (20 + jsonLength > bytes.byteLength) throw new Error("GLB JSON chunk is truncated");
  const text = new TextDecoder("utf-8", { fatal: true })
    .decode(new Uint8Array(bytes, 20, jsonLength))
    .replace(/[\u0000\u0020]+$/u, "");
  return JSON.parse(text) as GlbDocument;
}

export function inspectItemPreviewSourceNode(
  bytes: ArrayBuffer,
  requestedValue: string,
): ItemPreviewSourceNodeContract {
  const document = parseGlbDocument(bytes);
  const nodes = document.nodes ?? [];
  const defaultScene = document.scene === undefined
    ? undefined
    : document.scenes?.[document.scene];
  const rootIndices = defaultScene?.nodes ?? [];
  const rootNames = rootIndices.flatMap((nodeIndex) => {
    const name = nodes[nodeIndex]?.name;
    return typeof name === "string" && name.length > 0 ? [name] : [];
  });
  const requested = requestedValue.trim();
  if (!requested) return { rootNames, selectedName: null };
  const matches = nodes.flatMap((node, index) => node.name === requested ? [index] : []);
  if (matches.length !== 1) {
    throw new Error(
      `sourceNode ${JSON.stringify(requested)} matched ${matches.length} nodes in the GLB document`,
    );
  }
  if (!defaultScene) {
    throw new Error("sourceNode selection requires an explicit default GLB scene");
  }
  if (!rootIndices.includes(matches[0])) {
    throw new Error(
      `sourceNode ${JSON.stringify(requested)} is unique but is not a root of the default scene`,
    );
  }
  return { rootNames, selectedName: requested };
}
