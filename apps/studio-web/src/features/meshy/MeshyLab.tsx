import { useState } from "react";
import {
  type MeshyArtifactProvenance,
  type MeshyBridgeClient,
  type MeshyGeometryTarget,
  type MeshyProfile,
  type MeshyRun,
  type MeshyRunPreview,
  MESHY_GEOMETRY_TARGETS,
} from "./bridge";

type LabScreen = "CONNECT" | "CONFIGURE" | "REVIEW" | "RUN";

export interface MeshyLabProps {
  readonly bridge: MeshyBridgeClient;
  readonly onBack: () => void;
  readonly onImport: (file: File, provenance: MeshyArtifactProvenance) => void;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function profileCode(profile: MeshyProfile) {
  return profile.id.slice(0, 2);
}

export function MeshyLab({ bridge, onBack, onImport }: MeshyLabProps) {
  const [screen, setScreen] = useState<LabScreen>("CONNECT");
  const [pairingCode, setPairingCode] = useState("");
  const [sessionToken, setSessionToken] = useState<string>();
  const [profiles, setProfiles] = useState<readonly MeshyProfile[]>([]);
  const [profile, setProfile] = useState<MeshyProfile>();
  const [availableCredits, setAvailableCredits] = useState<number>();
  const [prompt, setPrompt] = useState("");
  const [geometryTarget, setGeometryTarget] = useState<MeshyGeometryTarget>("AURORA_PROOF");
  const [h1Preflight, setH1Preflight] = useState({ standardHumanoid: false, clearLimbs: false, noWeapon: false });
  const [preview, setPreview] = useState<MeshyRunPreview>();
  const [run, setRun] = useState<MeshyRun>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();

  const perform = async (operation: () => Promise<void>) => {
    setBusy(true);
    setError(undefined);
    try {
      await operation();
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setBusy(false);
    }
  };

  const connect = () => void perform(async () => {
    const health = await bridge.health();
    if (health.protocolVersion !== 1 || health.status !== "READY") {
      throw new Error("The local Meshy Bridge does not support this Studio protocol.");
    }
    const pairing = await bridge.pair({ pairingCode });
    const [balance, availableProfiles] = await Promise.all([
      bridge.balance(pairing.sessionToken),
      bridge.profiles(pairing.sessionToken),
    ]);
    if (!availableProfiles.length) throw new Error("The local Bridge did not expose any supported proof profiles.");
    setSessionToken(pairing.sessionToken);
    setAvailableCredits(balance.availableCredits);
    setProfiles(availableProfiles);
    setProfile(availableProfiles[0]);
    setScreen("CONFIGURE");
  });

  const review = () => void perform(async () => {
    if (!sessionToken || !profile) return;
    const nextPreview = await bridge.previewRun(sessionToken, {
      profileId: profile.id,
      prompt,
      geometryTarget,
      ...(profile.id.startsWith("H1") && h1Preflight.standardHumanoid && h1Preflight.clearLimbs && h1Preflight.noWeapon
        ? { h1Preflight: { standardHumanoid: true as const, clearLimbs: true as const, noWeapon: true as const } }
        : {}),
    });
    setPreview(nextPreview);
    setScreen("REVIEW");
  });

  const generate = () => void perform(async () => {
    if (!sessionToken || !preview) return;
    const nextRun = await bridge.createRun(sessionToken, {
      previewId: preview.previewId,
      confirmationNonce: crypto.randomUUID(),
    });
    setRun(nextRun);
    setScreen("RUN");
  });

  const refresh = () => void perform(async () => {
    if (!sessionToken || !run) return;
    setRun(await bridge.getRun(sessionToken, run.id));
  });

  const cancel = () => void perform(async () => {
    if (!sessionToken || !run) return;
    setRun(await bridge.cancelRun(sessionToken, run.id));
  });

  const importArtifact = () => void perform(async () => {
    if (!sessionToken || !run) return;
    const artifact = await bridge.downloadArtifact(sessionToken, run.id);
    onImport(artifact.file, artifact.provenance);
  });

  const downloadProvenance = () => void perform(async () => {
    if (!sessionToken || !run) return;
    const provenance = await bridge.provenance(sessionToken, run.id);
    const url = URL.createObjectURL(new Blob([JSON.stringify(provenance, null, 2)], { type: "application/json" }));
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `meshy-${run.id}-provenance.json`;
    anchor.click();
    URL.revokeObjectURL(url);
  });

  return (
    <section className="meshy-lab" aria-labelledby="meshy-lab-heading">
      <header className="meshy-lab__header">
        <button type="button" className="button button--quiet" onClick={onBack}>Back to Source</button>
        <div><p className="eyebrow">Optional local integration</p><h1 id="meshy-lab-heading">Meshy Lab</h1></div>
        <span className="status-badge status-badge--neutral">Local Bridge only</span>
      </header>

      {error ? <p className="meshy-lab__error" role="alert">{error}</p> : null}

      {screen === "CONNECT" ? (
        <div className="meshy-lab__connect panel">
          <h2>Connect local bridge</h2>
          <p>Meshy credentials stay in the local Bridge process. Studio receives only a temporary local session.</p>
          <label htmlFor="meshy-pairing-code">Pairing code</label>
          <input id="meshy-pairing-code" value={pairingCode} onChange={(event) => setPairingCode(event.target.value)} autoComplete="off" />
          <button type="button" className="button button--primary" onClick={connect} disabled={busy || !pairingCode.trim()}>
            Connect local bridge
          </button>
        </div>
      ) : null}

      {screen === "CONFIGURE" && profile ? (
        <div className="meshy-lab__workspace">
          <aside className="panel meshy-lab__profiles" aria-label="Proof profiles">
            <h2>Proof profiles</h2>
            {profiles.map((candidate) => (
              <button
                key={candidate.id}
                type="button"
                className={`meshy-lab__profile${candidate.id === profile.id ? " meshy-lab__profile--selected" : ""}`}
                onClick={() => setProfile(candidate)}
                aria-pressed={candidate.id === profile.id}
              >
                <strong>{profileCode(candidate)} · {candidate.label}</strong>
                <span>{candidate.description}</span>
              </button>
            ))}
          </aside>
          <main className="panel meshy-lab__configuration">
            <p className="eyebrow">Proof asset workflow</p>
            <h2>Configure {profileCode(profile)} asset</h2>
            <p>Available balance: {availableCredits ?? "unavailable"} credits</p>
            <label htmlFor="meshy-asset-prompt">Asset prompt</label>
            <textarea id="meshy-asset-prompt" value={prompt} onChange={(event) => setPrompt(event.target.value)} maxLength={600} />
            <fieldset>
              <legend>Geometry target</legend>
              {MESHY_GEOMETRY_TARGETS.map((target) => (
                <label key={target}>
                  <input type="radio" name="meshy-geometry-target" checked={geometryTarget === target} onChange={() => setGeometryTarget(target)} />
                  {target === "AURORA_PROOF" ? "Aurora proof (target: 1,500 triangles)" : target.replaceAll("_", " ").toLowerCase()}
                </label>
              ))}
            </fieldset>
            {profile.id.startsWith("H1") ? (
              <fieldset className="meshy-lab__preflight">
                <legend>H1 rigging preflight</legend>
                <label><input type="checkbox" checked={h1Preflight.standardHumanoid} onChange={(event) => setH1Preflight((current) => ({ ...current, standardHumanoid: event.target.checked }))} />Standard bipedal humanoid</label>
                <label><input type="checkbox" checked={h1Preflight.clearLimbs} onChange={(event) => setH1Preflight((current) => ({ ...current, clearLimbs: event.target.checked }))} />Clear limbs and body structure</label>
                <label><input type="checkbox" checked={h1Preflight.noWeapon} onChange={(event) => setH1Preflight((current) => ({ ...current, noWeapon: event.target.checked }))} />No weapon in the source intent</label>
              </fieldset>
            ) : null}
            <p>Pipeline: {profile.stages.join(" -> ")}</p>
            <button type="button" className="button button--primary" onClick={review} disabled={busy || !prompt.trim() || (profile.id.startsWith("H1") && !(h1Preflight.standardHumanoid && h1Preflight.clearLimbs && h1Preflight.noWeapon))}>
              Review generation
            </button>
          </main>
        </div>
      ) : null}

      {screen === "REVIEW" && preview ? (
        <div className="meshy-lab__review panel">
          <p className="eyebrow">Explicit paid operation</p>
          <h2>Review generation</h2>
          <dl>
            <div><dt>Profile</dt><dd>{profileCode(preview.profile)} {preview.profile.label}</dd></div>
            <div><dt>Output</dt><dd>GLB</dd></div>
            <div><dt>Pipeline</dt><dd>{preview.stages.join(" -> ")}</dd></div>
            <div><dt>Maximum cost</dt><dd>{preview.maximumCredits} credits maximum</dd></div>
          </dl>
          <p>Your Meshy account is charged only after confirmation. No task exists yet.</p>
          <div className="meshy-lab__actions">
            <button type="button" className="button button--secondary" onClick={() => setScreen("CONFIGURE")} disabled={busy}>Back to configuration</button>
            <button type="button" className="button button--primary" onClick={generate} disabled={busy}>
              Generate {profileCode(preview.profile)} asset
            </button>
          </div>
        </div>
      ) : null}

      {screen === "RUN" && run ? (
        <div className="meshy-lab__run panel">
          <p className="eyebrow">Meshy run</p>
          <h2>{run.status === "QUEUED" ? "Generation queued" : `Run ${run.status.toLowerCase()}`}</h2>
          <p>{profileCode(run.profile)} · {run.progress}% · {run.profile.stages.join(" -> ")}</p>
          <p>Task IDs remain in the local proof provenance and are never used as credentials.</p>
          <div className="meshy-lab__actions">
            <button type="button" className="button button--secondary" onClick={refresh} disabled={busy}>Refresh status</button>
            {run.status !== "READY" && run.status !== "CANCELED" && run.status !== "FAILED" ? (
              <button type="button" className="button button--quiet" onClick={cancel} disabled={busy}>Cancel run</button>
            ) : null}
            {run.status === "READY" ? (
              <>
                <button type="button" className="button button--secondary" onClick={downloadProvenance} disabled={busy}>Download provenance</button>
                <button type="button" className="button button--primary" onClick={importArtifact} disabled={busy}>Import verified GLB to Source</button>
              </>
            ) : null}
          </div>
          <details className="meshy-lab__technical">
            <summary>Technical details</summary>
            <p>Run {run.id}</p>
            <p>{Object.entries(run.taskIds).map(([stage, taskId]) => `${stage}: ${taskId}`).join(" · ") || "No Meshy task IDs yet."}</p>
          </details>
        </div>
      ) : null}
    </section>
  );
}
