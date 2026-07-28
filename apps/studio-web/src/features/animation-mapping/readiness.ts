import { projectAnimationCatalogRowsV1 } from "./catalog";
import { detectFallbackCyclesV1 } from "./fallbacks";
import {
  validateCustomAnimationNameV1,
  validateCustomAnimationOutputNamesV1,
  validateCustomAnimationPhasesV1,
} from "./customAnimations";
import type {
  AnimationCatalogRowV1,
  AnimationMappingDiagnosticV1,
  CreatureAnimationAuthoringV1,
  CreatureAnimationInspectionV1,
  CreatureAnimationMappingStatusV1,
} from "./types";

export interface AnimationMappingStatusCountsV1 {
  mapped: number;
  review: number;
  blockers: number;
  custom: number;
}

export type AnimationDiagnosticGroupV1 =
  | "SOURCE"
  | "RIG"
  | "BASE_COVERAGE"
  | "FALLBACK"
  | "CUSTOM"
  | "READBACK";

export function validateCreatureAnimationAuthoringV1(
  authoring: CreatureAnimationAuthoringV1,
  inspection: CreatureAnimationInspectionV1,
): AnimationMappingDiagnosticV1[] {
  const rows = projectAnimationCatalogRowsV1(authoring, inspection);
  const diagnostics: AnimationMappingDiagnosticV1[] = rows.flatMap(
    (row) => row.diagnosticCodes.map((code) => ({
    schemaVersion: 1 as const,
    code,
    path: row.kind === "BASE"
      ? `baseSlots.${row.slot}`
      : `customAnimations.${row.id.replace("custom:", "")}`,
    level: row.status === "NEEDS_REVIEW" ? "WARNING" as const : "BLOCKING" as const,
    message: diagnosticMessage(code, row),
    action: diagnosticAction(code),
    })),
  );

  const assigned = new Set<string>();
  authoring.assignments.forEach((assignment, index) => {
    if (assigned.has(assignment.targetSlot)) {
      diagnostics.push({
        schemaVersion: 1,
        code: "M2A-ANIMATION-ASSIGNMENT-DUPLICATE",
        path: `assignments[${index}].targetSlot`,
        level: "BLOCKING",
        message: `Base slot ${assignment.targetSlot} has more than one assignment.`,
        action: "Keep one source assignment for this base slot.",
      });
    }
    assigned.add(assignment.targetSlot);
    if (
      assignment.sourceKind === "INHERITED_SUPERMODEL"
      || assignment.provenance.provider === "COMPATIBLE_SUPERMODEL"
    ) {
      diagnostics.push(looksLikeFilesystemPath(assignment.provenance.assetId)
        ? {
            schemaVersion: 1,
            code: "M2A-ANIMATION-SUPERMODEL-PATH-FORBIDDEN",
            path: `assignments[${index}].provenance.assetId`,
            level: "BLOCKING",
            message: "A supermodel source must be a portable provider identity, not a filesystem path.",
            action: "Select a compatible provider from the application catalog.",
          }
        : {
            schemaVersion: 1,
            code: "M2A-ANIMATION-SUPERMODEL-PROVIDER-UNAVAILABLE",
            path: `assignments[${index}].provenance.assetId`,
            level: "BLOCKING",
            message: "No compatible packaged supermodel provider is available for this build lane.",
            action: "Use a local or generated source until a versioned provider is installed.",
          });
    }
  });
  diagnostics.push(...detectFallbackCyclesV1(authoring.fallbacks));
  authoring.customAnimations.forEach((custom, index) => {
    diagnostics.push(
      ...validateCustomAnimationNameV1(
        custom.name,
        authoring.customAnimations
          .filter((_other, otherIndex) => otherIndex !== index)
          .map(({ name }) => name),
      ),
      ...validateCustomAnimationPhasesV1(custom),
    );
  });
  diagnostics.push(
    ...validateCustomAnimationOutputNamesV1(authoring.customAnimations),
  );
  return deduplicateDiagnostics(diagnostics);
}

export function getCreatureAnimationMappingStatusV1(
  diagnostics: readonly AnimationMappingDiagnosticV1[],
): CreatureAnimationMappingStatusV1 {
  if (diagnostics.some(({ level }) => level === "BLOCKING")) return "BLOCKED";
  if (diagnostics.some(({ level }) => level === "WARNING")) return "NEEDS_REVIEW";
  return "READY";
}

export function countAnimationMappingStatusesV1(
  rows: readonly AnimationCatalogRowV1[],
): AnimationMappingStatusCountsV1 {
  const base = rows.filter(({ kind }) => kind === "BASE");
  return {
    mapped: base.filter(({ status }) => status === "MAPPED").length,
    review: base.filter(({ status }) => status === "NEEDS_REVIEW").length,
    blockers: base.filter(({ status }) => status === "BLOCKED").length,
    custom: rows.filter(({ kind }) => kind === "CUSTOM").length,
  };
}

export function canBuildCreatureAnimationPackageV1(
  status: CreatureAnimationMappingStatusV1,
): boolean {
  return status === "READY";
}

export function groupAnimationDiagnosticsV1(
  diagnostics: readonly AnimationMappingDiagnosticV1[],
): Record<AnimationDiagnosticGroupV1, AnimationMappingDiagnosticV1[]> {
  const groups: Record<AnimationDiagnosticGroupV1, AnimationMappingDiagnosticV1[]> = {
    SOURCE: [],
    RIG: [],
    BASE_COVERAGE: [],
    FALLBACK: [],
    CUSTOM: [],
    READBACK: [],
  };
  diagnostics.forEach((diagnostic) => {
    const evidence = `${diagnostic.code} ${diagnostic.path}`;
    const group: AnimationDiagnosticGroupV1 = /READBACK/.test(evidence)
      ? "READBACK"
      : /RIG/.test(evidence)
        ? "RIG"
        : /FALLBACK/.test(evidence)
          ? "FALLBACK"
          : /CUSTOM/.test(evidence)
            ? "CUSTOM"
            : /SOURCE-CLIP|SUPERMODEL/.test(evidence)
              ? "SOURCE"
              : "BASE_COVERAGE";
    groups[group].push(diagnostic);
  });
  return groups;
}

export function focusFirstBlockingAnimationIssueV1(
  diagnostics: readonly AnimationMappingDiagnosticV1[],
  root: ParentNode = document,
): boolean {
  const first = diagnostics.find(({ level }) => level === "BLOCKING");
  if (!first) return false;
  const candidates = root.querySelectorAll<HTMLElement>("[data-animation-diagnostic-code]");
  const target = Array.from(candidates).find(
    (element) => element.dataset.animationDiagnosticCode === first.code,
  );
  target?.focus();
  return Boolean(target);
}

export function formatAnimationDiagnosticV1(
  diagnostic: AnimationMappingDiagnosticV1,
): string {
  return `${diagnostic.message} ${diagnostic.action}`.trim();
}

function diagnosticMessage(code: string, row: AnimationCatalogRowV1): string {
  switch (code) {
    case "M2A-ANIMATION-SOURCE-UNASSIGNED":
      return `${row.label} does not have an animation source.`;
    case "M2A-ANIMATION-FALLBACK-REVIEW-REQUIRED":
      return `${row.label} uses a fallback that still needs review.`;
    case "M2A-ANIMATION-SOURCE-CLIP-MISSING":
      return `${row.label} references a source clip that is not in the current GLB.`;
    default:
      return `${row.label} has animation mapping issue ${code}.`;
  }
}

function diagnosticAction(code: string): string {
  if (code.includes("FALLBACK")) return "Review the fallback source, target and reason.";
  if (code.includes("CUSTOM")) return "Correct the custom animation definition.";
  return "Assign a compatible source animation.";
}

function looksLikeFilesystemPath(value: string): boolean {
  return value.includes("/")
    || value.includes("\\")
    || value.charAt(1) === ":"
    || value.startsWith(".");
}

function deduplicateDiagnostics(
  diagnostics: readonly AnimationMappingDiagnosticV1[],
): AnimationMappingDiagnosticV1[] {
  const seen = new Set<string>();
  return diagnostics.filter((diagnostic) => {
    const identity = `${diagnostic.code}\0${diagnostic.path}`;
    if (seen.has(identity)) return false;
    seen.add(identity);
    return true;
  });
}
