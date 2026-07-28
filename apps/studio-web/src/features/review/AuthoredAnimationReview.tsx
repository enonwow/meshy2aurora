import type {
  AnimationStudioDocumentV1,
  AuthoredAnimationClipV1,
  CreatureAnimationAuthoringV2,
  CustomAnimationClipReferenceV2,
  CustomAnimationDefinitionV2,
} from "../animation-studio/types";
import type { AnimationStudioReadbackReconciliationV1 } from "./reconcileAnimationStudioReadback";

export interface AuthoredAnimationReviewProps {
  studio: AnimationStudioDocumentV1;
  authoring: CreatureAnimationAuthoringV2;
  studioFingerprintSha256: string;
  reconciliation: AnimationStudioReadbackReconciliationV1;
}

export function AuthoredAnimationReview({
  studio,
  authoring,
  studioFingerprintSha256,
  reconciliation,
}: AuthoredAnimationReviewProps) {
  const clipNames = new Map(studio.authoredClips.map(({ id, name }) => [id, name]));

  return (
    <section className="panel authored-animation-review" aria-label="Authored animations">
      <header>
        <div>
          <span>Animation Studio</span>
          <h3>Authored animations</h3>
        </div>
        <strong data-status={reconciliation.status.toLowerCase()}>
          Readback {reconciliation.status}
        </strong>
      </header>

      <dl>
        <div>
          <dt>Studio document</dt>
          <dd>
            <strong>{studio.status}</strong>
            <span>revision {studio.authoringRevision}</span>
          </dd>
        </div>
        <div>
          <dt>Studio fingerprint</dt>
          <dd title={studioFingerprintSha256}>
            <code>{shortFingerprint(studioFingerprintSha256)}</code>
          </dd>
        </div>
        <div>
          <dt>Source</dt>
          <dd>
            <strong>Source GLB unchanged</strong>
            <span>{studio.sourceRevision}</span>
          </dd>
        </div>
        <div>
          <dt>Canonical readback</dt>
          <dd>
            <strong>{reconciliation.status}</strong>
            <span>
              {reconciliation.matchedClipIds.length}/{reconciliation.checkedClipCount} clips matched
            </span>
          </dd>
        </div>
      </dl>

      <div>
        <h4>Authored clip library</h4>
        {studio.authoredClips.length === 0 ? (
          <p>No authored animations.</p>
        ) : (
          <ul>
            {studio.authoredClips.map((clip) => (
              <li key={clip.id}>
                <article>
                  <header>
                    <strong>{clip.name}</strong>
                    <span data-status={clip.status.toLowerCase()}>{clip.status}</span>
                  </header>
                  <p>{sourceProvenance(clip)}</p>
                  <p>{clipUsage(clip.id, authoring)}</p>
                  <small>
                    Stable ID {clip.id} · revision {clip.revision} · {clip.tracks.length} track(s) · {clip.events.length} event(s)
                  </small>
                </article>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div>
        <h4>Custom animation usage</h4>
        {authoring.customAnimations.length === 0 ? (
          <p>No custom animations reference the authored library.</p>
        ) : (
          <ul>
            {authoring.customAnimations.map((custom) => (
              <li key={custom.id}>
                <article>
                  <header>
                    <strong>{custom.name}</strong>
                    <span>{playbackLabel(custom.playback)}</span>
                  </header>
                  <p>{customDefinitionSummary(custom, clipNames)}</p>
                  <p>{customBase42Usage(custom.id, authoring)}</p>
                  <small>
                    {custom.provenance.provider} · {custom.provenance.ownership} · {custom.provenance.assetId}
                  </small>
                </article>
              </li>
            ))}
          </ul>
        )}
      </div>

      {reconciliation.diagnostics.length > 0 && (
        <div role="alert">
          <h4>Readback blockers</h4>
          <ul>
            {reconciliation.diagnostics.map((item, index) => (
              <li key={`${item.code}:${item.path}:${index}`}>
                <strong>{item.code}</strong>
                <span>{item.message}</span>
                <small>{item.path} · {item.action}</small>
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}

function sourceProvenance(clip: AuthoredAnimationClipV1) {
  const details = clip.source.kind === "SOURCE_CLIP_COPY"
    ? [
        clip.source.sourceClipName,
        clip.source.sourceClipFingerprint
          ? shortFingerprint(clip.source.sourceClipFingerprint)
          : null,
      ]
    : clip.source.kind === "PROCEDURAL_TEMPLATE"
      ? [clip.source.proceduralTemplate]
      : [];
  return [
    clip.source.kind,
    ...details.filter((item): item is string => Boolean(item)),
    clip.source.sourceRevision,
  ].join(" · ");
}

function clipUsage(
  clipId: string,
  authoring: CreatureAnimationAuthoringV2,
) {
  const customIds = new Set(
    authoring.customAnimations
      .filter((custom) => customClipIds(custom).includes(clipId))
      .map(({ id }) => id),
  );
  const slots = authoring.assignments
    .filter((assignment) => (
      assignment.sourceKind === "CUSTOM"
      && assignment.customAnimationId !== null
      && customIds.has(assignment.customAnimationId)
    ))
    .map(({ targetSlot }) => targetSlot);
  return slots.length > 0
    ? `Base 42: ${slots.join(", ")}`
    : "Base 42: not assigned";
}

function customBase42Usage(
  customId: string,
  authoring: CreatureAnimationAuthoringV2,
) {
  const slots = authoring.assignments
    .filter(({ customAnimationId }) => customAnimationId === customId)
    .map(({ targetSlot }) => targetSlot);
  return slots.length > 0
    ? `Base 42 assignments: ${slots.join(", ")}`
    : "Base 42 assignments: none";
}

function customDefinitionSummary(
  custom: CustomAnimationDefinitionV2,
  clipNames: ReadonlyMap<string, string>,
) {
  if (custom.playback === "ONE_SHOT") {
    return `Clip: ${referenceLabel(custom.clipReference, clipNames)}`;
  }
  return custom.phases.length > 0
    ? custom.phases
        .map(({ phase, clipReference }) => (
          `${phase}: ${referenceLabel(clipReference, clipNames)}`
        ))
        .join(" · ")
    : "No phases";
}

function referenceLabel(
  reference: CustomAnimationClipReferenceV2 | null,
  clipNames: ReadonlyMap<string, string>,
) {
  if (!reference) return "not selected";
  if (reference.sourceKind === "AUTHORED_CLIP") {
    return reference.authoredClipId
      ? clipNames.get(reference.authoredClipId) ?? reference.authoredClipId
      : "missing authored clip";
  }
  return reference.sourceClipName ?? "missing source clip";
}

function customClipIds(custom: CustomAnimationDefinitionV2) {
  return [
    custom.clipReference,
    ...custom.phases.map(({ clipReference }) => clipReference),
  ].flatMap((reference) => (
    reference?.sourceKind === "AUTHORED_CLIP" && reference.authoredClipId
      ? [reference.authoredClipId]
      : []
  ));
}

function playbackLabel(playback: CustomAnimationDefinitionV2["playback"]) {
  return playback === "ONE_SHOT" ? "ONE SHOT" : "LOOPING PHASED";
}

function shortFingerprint(value: string) {
  return value.length > 12 ? `${value.slice(0, 12)}...` : value;
}
