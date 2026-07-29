import type { WorkerArtifact } from "../../worker/types";
import type { ProjectBuildIdentityV1 } from "../project";

export type DownloadManifestTargetV1 = "CREATURE" | "PLACEABLE" | "TILE";

export interface DownloadManifestInputIdentityV1 {
  readonly role: string;
  readonly fileName: string;
  readonly byteLength: number | null;
  readonly sha256: string;
}

export interface DownloadManifestV1 {
  readonly schemaVersion: 1;
  readonly projectIdentity: ProjectBuildIdentityV1;
  readonly target: DownloadManifestTargetV1;
  readonly buildStatus:
    | "OFFLINE_READBACK_RECONCILED"
    | "OFFLINE_READBACK_INCOMPLETE";
  readonly inputs: readonly DownloadManifestInputIdentityV1[];
  readonly outputs: readonly {
    artifactId: string;
    kind: WorkerArtifact["kind"];
    fileName: string;
    mediaType: string;
    byteLength: number;
    sha256: string;
    provenance: "M2A_WASM_WORKER";
  }[];
  readonly ownerProof: {
    status: "PENDING_OWNER";
    modelVisibility: "not_tested";
    proofCompleteness: "missing";
  };
  readonly zipPolicy: "NOT_INCLUDED_IN_E4_CANONICAL_DELIVERY";
}

export interface DownloadReadinessV1 {
  readonly allowed: boolean;
  readonly status: "READY" | "LOCKED";
  readonly reason: string;
}

const SHA256 = /^[a-f0-9]{64}$/;

function exactProjectIdentity(
  left: ProjectBuildIdentityV1,
  right: ProjectBuildIdentityV1,
) {
  return left.schemaVersion === right.schemaVersion
    && left.projectId === right.projectId
    && left.projectName === right.projectName
    && left.projectRevision === right.projectRevision;
}

export function projectDownloadReadinessV1(input: {
  readonly currentProjectIdentity: ProjectBuildIdentityV1;
  readonly builtProjectIdentity: ProjectBuildIdentityV1;
  readonly offlineReconciled: boolean;
}): DownloadReadinessV1 {
  if (!exactProjectIdentity(
    input.currentProjectIdentity,
    input.builtProjectIdentity,
  )) {
    return {
      allowed: false,
      status: "LOCKED",
      reason: "Download is locked because the result belongs to a different project revision.",
    };
  }
  if (!input.offlineReconciled) {
    return {
      allowed: false,
      status: "LOCKED",
      reason: "Download is locked until canonical binary readback is fully reconciled.",
    };
  }
  return {
    allowed: true,
    status: "READY",
    reason: "Exact current project revision and canonical offline readback are reconciled.",
  };
}

export function createDownloadManifestV1(input: {
  readonly projectIdentity: ProjectBuildIdentityV1;
  readonly target: DownloadManifestTargetV1;
  readonly inputs: readonly DownloadManifestInputIdentityV1[];
  readonly artifacts: readonly WorkerArtifact[];
  readonly offlineReconciled: boolean;
}): DownloadManifestV1 {
  validateProjectIdentity(input.projectIdentity);
  const inputs = [...input.inputs]
    .sort((left, right) => (
      left.role.localeCompare(right.role) || left.fileName.localeCompare(right.fileName)
    ));
  if (
    new Set(inputs.map(({ role }) => role)).size !== inputs.length
    || inputs.some((item) => (
      item.role.trim().length === 0
      || item.fileName.trim().length === 0
      || !SHA256.test(item.sha256)
      || (
        item.byteLength !== null
        && (!Number.isSafeInteger(item.byteLength) || item.byteLength < 0)
      )
    ))
  ) {
    throw new Error("Download manifest input identity is invalid");
  }
  const outputs = [...input.artifacts]
    .sort((left, right) => left.artifactId.localeCompare(right.artifactId))
    .map((artifact) => {
      if (
        artifact.provenance !== "M2A_WASM_WORKER"
        || artifact.artifactId.trim().length === 0
        || artifact.fileName.trim().length === 0
        || artifact.byteLength !== artifact.bytes.byteLength
        || !SHA256.test(artifact.sha256)
      ) {
        throw new Error("Download manifest output identity is invalid");
      }
      return {
        artifactId: artifact.artifactId,
        kind: artifact.kind,
        fileName: artifact.fileName,
        mediaType: artifact.mediaType,
        byteLength: artifact.byteLength,
        sha256: artifact.sha256,
        provenance: "M2A_WASM_WORKER" as const,
      };
    });
  if (
    outputs.length === 0
    || new Set(outputs.map(({ artifactId }) => artifactId)).size !== outputs.length
  ) {
    throw new Error("Download manifest output inventory is invalid");
  }
  return {
    schemaVersion: 1,
    projectIdentity: { ...input.projectIdentity },
    target: input.target,
    buildStatus: input.offlineReconciled
      ? "OFFLINE_READBACK_RECONCILED"
      : "OFFLINE_READBACK_INCOMPLETE",
    inputs,
    outputs,
    ownerProof: {
      status: "PENDING_OWNER",
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
    },
    zipPolicy: "NOT_INCLUDED_IN_E4_CANONICAL_DELIVERY",
  };
}

export function serializeDownloadManifestV1(manifest: DownloadManifestV1) {
  return `${JSON.stringify(manifest, null, 2)}\n`;
}

export function downloadManifestFileNameV1(
  artifacts: readonly WorkerArtifact[],
) {
  const modules = artifacts.filter(({ kind }) => kind === "MODULE");
  if (modules.length !== 1 || !modules[0]!.fileName.toLowerCase().endsWith(".mod")) {
    throw new Error("Download manifest requires exactly one generated module artifact");
  }
  return `${modules[0]!.fileName.slice(0, -4)}-download-manifest.json`;
}

export function validateDownloadManifestInventoryV1(
  manifest: DownloadManifestV1,
  artifacts: readonly WorkerArtifact[],
) {
  if (
    manifest.outputs.length !== artifacts.length
    || manifest.outputs.some((output) => {
      const matches = artifacts.filter(({ artifactId }) => (
        artifactId === output.artifactId
      ));
      const artifact = matches[0];
      return matches.length !== 1
        || !artifact
        || artifact.kind !== output.kind
        || artifact.fileName !== output.fileName
        || artifact.mediaType !== output.mediaType
        || artifact.byteLength !== output.byteLength
        || artifact.sha256 !== output.sha256
        || artifact.provenance !== output.provenance;
    })
  ) {
    throw new Error("Download manifest does not match the exact artifact inventory");
  }
}

function validateProjectIdentity(identity: ProjectBuildIdentityV1) {
  if (
    identity.schemaVersion !== 1
    || identity.projectId.trim().length === 0
    || identity.projectName.trim().length === 0
    || !Number.isSafeInteger(identity.projectRevision)
    || identity.projectRevision < 1
  ) {
    throw new Error("Download manifest project identity is invalid");
  }
}
