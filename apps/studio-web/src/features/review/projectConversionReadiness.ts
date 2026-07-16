import type { BinaryMdlInspectionReport, ReadbackDiagnostic } from "../preview/types";
import type {
  CanonicalConversionDiagnostic,
  CanonicalConversionGate,
  CanonicalResultSnapshot,
} from "../results/projectCanonicalResult";

export type ReadinessStatus = "PASS" | "WARNING" | "FAIL" | "NOT_CHECKED" | "OPEN";
export type ReadinessCategoryId =
  | "GEOMETRY"
  | "MATERIALS_TEXTURES"
  | "RIG"
  | "ANIMATIONS"
  | "BINARY_READBACK"
  | "PACKAGE_ASSEMBLY"
  | "RUNTIME_PROOF";

export interface ReadinessItem {
  id: ReadinessCategoryId;
  label: string;
  status: ReadinessStatus;
  statusLabel: string;
  checkCount: number;
  detail: string;
}

export interface ReadinessValidationEntry {
  id: string;
  category?: ReadinessCategoryId;
  source: "CONVERSION_GATE" | "CONVERSION_DIAGNOSTIC" | "BINARY_READBACK";
  code: string;
  severity: string;
  path: string;
  expected?: string;
  actual?: string;
  message: string;
}

export interface ConversionReadinessProjection {
  items: ReadinessItem[];
  validation: ReadinessValidationEntry[];
  conversionEligible: boolean;
}

const categoryLabels: Record<Exclude<ReadinessCategoryId, "BINARY_READBACK" | "PACKAGE_ASSEMBLY" | "RUNTIME_PROOF">, string> = {
  GEOMETRY: "Geometry",
  MATERIALS_TEXTURES: "Materials & Textures",
  RIG: "Rig",
  ANIMATIONS: "Animations",
};

type ConversionCategoryId = keyof typeof categoryLabels;

function normalizedSeverity(value: string) {
  return value.trim().toUpperCase();
}

function statusFromGates(gates: readonly CanonicalConversionGate[]): ReadinessStatus {
  if (gates.length === 0) return "NOT_CHECKED";
  const severities = gates.map(({ severity }) => normalizedSeverity(severity));
  if (severities.some((severity) => ["BLOCKING", "ERROR", "FATAL", "FAIL", "FAILED"].includes(severity))) return "FAIL";
  if (severities.some((severity) => ["WARNING", "WARN"].includes(severity))) return "WARNING";
  if (severities.every((severity) => ["PASS", "PASSED", "OK", "SUCCESS"].includes(severity))) return "PASS";
  return "NOT_CHECKED";
}

export function categoryForConversionEvidence(code: string, path: string): ConversionCategoryId | undefined {
  const evidence = `${code} ${path}`.toUpperCase();
  if (/ANIMATION|CLIP|KEYFRAME/.test(evidence)) return "ANIMATIONS";
  if (/MATERIAL|TEXTURE|IMAGE|SAMPLER|\bUV\b|UV0/.test(evidence)) return "MATERIALS_TEXTURES";
  if (/\bRIG\b|WEIGHT|BONE|SKIN|JOINT|INFLUENCE/.test(evidence)) return "RIG";
  if (/GEOMETRY|PRIMITIVE|TRIANGLE|VERTEX|NORMAL|TANGENT|BOUNDS|SEGMENT|MESH|SOURCESELECTION|SOURCE\.IR\.SCENE/.test(evidence)) return "GEOMETRY";
  return undefined;
}

function validationSeverity(value: string) {
  const severity = normalizedSeverity(value);
  if (["BLOCKING", "ERROR", "FATAL", "FAIL", "FAILED"].includes(severity)) return "FAIL";
  if (["WARNING", "WARN"].includes(severity)) return "WARNING";
  if (["PASS", "PASSED", "OK", "SUCCESS"].includes(severity)) return "PASS";
  return severity || "UNKNOWN";
}

function gateValidation(gate: CanonicalConversionGate, index: number): ReadinessValidationEntry {
  return {
    id: `conversion-gate:${index}:${gate.code}:${gate.path}`,
    category: categoryForConversionEvidence(gate.code, gate.path),
    source: "CONVERSION_GATE",
    code: gate.code,
    severity: validationSeverity(gate.severity),
    path: gate.path,
    expected: gate.expected,
    actual: gate.actual,
    message: gate.message,
  };
}

function diagnosticValidation(diagnostic: CanonicalConversionDiagnostic, index: number): ReadinessValidationEntry {
  return {
    id: `conversion-diagnostic:${index}:${diagnostic.code}:${diagnostic.path}`,
    category: categoryForConversionEvidence(diagnostic.code, diagnostic.path),
    source: "CONVERSION_DIAGNOSTIC",
    code: diagnostic.code,
    severity: validationSeverity(diagnostic.severity),
    path: diagnostic.path,
    message: diagnostic.message,
  };
}

function readbackValidation(diagnostic: ReadbackDiagnostic, index: number): ReadinessValidationEntry {
  return {
    id: `readback:${index}:${diagnostic.code}:${diagnostic.offset}`,
    category: "BINARY_READBACK",
    source: "BINARY_READBACK",
    code: diagnostic.code,
    severity: validationSeverity(diagnostic.severity),
    path: `byteOffset:${diagnostic.offset}`,
    message: diagnostic.context,
  };
}

function runtimeProof(result: CanonicalResultSnapshot): ReadinessItem {
  const { engineFacingProof, uvRuntimeProof } = result.conversionEvidence.policies;
  const samePolicy = engineFacingProof === uvRuntimeProof;
  const status = engineFacingProof.startsWith("OPEN_") || uvRuntimeProof.startsWith("OPEN_") ? "OPEN" : "NOT_CHECKED";
  return {
    id: "RUNTIME_PROOF",
    label: "Runtime Proof",
    status,
    statusLabel: samePolicy ? engineFacingProof : `${engineFacingProof} / ${uvRuntimeProof}`,
    checkCount: 0,
    detail: `Engine-facing: ${engineFacingProof}; UV runtime: ${uvRuntimeProof}`,
  };
}

export function projectConversionReadiness(
  result: CanonicalResultSnapshot,
  readback: BinaryMdlInspectionReport,
): ConversionReadinessProjection {
  const conversionItems = (Object.keys(categoryLabels) as ConversionCategoryId[]).map((id): ReadinessItem => {
    const gates = result.conversionEvidence.gates.filter((gate) => categoryForConversionEvidence(gate.code, gate.path) === id);
    const status = statusFromGates(gates);
    return {
      id,
      label: categoryLabels[id],
      status,
      statusLabel: status,
      checkCount: gates.length,
      detail: gates.length === 0
        ? "No positive canonical rule evidence was emitted for this category."
        : `${gates.length} canonical conversion gate(s) reported.`,
    };
  });

  const readbackEvidence = readback.validation;
  const readbackStatus: ReadinessStatus = !readbackEvidence
    ? "NOT_CHECKED"
    : readbackEvidence.status === "ERROR"
      ? "FAIL"
      : readbackEvidence.status;
  const readbackItem: ReadinessItem = {
    id: "BINARY_READBACK",
    label: "Binary Readback",
    status: readbackStatus,
    statusLabel: readbackStatus,
    checkCount: readbackEvidence ? 1 + readbackEvidence.diagnostics.total : 0,
    detail: readbackEvidence
      ? `${readbackEvidence.structure.rootNodeCount} root node(s); ${readbackEvidence.diagnostics.total} diagnostic(s).`
      : "Canonical binary readback validation evidence is unavailable.",
  };

  const packageEvidence = result.packageAssemblyEvidence;
  const packageItem: ReadinessItem = {
    id: "PACKAGE_ASSEMBLY",
    label: "Package Assembly",
    status: packageEvidence.strictReconciled ? "PASS" : "NOT_CHECKED",
    statusLabel: packageEvidence.strictReconciled ? "PASS" : "NOT_CHECKED",
    checkCount: packageEvidence.resourceCount + packageEvidence.artifactCount,
    detail: `${packageEvidence.resourceCount} reconciled resource(s); ${packageEvidence.artifactCount} reconciled artifact(s).`,
  };

  const validation = [
    ...result.conversionEvidence.gates.map(gateValidation),
    ...result.conversionEvidence.diagnostics.map(diagnosticValidation),
    ...readback.diagnostics.map(readbackValidation),
  ];
  readbackEvidence?.structure.structuralErrors.forEach((code, index) => validation.push({
    id: `readback-structure:${index}:${code}`,
    category: "BINARY_READBACK",
    source: "BINARY_READBACK",
    code,
    severity: "FAIL",
    path: "readback.nodeTree.roots",
    message: "Canonical binary readback reported a structural error.",
  }));

  return {
    items: [...conversionItems, readbackItem, packageItem, runtimeProof(result)],
    validation,
    conversionEligible: result.conversionEvidence.conversionEligible,
  };
}
