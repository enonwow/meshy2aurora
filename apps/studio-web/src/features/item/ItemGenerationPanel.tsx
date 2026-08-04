import { useEffect, useState } from "react";

import {
  DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
  type MeshyArtifactProvenance,
  type MeshyBridgeClient,
  type MeshyRun,
} from "../meshy/bridge";
import {
  bindItemGenerationTask,
  createItemGenerationPlan,
  createItemGenerationSession,
  deserializeItemGenerationSession,
  recordItemGenerationArtifact,
  recordItemGenerationRun,
  recordRecoveredItemGenerationArtifact,
  serializeItemGenerationSession,
  setItemGenerationConcept,
  type ItemGenerationSessionV1,
} from "./itemGeneration";
import type { ItemBaseItemRow } from "./types";

const PAID_TERMINAL_POLL_LIMIT = 180;

function sessionStorageKey(baseitemsSha256: string, baseItem: number) {
  return `meshy2aurora:item-generation:v1:${baseitemsSha256}:${baseItem}`;
}

function sessionMatchesRow(
  session: ItemGenerationSessionV1,
  row: ItemBaseItemRow,
  baseitemsSha256: string,
) {
  const expectedFields = row.partSlots
    .filter(({ sourceKind }) => sourceKind === "MESHY_GLB")
    .map(({ field }) => field);
  return session.baseitemsSha256 === baseitemsSha256
    && session.baseItem === row.baseItem
    && session.itemClass === row.itemClass
    && session.modelType === row.modelType
    && JSON.stringify(session.slots.map(({ field }) => field)) === JSON.stringify(expectedFields);
}

function restoredSession(row: ItemBaseItemRow, baseitemsSha256: string) {
  try {
    const serialized = window.localStorage.getItem(sessionStorageKey(baseitemsSha256, row.baseItem));
    if (!serialized) return undefined;
    const session = deserializeItemGenerationSession(serialized);
    return sessionMatchesRow(session, row, baseitemsSha256) ? session : undefined;
  } catch {
    return undefined;
  }
}

async function sha256(bytes: ArrayBuffer) {
  const digest = await crypto.subtle.digest("SHA-256", bytes.slice(0));
  return Array.from(new Uint8Array(digest), (value) => value.toString(16).padStart(2, "0")).join("");
}

function base64(bytes: Uint8Array) {
  let binary = "";
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000));
  }
  return btoa(binary);
}

async function conceptDataUri(file: File) {
  const bytes = new Uint8Array(await file.arrayBuffer());
  return `data:${file.type};base64,${base64(bytes)}`;
}

function taskId(run: MeshyRun) {
  return run.taskIds.PREVIEW ?? run.taskIds.REFINE ?? null;
}

function delay(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

export interface ItemGenerationPanelProps {
  readonly row: ItemBaseItemRow;
  readonly baseitemsSha256: string;
  readonly bridge?: MeshyBridgeClient;
  readonly onArtifact: (field: string, file: File, provenance: MeshyArtifactProvenance) => void;
  readonly onSessionChange?: (session: ItemGenerationSessionV1) => void;
}

export function ItemGenerationPanel({
  row,
  baseitemsSha256,
  bridge,
  onArtifact,
  onSessionChange,
}: ItemGenerationPanelProps) {
  const newSession = () => restoredSession(row, baseitemsSha256) ?? createItemGenerationSession(
      row,
      baseitemsSha256,
      crypto.randomUUID(),
      new Date().toISOString(),
    );
  const [session, setSession] = useState<ItemGenerationSessionV1>(newSession);
  const [conceptFiles, setConceptFiles] = useState<Record<string, File>>({});
  const [recoveryTaskIds, setRecoveryTaskIds] = useState<Record<string, string>>(() => Object.fromEntries(
    session.slots.flatMap((slot) => slot.run?.taskId ? [[slot.field, slot.run.taskId]] : []),
  ));
  const [ownerCap, setOwnerCap] = useState(
    session.ownerCreditCap ?? row.capability.meshySourceCount * 30,
  );
  const [pairingCode, setPairingCode] = useState("");
  const [sessionToken, setSessionToken] = useState<string>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    const next = newSession();
    setSession(next);
    setConceptFiles({});
    setRecoveryTaskIds(Object.fromEntries(
      next.slots.flatMap((slot) => slot.run?.taskId ? [[slot.field, slot.run.taskId]] : []),
    ));
    setOwnerCap(next.ownerCreditCap ?? row.capability.meshySourceCount * 30);
    setError(undefined);
  }, [baseitemsSha256, row.baseItem]);

  useEffect(() => {
    if (session.baseitemsSha256 === baseitemsSha256 && session.baseItem === row.baseItem) {
      try {
        window.localStorage.setItem(
          sessionStorageKey(baseitemsSha256, row.baseItem),
          serializeItemGenerationSession(session),
        );
      } catch {
        setError("The non-secret Item recovery sidecar could not be saved in local browser storage.");
      }
      onSessionChange?.(session);
    }
  }, [baseitemsSha256, onSessionChange, row.baseItem, session]);

  const pair = async () => {
    if (!bridge) return;
    setBusy(true);
    setError(undefined);
    try {
      const pairing = pairingCode.trim()
        ? await bridge.pair({ pairingCode: pairingCode.trim() })
        : await bridge.pairAutomatically();
      setSessionToken(pairing.sessionToken);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const restoreSidecarFile = async (file: File) => {
    setBusy(true);
    setError(undefined);
    try {
      if (file.size <= 0 || file.size > 1_048_576) {
        throw new Error("The Item recovery sidecar must be a non-empty JSON file no larger than 1 MiB.");
      }
      const restored = deserializeItemGenerationSession(await file.text());
      if (!sessionMatchesRow(restored, row, baseitemsSha256)) {
        throw new Error("The Item recovery sidecar belongs to a different baseitems.2da, BaseItem, ModelType, or slot list.");
      }
      setSession(restored);
      setConceptFiles({});
      setRecoveryTaskIds(Object.fromEntries(
        restored.slots.flatMap((slot) => slot.run?.taskId ? [[slot.field, slot.run.taskId]] : []),
      ));
      setOwnerCap(restored.ownerCreditCap ?? row.capability.meshySourceCount * 30);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const downloadSidecar = () => {
    const bytes = serializeItemGenerationSession(session);
    const url = URL.createObjectURL(new Blob([bytes], { type: "application/json" }));
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `item-generation-${row.baseItem}-${session.sessionId.replace(/[^a-z0-9_-]+/gi, "-")}.json`;
    anchor.rel = "noopener";
    document.body.append(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 0);
  };

  const selectConcept = async (field: string, file: File) => {
    if (!["image/png", "image/jpeg"].includes(file.type)) {
      setError(`${field} requires a PNG or JPEG concept.`);
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      const bytes = await file.arrayBuffer();
      const next = setItemGenerationConcept(session, field, {
        fileName: file.name,
        mimeType: file.type as "image/png" | "image/jpeg",
        byteLength: bytes.byteLength,
        sha256: await sha256(bytes),
      });
      setConceptFiles((current) => ({ ...current, [field]: file }));
      setSession(next);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const reviewBatch = async () => {
    if (!bridge || !sessionToken) return;
    setBusy(true);
    setError(undefined);
    try {
      const pending = session.slots.filter((slot) => !slot.artifact);
      const [balance, entries] = await Promise.all([
        bridge.balance(sessionToken),
        Promise.all(pending.map(async (slot) => {
          const concept = conceptFiles[slot.field];
          if (!concept) throw new Error(`${slot.label} requires its exact concept file.`);
          const preview = await bridge.previewRun(sessionToken, {
            profileId: "S1-static-prop/v1",
            prompt: `${row.label} ${slot.role.toLowerCase()} Item part; isolated exact component`,
            source: "IMAGE",
            imageDataUrls: [await conceptDataUri(concept)],
            geometryTarget: "BALANCED",
            apiOptions: {
              ...DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
              targetPolycount: slot.targetPolycount,
              rigHumanoid: false,
            },
          });
          return {
            field: slot.field,
            previewId: preview.previewId,
            maximumCredits: preview.maximumCredits,
            targetPolycount: slot.targetPolycount,
          };
        })),
      ]);
      setSession(createItemGenerationPlan(session, entries, ownerCap, balance.availableCredits));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const pollToTerminal = async (runId: string) => {
    if (!bridge || !sessionToken) throw new Error("The local Bridge is not paired.");
    for (let attempt = 0; attempt < PAID_TERMINAL_POLL_LIMIT; attempt += 1) {
      const current = await bridge.getRun(sessionToken, runId);
      if (current.status === "READY") return current;
      if (current.status === "FAILED" || current.status === "CANCELED") {
        throw new Error(current.error?.message ?? `Meshy run ${current.status.toLowerCase()}.`);
      }
      await delay(1_000);
    }
    throw new Error("Meshy run did not reach a terminal state before the local polling limit.");
  };

  const confirmBatch = async () => {
    if (!bridge || !sessionToken) return;
    setBusy(true);
    setError(undefined);
    let next = session;
    try {
      for (const reviewed of session.slots.filter((slot) => slot.status === "REVIEWED")) {
        const previewId = reviewed.preview?.previewId;
        if (!previewId) throw new Error(`${reviewed.label} has no reviewed preview identity.`);
        const created = await bridge.createRun(sessionToken, {
          previewId,
          confirmationNonce: `item:${session.sessionId}:${reviewed.field}:${previewId}`,
        });
        next = recordItemGenerationRun(next, reviewed.field, {
          runId: created.id,
          taskId: taskId(created),
          createdAt: created.createdAt,
        });
        setSession(next);
        const ready = await pollToTerminal(created.id);
        const exactTaskId = taskId(ready);
        const currentSlot = next.slots.find(({ field }) => field === reviewed.field);
        if (!currentSlot?.run?.taskId) {
          if (!exactTaskId) throw new Error(`${reviewed.label} completed without an exact Meshy task ID.`);
          next = bindItemGenerationTask(next, reviewed.field, exactTaskId);
          setSession(next);
        } else if (exactTaskId !== currentSlot.run.taskId) {
          throw new Error(`${reviewed.label} returned a different task identity than the recorded run.`);
        }
        const artifact = await bridge.downloadArtifact(sessionToken, created.id);
        if (artifact.provenance.taskIds.PREVIEW !== exactTaskId
          || artifact.provenance.consumedCredits === undefined
          || !artifact.provenance.createdAt
          || !artifact.provenance.finishedAt) {
          throw new Error(`${reviewed.label} artifact is missing or changed its exact task, cost, or timestamp provenance.`);
        }
        next = recordItemGenerationArtifact(next, reviewed.field, {
          sha256: artifact.provenance.sha256,
          byteLength: artifact.provenance.byteLength,
          consumedCredits: artifact.provenance.consumedCredits,
          finishedAt: artifact.provenance.finishedAt,
        });
        setSession(next);
        onArtifact(reviewed.field, artifact.file, artifact.provenance);
      }
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const recover = async (field: string) => {
    if (!bridge || !sessionToken) return;
    const slot = session.slots.find((candidate) => candidate.field === field);
    const requestedTaskId = (recoveryTaskIds[field] ?? slot?.run?.taskId ?? "").trim();
    if (!requestedTaskId) return;
    setBusy(true);
    setError(undefined);
    try {
      const artifact = await bridge.recoverImageTo3dArtifact(sessionToken, requestedTaskId);
      const exactTaskId = artifact.provenance.taskIds.PREVIEW;
      if (exactTaskId !== requestedTaskId
        || artifact.provenance.consumedCredits === undefined
        || !artifact.provenance.createdAt
        || !artifact.provenance.finishedAt) {
        throw new Error("Recovered Image-to-3D artifact lacks exact task, cost, or timestamp provenance.");
      }
      if (slot?.artifact) {
        if (
          slot.run?.taskId !== exactTaskId
          || slot.artifact.sha256 !== artifact.provenance.sha256
          || slot.artifact.byteLength !== artifact.provenance.byteLength
          || slot.artifact.consumedCredits !== artifact.provenance.consumedCredits
          || slot.artifact.finishedAt !== artifact.provenance.finishedAt
        ) {
          throw new Error("Recovered artifact differs from the persisted exact Item session.");
        }
        onArtifact(field, artifact.file, artifact.provenance);
        return;
      }
      const next = recordRecoveredItemGenerationArtifact(session, field, {
        taskId: exactTaskId,
        createdAt: artifact.provenance.createdAt,
        artifact: {
          sha256: artifact.provenance.sha256,
          byteLength: artifact.provenance.byteLength,
          consumedCredits: artifact.provenance.consumedCredits,
          finishedAt: artifact.provenance.finishedAt,
        },
      });
      setSession(next);
      onArtifact(field, artifact.file, artifact.provenance);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const pending = session.slots.filter((slot) => !slot.artifact);
  const canReview = Boolean(sessionToken)
    && pending.length > 0
    && pending.every((slot) => slot.status === "CONCEPT_READY" && conceptFiles[slot.field]);
  const canConfirm = Boolean(sessionToken) && session.slots.some((slot) => slot.status === "REVIEWED");

  return (
    <section className="item-generation panel" aria-label="Item Meshy generation">
      <header>
        <div><p className="eyebrow">Separate Item generation case</p><h2>Generate exact Aurora part slots</h2></div>
        <code>{session.status}</code>
      </header>
      <p>
        BaseItem {row.baseItem} resolves the slots. This is not a Weapon/Shield split:
        each concept and task is bound directly to its UTI field.
      </p>
      <div className="item-generation__pairing">
        <label><span>Local Bridge pairing code</span><input aria-label="Item Meshy pairing code" value={pairingCode} onChange={(event) => setPairingCode(event.currentTarget.value)} /></label>
        <button type="button" className="button button--secondary" onClick={pair} disabled={!bridge || busy || Boolean(sessionToken)}>{sessionToken ? "Bridge paired" : "Pair Bridge"}</button>
      </div>
      <div className="item-generation__slots">
        {session.slots.map((slot) => (
          <article key={slot.field} data-status={slot.status}>
            <header><b>{slot.role}</b><strong>{slot.label}</strong><code>{slot.field}</code></header>
            <label>
              <span>Exact concept image</span>
              <input aria-label={`${slot.field} concept image`} type="file" accept="image/png,image/jpeg" disabled={busy || Boolean(slot.artifact)} onChange={(event) => {
                const file = event.currentTarget.files?.[0];
                if (file) void selectConcept(slot.field, file);
              }} />
            </label>
            <small>{slot.concept ? `${slot.concept.fileName} · ${slot.concept.sha256.slice(0, 12)}...` : "No concept bound"}</small>
            <label>
              <span>Existing Image-to-3D task ID (no new paid task)</span>
              <input aria-label={`${slot.field} recovery task ID`} value={recoveryTaskIds[slot.field] ?? slot.run?.taskId ?? ""} disabled={busy} onChange={(event) => setRecoveryTaskIds((current) => ({ ...current, [slot.field]: event.currentTarget.value }))} />
            </label>
            <button type="button" className="button button--secondary" disabled={busy || !sessionToken || !slot.concept || !(recoveryTaskIds[slot.field] ?? slot.run?.taskId)?.trim()} onClick={() => void recover(slot.field)}>{slot.artifact ? "Re-download exact task" : "Recover exact task"}</button>
            <footer><span>{slot.targetPolycount.toLocaleString()} target triangles</span><strong>{slot.status}</strong></footer>
          </article>
        ))}
      </div>
      <div className="item-generation__review">
        <label><span>Owner aggregate credit cap</span><input aria-label="Item generation owner credit cap" type="number" min="0" value={ownerCap} onChange={(event) => setOwnerCap(Number(event.currentTarget.value))} /></label>
        <dl>
          <div><dt>Unfinished slots</dt><dd>{pending.length}</dd></div>
          <div><dt>Reviewed maximum</dt><dd>{session.maximumCredits} credits</dd></div>
          <div><dt>Balance at review</dt><dd>{session.balanceAtReview ?? "not checked"}</dd></div>
        </dl>
        <div>
          <button type="button" className="button button--secondary" disabled={!canReview || busy} onClick={() => void reviewBatch()}>Review batch cost</button>
          <button type="button" className="button button--primary" disabled={!canConfirm || busy} onClick={() => void confirmBatch()}>Confirm paid Item batch</button>
        </div>
        <small>Confirmation creates only the reviewed unfinished slots and never replaces a successful task.</small>
      </div>
      <details>
        <summary>Recovery sidecar · saved locally without credentials</summary>
        <pre>{serializeItemGenerationSession(session)}</pre>
        <div>
          <button type="button" className="button button--secondary" disabled={busy} onClick={downloadSidecar}>Download recovery sidecar</button>
          <label className="button button--secondary">
            <span>Restore recovery sidecar</span>
            <input aria-label="Restore Item generation sidecar" type="file" accept=".json,application/json" disabled={busy} onChange={(event) => {
              const file = event.currentTarget.files?.[0];
              if (file) void restoreSidecarFile(file);
            }} />
          </label>
        </div>
        <button type="button" className="button button--secondary" disabled={busy} onClick={() => {
          const next = createItemGenerationSession(
            row,
            baseitemsSha256,
            crypto.randomUUID(),
            new Date().toISOString(),
          );
          setSession(next);
          setConceptFiles({});
          setRecoveryTaskIds({});
          setOwnerCap(row.capability.meshySourceCount * 30);
          setError(undefined);
        }}>Start new Item generation session</button>
      </details>
      {error ? <p className="item-error" role="alert">{error}</p> : null}
    </section>
  );
}
