import type {
  AnimationStudioDiagnosticV1,
  AnimationStudioDocumentV1,
  AnimationStudioReadbackStatusV1,
  AuthoredAnimationClipV1,
  AuthoredAnimationTrackPathV1,
  AuthoredAnimationTrackV1,
} from "../animation-studio/types";
import type {
  BinaryMdlInspectionReport,
  ReadbackAnimation,
  ReadbackController,
  ReadbackNode,
} from "../preview/types";
import type {
  CanonicalAnimationStudioEvidenceV1,
  CanonicalMaterializedAnimationClipV1,
} from "../results/projectCanonicalResult";

const READBACK_EPSILON = 1e-5;

export interface AnimationStudioReadbackReconciliationV1 {
  schemaVersion: 1;
  status: AnimationStudioReadbackStatusV1;
  checkedClipCount: number;
  matchedClipIds: string[];
  diagnostics: AnimationStudioDiagnosticV1[];
}

export interface AnimationStudioDownloadGateV1 {
  allowed: boolean;
  status: "BINARY_READY" | "BLOCKED";
  code:
    | "M2A-ANIMATION-READBACK-MATCH"
    | "M2A-ANIMATION-READBACK-MISMATCH";
  reason: string;
}

/**
 * Compares the authored Studio document with the canonical Rust binary-MDL
 * readback. This is deliberately independent of source GLB preview data.
 */
export function reconcileAnimationStudioReadbackV1(
  studio: AnimationStudioDocumentV1,
  readback: BinaryMdlInspectionReport,
  evidence?: CanonicalAnimationStudioEvidenceV1,
): AnimationStudioReadbackReconciliationV1 {
  const diagnostics: AnimationStudioDiagnosticV1[] = [];
  const matchedClipIds: string[] = [];

  if (studio.status !== "VALID") {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-DOCUMENT-NOT-VALID",
      "status",
      `Animation Studio document is ${studio.status}, not VALID.`,
      "Resolve Studio diagnostics and validate the document before building.",
    ));
  }
  if (!readback.validation) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-VALIDATION-MISSING",
      "readback.validation",
      "Canonical binary readback validation evidence is missing.",
      "Run canonical Rust binary-MDL readback before downloading artifacts.",
    ));
  } else if (readback.validation.status === "ERROR") {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-VALIDATION-ERROR",
      "readback.validation.status",
      "Canonical binary readback validation failed.",
      "Resolve binary readback errors and rebuild the package.",
    ));
  }

  for (const clip of studio.authoredClips) {
    const diagnosticStart = diagnostics.length;
    const clipPath = `authoredClips.${clip.id}`;
    if (clip.status !== "VALID") {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-CLIP-NOT-VALID",
        `${clipPath}.status`,
        `${clip.status} authored animation cannot be reconciled for download.`,
        "Validate the authored clip before building.",
      ));
      continue;
    }

    const evidenceClip = evidence?.authoredClips.find(({ id }) => id === clip.id);
    const outputNames = evidence
      ? [...new Set(evidenceClip?.usages.map(({ outputClipName }) => outputClipName)
        ?? [])]
      : [clip.name];
    if (evidence && !evidenceClip) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-USAGE-MISSING",
        `${clipPath}.id`,
        `V5 manifest has no authored usage evidence for stable ID ${clip.id}.`,
        "Rebuild from the exact Studio document and V2 mapping.",
      ));
      continue;
    }
    if (outputNames.length === 0) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-USAGE-MISSING",
        `${clipPath}.id`,
        `Authored animation ${clip.id} has no materialized output usage.`,
        "Assign the saved Custom animation or remove the unused authored clip.",
      ));
      continue;
    }
    for (const outputName of outputNames) {
      const candidates = readback.animations.filter(({ name }) => name === outputName);
      if (candidates.length === 0) {
        diagnostics.push(diagnostic(
          "M2A-ANIMATION-READBACK-CLIP-MISSING",
          `${clipPath}.name`,
          `Canonical readback is missing authored output ${outputName}.`,
          "Rebuild the package from the current Animation Studio document.",
        ));
        continue;
      }
      if (candidates.length > 1) {
        diagnostics.push(diagnostic(
          "M2A-ANIMATION-READBACK-CLIP-DUPLICATE",
          `${clipPath}.name`,
          `Canonical readback contains duplicate animation ${outputName}.`,
          "Rebuild with one unique output clip for each authored usage.",
        ));
        continue;
      }
      if (evidence) {
        const canonicalMatches = evidence.animationStudioReadback.clips.filter(
          (item) => item.authoredClipId === clip.id
            && item.outputClipName === outputName,
        );
        if (canonicalMatches.length !== 1) {
          diagnostics.push(diagnostic(
            canonicalMatches.length === 0
              ? "M2A-ANIMATION-READBACK-USAGE-MISSING"
              : "M2A-ANIMATION-READBACK-CLIP-DUPLICATE",
            `${clipPath}.id`,
            canonicalMatches.length === 0
              ? `Canonical core readback has no MATCH evidence for authored output ${outputName}.`
              : `Canonical core readback contains duplicate MATCH evidence for authored output ${outputName}.`,
            "Rebuild from the exact Studio document and inspect canonical materialization evidence.",
          ));
        } else {
          compareMaterializedClip(
            canonicalMatches[0]!.materializedClip,
            candidates[0]!,
            clipPath,
            diagnostics,
          );
        }
        continue;
      }
      compareClip(clip, candidates[0]!, clipPath, diagnostics);
    }
    if (diagnostics.length === diagnosticStart) matchedClipIds.push(clip.id);
  }

  return {
    schemaVersion: 1,
    status: diagnostics.length === 0 ? "MATCH" : "MISMATCH",
    checkedClipCount: studio.authoredClips.length,
    matchedClipIds,
    diagnostics,
  };
}

export function getAnimationStudioDownloadGateV1(
  reconciliation: AnimationStudioReadbackReconciliationV1,
): AnimationStudioDownloadGateV1 {
  return reconciliation.status === "MATCH" && reconciliation.diagnostics.length === 0
    ? {
        allowed: true,
        status: "BINARY_READY",
        code: "M2A-ANIMATION-READBACK-MATCH",
        reason:
          "All valid authored animations match canonical binary MDL readback. Runtime playback still requires separate owner proof.",
      }
    : {
        allowed: false,
        status: "BLOCKED",
        code: "M2A-ANIMATION-READBACK-MISMATCH",
        reason: "Animation download is blocked because canonical binary MDL readback does not match the Studio document.",
      };
}

function compareMaterializedClip(
  expected: CanonicalMaterializedAnimationClipV1,
  actual: ReadbackAnimation,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
) {
  compareScalar(
    expected.lengthSeconds,
    actual.length,
    "M2A-ANIMATION-READBACK-LENGTH-MISMATCH",
    `${path}.lengthSeconds`,
    "Materialized animation length differs from canonical core evidence.",
    diagnostics,
  );
  compareScalar(
    expected.transitionSeconds,
    actual.transition,
    "M2A-ANIMATION-READBACK-TRANSITION-MISMATCH",
    `${path}.transitionSeconds`,
    "Materialized animation transition differs from canonical core evidence.",
    diagnostics,
  );
  if (expected.name !== actual.name) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-NAME-MISMATCH",
      `${path}.name`,
      "Materialized animation name differs from canonical core evidence.",
      "Read back the exact binary emitted by the current build.",
    ));
  }
  if (expected.animationRoot !== actual.animationRoot) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-ANIMROOT-MISMATCH",
      `${path}.animationRoot`,
      "Materialized animation root differs from canonical core evidence.",
      "Read back the exact binary emitted by the current build.",
    ));
  }
  if (
    expected.events.length !== actual.events.length
    || expected.events.some((event, index) => {
      const readbackEvent = actual.events[index];
      return !readbackEvent
        || event.name !== readbackEvent.name
        || !near(event.timeSeconds, readbackEvent.time);
    })
  ) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-EVENTS-MISMATCH",
      `${path}.events`,
      "Materialized animation events differ from canonical core evidence.",
      "Read back the exact binary emitted by the current build.",
    ));
  }

  const nodes = flattenNodes(actual.nodeTree.roots);
  const expectedTrackKeys = new Set<string>();
  expected.tracks.forEach((track, index) => {
    const trackPath = `${path}.materializedTracks[${index}]`;
    if (track.path !== "TRANSLATION" && track.path !== "ROTATION") {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-TRACK-MISSING",
        `${trackPath}.path`,
        `Canonical materialized ${track.path} cannot be reconciled with the binary controller inventory.`,
        "Block Download until the binary readback exposes this controller type.",
      ));
      return;
    }
    const key = trackKey(track.targetNodeId, track.path);
    expectedTrackKeys.add(key);
    const targetNodes = nodes.filter(({ number }) => (
      number === track.targetNodeId
    ));
    const controllerName = controllerNameForPath(track.path);
    const controllers = targetNodes.length === 1
      ? targetNodes[0]!.controllers.filter((controller) => (
          controller.controllerName === controllerName
        ))
      : [];
    if (targetNodes.length !== 1 || controllers.length !== 1) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-TRACK-MISSING",
        trackPath,
        "Canonical materialized controller is missing or duplicated in binary readback.",
        "Read back the exact binary emitted by the current build.",
      ));
      return;
    }
    if (!numberArraysMatch(track.timesSeconds, controllers[0]!.times)) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-TIMES-MISMATCH",
        `${trackPath}.timesSeconds`,
        "Materialized keyframe times differ from canonical core evidence.",
        "Read back the exact binary emitted by the current build.",
      ));
    }
    if (!valueRowsMatch(track.path, track.values, controllers[0]!.values)) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-VALUES-MISMATCH",
        `${trackPath}.values`,
        "Materialized keyframe values differ from canonical core evidence.",
        "Read back the exact binary emitted by the current build.",
      ));
    }
  });
  for (const node of nodes) {
    for (const controller of node.controllers) {
      const pathKind = pathForControllerName(controller.controllerName);
      if (pathKind && !expectedTrackKeys.has(trackKey(node.number, pathKind))) {
        diagnostics.push(diagnostic(
          "M2A-ANIMATION-READBACK-TRACK-UNEXPECTED",
          `${path}.materializedTracks`,
          `Binary readback contains an unexpected ${pathKind} controller for target ${node.number}.`,
          "Read back the exact binary emitted by the current build.",
        ));
      }
    }
  }
}

function compareClip(
  expected: AuthoredAnimationClipV1,
  actual: ReadbackAnimation,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
) {
  compareScalar(
    expected.lengthSeconds,
    actual.length,
    "M2A-ANIMATION-READBACK-LENGTH-MISMATCH",
    `${path}.lengthSeconds`,
    "Authored animation length differs from canonical readback.",
    diagnostics,
  );
  compareScalar(
    expected.transitionSeconds,
    actual.transition,
    "M2A-ANIMATION-READBACK-TRANSITION-MISMATCH",
    `${path}.transitionSeconds`,
    "Authored animation transition differs from canonical readback.",
    diagnostics,
  );
  if (expected.animationRoot !== actual.animationRoot) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-ANIMROOT-MISMATCH",
      `${path}.animationRoot`,
      "Authored animation root differs from canonical readback.",
      "Rebuild the package from the current authored animation.",
    ));
  }
  if (!eventsMatch(expected, actual)) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-EVENTS-MISMATCH",
      `${path}.events`,
      "Authored animation events differ from canonical readback.",
      "Rebuild and verify event names and times.",
    ));
  }
  compareTracks(expected, actual, path, diagnostics);
}

function compareTracks(
  expected: AuthoredAnimationClipV1,
  actual: ReadbackAnimation,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
) {
  const nodes = flattenNodes(actual.nodeTree.roots);
  const expectedTrackKeys = new Set<string>();

  for (const track of expected.tracks) {
    const trackPath = `${path}.tracks.${track.id}`;
    const key = trackKey(track.targetNodeId, track.path);
    if (expectedTrackKeys.has(key)) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-TRACK-DUPLICATE",
        trackPath,
        "Authored animation contains duplicate target/path tracks.",
        "Keep exactly one translation or rotation track per target node.",
      ));
      continue;
    }
    expectedTrackKeys.add(key);

    const targetNodes = nodes.filter(({ number }) => number === track.targetNodeId);
    if (targetNodes.length !== 1) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-READBACK-TRACK-TARGET-MISMATCH",
        `${trackPath}.targetNodeId`,
        targetNodes.length === 0
          ? `Canonical readback is missing target node ${track.targetNodeId}.`
          : `Canonical readback contains duplicate target node ${track.targetNodeId}.`,
        "Rebuild with the current rig and authored animation targets.",
      ));
      continue;
    }

    const controllerName = controllerNameForPath(track.path);
    const controllers = targetNodes[0]!.controllers.filter(
      (controller) => controller.controllerName === controllerName,
    );
    if (controllers.length !== 1) {
      diagnostics.push(diagnostic(
        controllers.length === 0
          ? "M2A-ANIMATION-READBACK-TRACK-MISSING"
          : "M2A-ANIMATION-READBACK-TRACK-DUPLICATE",
        `${trackPath}.path`,
        controllers.length === 0
          ? `Canonical readback is missing ${track.path} for target ${track.targetNodeId}.`
          : `Canonical readback contains duplicate ${track.path} for target ${track.targetNodeId}.`,
        "Rebuild the package from the current authored animation tracks.",
      ));
      continue;
    }
    compareController(track, controllers[0]!, trackPath, diagnostics);
  }

  for (const node of nodes) {
    for (const controller of node.controllers) {
      const pathKind = pathForControllerName(controller.controllerName);
      if (pathKind && !expectedTrackKeys.has(trackKey(node.number, pathKind))) {
        diagnostics.push(diagnostic(
          "M2A-ANIMATION-READBACK-TRACK-UNEXPECTED",
          `${path}.tracks`,
          `Canonical readback contains unexpected ${pathKind} for target ${node.number}.`,
          "Rebuild the package from the current authored animation tracks.",
        ));
      }
    }
  }
}

function compareController(
  expected: AuthoredAnimationTrackV1,
  actual: ReadbackController,
  path: string,
  diagnostics: AnimationStudioDiagnosticV1[],
) {
  const expectedTimes = expected.keyframes.map(({ timeSeconds }) => timeSeconds);
  if (!numberArraysMatch(expectedTimes, actual.times)) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-TIMES-MISMATCH",
      `${path}.keyframes`,
      "Authored keyframe times differ from canonical readback.",
      "Rebuild the package from the current authored keyframes.",
    ));
  }

  const expectedValues = expected.keyframes.map(({ value }) => value);
  if (!valueRowsMatch(expected.path, expectedValues, actual.values)) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-READBACK-VALUES-MISMATCH",
      `${path}.keyframes`,
      "Authored keyframe values differ from canonical readback.",
      "Rebuild and verify the authored transform values.",
    ));
  }
}

function eventsMatch(
  expected: AuthoredAnimationClipV1,
  actual: ReadbackAnimation,
) {
  const authoredEvents = [...expected.events].sort(
    (left, right) => left.timeSeconds - right.timeSeconds,
  );
  if (authoredEvents.length !== actual.events.length) return false;
  return authoredEvents.every((event, index) => {
    const readbackEvent = actual.events[index];
    return readbackEvent !== undefined
      && event.name === readbackEvent.name
      && near(event.timeSeconds, readbackEvent.time);
  });
}

function valueRowsMatch(
  path: AuthoredAnimationTrackPathV1,
  expected: readonly (readonly number[])[],
  actual: readonly (readonly number[])[],
) {
  if (expected.length !== actual.length) return false;
  return expected.every((row, index) => {
    const actualRow = actual[index];
    if (!actualRow) return false;
    if (path === "ROTATION") {
      const left = canonicalQuaternion(row);
      const right = canonicalQuaternion(actualRow);
      return left !== null && right !== null && numberArraysMatch(left, right);
    }
    return row.length === 3
      && actualRow.length === 3
      && numberArraysMatch(row, actualRow);
  });
}

function canonicalQuaternion(
  value: readonly number[],
): [number, number, number, number] | null {
  if (value.length !== 4 || value.some((item) => !Number.isFinite(item))) return null;
  const magnitude = Math.hypot(value[0]!, value[1]!, value[2]!, value[3]!);
  if (!Number.isFinite(magnitude) || magnitude <= Number.EPSILON) return null;
  const normalized = value.map((item) => item / magnitude) as [
    number,
    number,
    number,
    number,
  ];
  const negate = normalized[3] < 0
    || (normalized[3] === 0
      && normalized.slice(0, 3).find((item) => item !== 0)! < 0);
  return negate
    ? normalized.map((item) => -item) as [number, number, number, number]
    : normalized;
}

function flattenNodes(roots: readonly ReadbackNode[]) {
  const nodes: ReadbackNode[] = [];
  const visit = (node: ReadbackNode) => {
    nodes.push(node);
    node.children.forEach(visit);
  };
  roots.forEach(visit);
  return nodes;
}

function controllerNameForPath(path: AuthoredAnimationTrackPathV1) {
  return path === "TRANSLATION" ? "position" : "orientation";
}

function pathForControllerName(
  controllerName: string | undefined,
): AuthoredAnimationTrackPathV1 | null {
  if (controllerName === "position") return "TRANSLATION";
  if (controllerName === "orientation") return "ROTATION";
  return null;
}

function trackKey(
  targetNodeId: number,
  path: AuthoredAnimationTrackPathV1,
) {
  return `${targetNodeId}:${path}`;
}

function numberArraysMatch(
  left: readonly number[],
  right: readonly number[],
) {
  return left.length === right.length
    && left.every((value, index) => near(value, right[index]));
}

function compareScalar(
  expected: number,
  actual: number,
  code: string,
  path: string,
  message: string,
  diagnostics: AnimationStudioDiagnosticV1[],
) {
  if (!near(expected, actual)) {
    diagnostics.push(diagnostic(
      code,
      path,
      message,
      "Rebuild the package from the current authored animation.",
    ));
  }
}

function near(left: number, right: number | undefined) {
  return right !== undefined
    && Number.isFinite(left)
    && Number.isFinite(right)
    && Math.abs(left - right) <= READBACK_EPSILON;
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
): AnimationStudioDiagnosticV1 {
  return {
    schemaVersion: 1,
    code,
    path,
    level: "BLOCKING",
    message,
    action,
  };
}
