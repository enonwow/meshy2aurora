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

export interface ReadbackVec2 { x: number; y: number }
export interface ReadbackVec3 { x: number; y: number; z: number }

export interface ReadbackController {
  controllerName?: string;
  values: number[][];
}

export interface ReadbackMesh {
  vertices: ReadbackVec3[];
  normals: ReadbackVec3[];
  uv0: ReadbackVec2[];
  rawIndices: number[][];
  faces: Array<{ vertexIndices: [number, number, number] }>;
}

export interface ReadbackNode {
  offset: number;
  number: number;
  name: string;
  controllers: ReadbackController[];
  mesh?: ReadbackMesh;
  children: ReadbackNode[];
}

export interface ReadbackDiagnostic {
  schemaVersion: number;
  code: string;
  severity: string;
  offset: number;
  context: string;
}

export interface BinaryMdlInspectionReport {
  schemaVersion: number;
  format: string;
  nodeTree: { roots: ReadbackNode[] };
  diagnostics: ReadbackDiagnostic[];
}

export interface StudioDiagnostic {
  id: string;
  code: string;
  severity: string;
  message: string;
  path: string;
  target?: ModelPartRef;
}

function flattenNodes(roots: ReadbackNode[]): ReadbackNode[] {
  const result: ReadbackNode[] = [];
  const visit = (node: ReadbackNode) => {
    result.push(node);
    node.children.forEach(visit);
  };
  roots.forEach(visit);
  return result;
}

export function mapReadbackDiagnostics(report: BinaryMdlInspectionReport): StudioDiagnostic[] {
  const nodes = flattenNodes(report.nodeTree.roots);
  return report.diagnostics.map((diagnostic, index) => {
    const node = nodes.find((candidate) => candidate.offset === diagnostic.offset);
    return {
      id: `${diagnostic.code}:${diagnostic.offset}:${index}`,
      code: diagnostic.code,
      severity: diagnostic.severity,
      message: diagnostic.context,
      path: `byteOffset:${diagnostic.offset}`,
      target: node ? {
        kind: "READBACK_NODE",
        id: node.number,
        label: node.name || `node ${node.number}`,
      } : undefined,
    };
  });
}
