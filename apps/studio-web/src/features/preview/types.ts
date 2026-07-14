export interface SourcePreviewInput {
  provenance: "SOURCE";
  file: File;
  sourceSha256: string;
}

export interface ModelPartRef {
  kind: "SOURCE_NODE" | "AURORA_SEGMENT" | "READBACK_NODE";
  id: number | string;
  label: string;
}
