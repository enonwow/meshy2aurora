import { useEffect, useState } from "react";
import {
  type MeshyArtifactProvenance,
  type MeshyAnimationActionMapping,
  type MeshyBridgeClient,
  type MeshyGenerationSource,
  type MeshyHistoryItem,
  type MeshyHistoryPage,
  type MeshyImageAiModel,
  type MeshyImageRun,
  type MeshyImageRunMode,
  type MeshyImageRunPreview,
  type MeshyProfile,
  type MeshyRetexturePreview,
  type MeshyRetextureRun,
  type MeshyRun,
  type MeshyRunArtifact,
  type MeshyRunPreview,
  type MeshyTextTo3DOptions,
  DEFAULT_MESHY_TEXT_TO_3D_OPTIONS,
  AURORA_MODEL_TRIANGLE_BUDGET_V1,
  MESHY_BRIDGE_CAPABILITIES_V2,
  MESHY_BRIDGE_PROTOCOL_VERSION,
  NWN_DIRECT_CREATURE_CLIPS,
  validateMeshyAnimationActions,
} from "./bridge";
import { MeshyModelViewport } from "./MeshyModelViewport";

type LabScreen = "CONNECT" | "CONFIGURE" | "REVIEW" | "RUN" | "HISTORY" | "IMAGE_CONFIGURE" | "IMAGE_REVIEW" | "IMAGE_RUN";
type HistoryFilter = "READY" | "ALL";
const ACTIVE_MESHY_RUN_STORAGE_KEY = "m2a.meshy.activeRun.v2";

function bridgeCapabilitiesMatch(
  capabilities: unknown,
): capabilities is typeof MESHY_BRIDGE_CAPABILITIES_V2 {
  return JSON.stringify(capabilities) === JSON.stringify(MESHY_BRIDGE_CAPABILITIES_V2);
}

function rememberedRunId() {
  try { return window.sessionStorage.getItem(ACTIVE_MESHY_RUN_STORAGE_KEY) ?? undefined; }
  catch { return undefined; }
}

function rememberRun(run: MeshyRun) {
  try { window.sessionStorage.setItem(ACTIVE_MESHY_RUN_STORAGE_KEY, run.id); }
  catch { /* Recovery is best-effort when browser storage is disabled. */ }
}

export interface MeshyLabProps {
  readonly bridge: MeshyBridgeClient;
  readonly onBack: () => void;
  readonly onImport: (file: File, provenance: MeshyArtifactProvenance) => void;
}

function canRecoverHistoryItem(item: MeshyHistoryItem) {
  return item.stage === "REFINE" && item.status === "SUCCEEDED" && item.glbAvailable;
}

function errorMessage(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  return message === "The local Bridge pairing code is invalid."
    ? "Pairing code is invalid. It is not your Meshy API key. Get the newest code from the local Bridge log below."
    : message;
}

function profileCode(profile: MeshyProfile) {
  return profile.id.slice(0, 2);
}

function readDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new Error(`Could not read ${file.name}.`));
    reader.onload = () => resolve(String(reader.result));
    reader.readAsDataURL(file);
  });
}

function MeshyWorkspaceIcon({ tool }: { readonly tool: "model" | "image" | "assets" | "current" }) {
  const common = { fill: "none", stroke: "currentColor", strokeWidth: 1.7, strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
  if (tool === "model") return <svg viewBox="0 0 24 24" aria-hidden="true" {...common}><path d="m12 3 7 4v8l-7 4-7-4V7l7-4Z" /><path d="m5 7 7 4 7-4M12 11v8" /><path d="m8.5 5 7 4" /></svg>;
  if (tool === "image") return <svg viewBox="0 0 24 24" aria-hidden="true" {...common}><rect x="3.5" y="4.5" width="17" height="15" rx="2" /><circle cx="8.5" cy="9" r="1.5" /><path d="m4 17 5-5 3.5 3 2.5-2.5 5 4.5" /></svg>;
  if (tool === "assets") return <svg viewBox="0 0 24 24" aria-hidden="true" {...common}><rect x="4" y="4" width="6" height="6" rx="1" /><rect x="14" y="4" width="6" height="6" rx="1" /><rect x="4" y="14" width="6" height="6" rx="1" /><rect x="14" y="14" width="6" height="6" rx="1" /></svg>;
  return <svg viewBox="0 0 24 24" aria-hidden="true" {...common}><path d="M12 3v4M12 17v4M3 12h4M17 12h4M5.6 5.6l2.8 2.8M15.6 15.6l2.8 2.8M18.4 5.6l-2.8 2.8M8.4 15.6l-2.8 2.8" /><circle cx="12" cy="12" r="3" /></svg>;
}

function MeshySourceIcon({ source }: { readonly source: MeshyGenerationSource }) {
  const common = { fill: "none", stroke: "currentColor", strokeWidth: 1.55, strokeLinecap: "round" as const, strokeLinejoin: "round" as const };
  if (source === "TEXT") return <svg viewBox="0 0 32 32" aria-hidden="true" {...common}><path d="M7 6.5h18v19H7z" /><path d="M11 12h10M11 16h7M11 20h9" /><path d="m21 21 3.5 3.5" /></svg>;
  if (source === "IMAGE") return <svg viewBox="0 0 32 32" aria-hidden="true" {...common}><rect x="5" y="7" width="22" height="18" rx="3" /><circle cx="11.5" cy="13" r="1.7" /><path d="m6 22 6.5-6 4.5 4 3-3 6 5" /></svg>;
  return <svg viewBox="0 0 32 32" aria-hidden="true" {...common}><rect x="4" y="7" width="15" height="17" rx="2.5" /><rect x="13" y="10" width="15" height="17" rx="2.5" /><path d="m6 21 4-4 3 2.7 3-3M15 24l4-4 3 2.7 3-3" /></svg>;
}

function MeshyBrandMark() {
  return <svg className="meshy-lab__brand-mark" viewBox="0 0 64 64" role="img" aria-label="Meshy"><path fill="#C5F955" d="M14.297 22.425a26.493 26.493 0 0 1 2.36-4.345l.015-.024c.102-.155.206-.308.31-.459 6.16-9.554 19.384-15.28 30.566-10.208 8.837 3.783 13.072 11.507 12.95 18.383-.061 3.43-1.22 6.775-3.613 9.193-.41.414-.849.793-1.315 1.136 2.16 1.715 3.572 3.8 3.939 6.236.549 3.647-1.418 6.905-4.187 9.31-5.54 4.81-15.923 7.824-27.75 6.567-1.964-.178-9.062-1.574-10.678-4.168a10.753 10.753 0 0 1-2.31-1.475c-1.105-.942-2.018-2.198-2.354-3.783-.49-2.305.776-6.223 3.405-7.343.945 4.643 4.317 4.925 6.82 5.186 1.025-.86 1.519-1.348 2.55-2.64-2.002.162-4.544-.384-6.495-1.426-3.58-1.91-5.735-5.42-5.97-10.435-.233-3.284.473-6.693 1.757-9.705Zm5.029-3.275a25.832 25.832 0 0 0-.324.48 23.685 23.685 0 0 0-2.115 3.893l.007.061c-1.164 2.786-1.708 5.629-1.558 8.184l.015.226V32v-.006.007l.01.185c.412 7.067 5.235 9.33 11.163 9.259.96-.012 1.949-.084 2.953-.208-.259 1.113-.549 2.077-.916 2.916-.527 1.203-1.213 2.152-2.194 2.92 3.534.266 7.36-.255 10.783-1.303 4.73-.346 12.071-3.058 10.58-8.286h.001a5.832 5.832 0 0 0-.08-.26c-.16-.593-.352-1.17-.548-1.716l-.015-.041.017.017a11.57 11.57 0 0 0 3.16-.105 9.764 9.764 0 0 0 .953-.223 8.7 8.7 0 0 0 1.53-.603l.015-.008.014-.007.226-.12C60.58 30.22 59.365 15.48 46.41 9.955c-9.658-4.4-21.513.516-27.084 9.194Zm31.32 17.22c5.762 6.672.077 12.731-8.901 16.061 3.338-2.156 6.079-5.534 7.73-10.43-9.99 9.45-32.909 10.428-33.977 4.087a7.795 7.795 0 0 1-.031-.691v-.005c-2.299 4.518 2.792 7.315 9.764 7.72l.09.005a31.994 31.994 0 0 0 2.735.03l.037-.002a34.57 34.57 0 0 0 3.39-.28c4.113-.544 8.318-1.845 11.742-4.009-1.923 1.672-3.803 2.891-5.718 3.785-3.014 1.406-6.114 2.007-9.606 2.3h-.009c-.803.068-1.627.12-2.476.16.809.14 1.62.247 2.433.32 1.283.137 2.542.22 3.775.255 2.323.065 4.55-.045 6.651-.3 14.888-1.81 23.467-10.938 15.43-17.17a13.73 13.73 0 0 0-.879-.627 18.31 18.31 0 0 0-2.177-1.212l-.008-.004.005.006Z" /><path fill="#FF3E8F" d="M19.023 19.599c-6.643 10.064-4.875 23.377 9.438 21.018l14.19-4.671c13.29-4.423 13.925-17.975 5.54-23.949C41.838 7.47 27.114 7.34 19.022 19.6Z" /><path fill="#C5F955" d="M26.438 27.52c6.303-1.706 12.976-8.954 14.002-18.245-2.183-.202-4.61-.054-7.265.532-7.993 1.763-13.675 7.582-16.29 13.716.531 4.753 7.898 4.445 9.553 3.996Z" /><path fill="#67B800" d="M26.439 27.519c6.303-1.705 12.203-8.686 13.229-17.977l-1.81.433c-2.92 10.498-11.15 17.866-18.539 15.814 1.092 2.023 6.267 1.961 7.12 1.73Z" /><path fill="#E9FFCE" d="M37.926 11.483c-8.34-1.67-18.172 3.596-20.505 12.68 3.39-6.505 12.744-12.37 20.505-12.68Z" /><path fill="#C91C65" d="m54.737 19.83-12.78 17.564-13.023 2.75c-.139.1-.264.057-.267.056l.267-.056c.212-.154.456-.642.356-2.123-.27-.766-.452-2.574-2.289-3.804-.882-.59-1.717-1.376-1.57-2.427.144-1.037 1.08-1.652 2.098-1.897 8.8-2.115 12.868-11.126 13.772-19.64l7.708 3.337 5.728 6.24Z" /><path fill="#181818" d="M46.41 9.956a18.155 18.155 0 0 0-4.917-1.446v-.001l-.017-.001c-8.752-1.262-17.682 3.372-22.388 11.02 4.665-6.264 13.163-9.89 20.197-8.912-.595 3.543-2.885 8.819-7.034 12.553-4.117 3.706-10.358 5.104-14.183 2.821 2.82 3.352 11.388 2.923 16.044-1.1 4.365-3.77 7.07-9.665 7.568-13.767.204.05.405.102.604.158 2.122.673 2.712 2.5 2.855 3.777.095.844-.428 1.58-1.05 2.162-1.316 1.23-2.628 3.213-3.39 5.347-.859 2.408-2.005 4.897-4.09 6.382-.934.665-1.674 1.236-2.244 1.735-1.988 1.31-3.518 3.469-4.112 6.552-.085.444-.134.704-.203 1.09-6.92 1.376-13.3.304-14.7-6.332.39 8.376 6.868 10.135 14.126 9.243-.63 2.708-1.444 4.532-3.11 5.835 1.262.095 2.56.09 3.863-.003 1.133-1.404 1.523-3.885 1.856-6.005.076-.487.15-.955.228-1.387a14.048 14.048 0 0 1 .211-.983c2.171-8.53 11.279-7.025 11.72-1.12.01.136.022.269.033.398.199 2.37.3 3.573-7.154 7.8 4.806-.343 12.342-3.14 10.527-8.547-.164-.608-.361-1.2-.562-1.758l.017.017c13.467 1.324 14.976-18.845-.695-25.528Zm3.745 6.42c-6.156 2.357-8.769 12.35-3.444 14.64l2.01-1.827c-2.968-3.153-2.313-9.756 1.44-12.804l-.006-.01Z" /></svg>;
}

function MeshyLibraryCard({ bridge, sessionToken, item, onOpen }: {
  readonly bridge: MeshyBridgeClient;
  readonly sessionToken: string;
  readonly item: MeshyHistoryItem;
  readonly onOpen: (item: MeshyHistoryItem) => void;
}) {
  const [thumbnailUrl, setThumbnailUrl] = useState<string>();
  useEffect(() => {
    if (!item.thumbnailAvailable) return;
    let active = true;
    let objectUrl: string | undefined;
    void bridge.downloadHistoryThumbnail(sessionToken, item.taskId).then(({ file }) => {
      objectUrl = URL.createObjectURL(file);
      if (active) setThumbnailUrl(objectUrl);
    }).catch(() => undefined);
    return () => { active = false; if (objectUrl) URL.revokeObjectURL(objectUrl); };
  }, [bridge, item.taskId, item.thumbnailAvailable, sessionToken]);

  return <button type="button" className="meshy-lab__library-card" onClick={() => onOpen(item)} aria-label={`Open ${item.prompt || "Meshy model"} in the Meshy viewport`}>
    {thumbnailUrl ? <img src={thumbnailUrl} alt="" /> : <span className="meshy-lab__library-placeholder" aria-hidden="true">◈</span>}
    <strong>{item.stage === "REFINE" ? "Verified model" : "Meshy task"}</strong>
    <span>{item.prompt || "No prompt returned"}</span>
    <small>{item.status}</small>
  </button>;
}

function MeshyLibraryPanel({ bridge, sessionToken, items, query, busy, page, hasNext, newestFirst, retextureTaskId, onQueryChange, onToggleSort, onBrowse, onPreviousPage, onNextPage, onOpen, onRetexture }: {
  readonly bridge: MeshyBridgeClient;
  readonly sessionToken?: string;
  readonly items: readonly MeshyHistoryItem[];
  readonly query: string;
  readonly busy: boolean;
  readonly page: number;
  readonly hasNext: boolean;
  readonly newestFirst: boolean;
  readonly retextureTaskId?: string;
  readonly onQueryChange: (query: string) => void;
  readonly onToggleSort: () => void;
  readonly onBrowse: () => void;
  readonly onPreviousPage: () => void;
  readonly onNextPage: () => void;
  readonly onOpen: (item: MeshyHistoryItem) => void;
  readonly onRetexture?: (inputTaskId: string) => void;
}) {
  const [showModelActions, setShowModelActions] = useState(false);
  return <aside className="panel meshy-lab__creator-library">
    <div className="meshy-lab__library-search"><input type="search" aria-label="Search generated models" value={query} onChange={(event) => onQueryChange(event.target.value)} placeholder="Search my generations" /><button type="button" className="button button--quiet" onClick={onBrowse} disabled={busy}>↗</button></div>
    <div className="meshy-lab__library-toolbar"><span aria-hidden="true">▦</span><span>My models</span><small>{items.length}</small><button type="button" aria-label={newestFirst ? "Sort models oldest first" : "Sort models newest first"} aria-pressed={newestFirst} onClick={onToggleSort}>⇅</button></div>
    <div className="meshy-lab__library-grid">{sessionToken ? items.map((item) => <MeshyLibraryCard key={item.taskId} bridge={bridge} sessionToken={sessionToken} item={item} onOpen={onOpen} />) : null}</div>
    <footer className="meshy-lab__library-pager"><button type="button" aria-label="Previous model page" onClick={onPreviousPage} disabled={busy || page <= 1}>‹</button><span>{page}/{hasNext ? "…" : page}</span><button type="button" aria-label="Next model page" onClick={onNextPage} disabled={busy || !hasNext}>›</button></footer>
    {retextureTaskId && onRetexture ? <><button type="button" className="meshy-lab__library-model-menu" aria-label="Open current model actions" aria-expanded={showModelActions} onClick={() => setShowModelActions((current) => !current)}>☰</button>{showModelActions ? <div className="meshy-lab__library-model-actions"><button type="button" onClick={() => onRetexture(retextureTaskId)} disabled={busy}>ReTexture current model</button></div> : null}</> : null}
    {!items.length ? <p className="meshy-lab__hint">No matching Meshy models.</p> : null}
  </aside>;
}

export function MeshyLab({ bridge, onBack, onImport }: MeshyLabProps) {
  const [screen, setScreen] = useState<LabScreen>("CONNECT");
  const [pairingCode, setPairingCode] = useState("");
  const [sessionToken, setSessionToken] = useState<string>();
  const [availableCredits, setAvailableCredits] = useState<number>();
  const [restartSupported, setRestartSupported] = useState(false);
  const [automaticPairingSupported, setAutomaticPairingSupported] = useState(false);
  const [restartNotice, setRestartNotice] = useState<string>();
  const [prompt, setPrompt] = useState("");
  const [generationSource, setGenerationSource] = useState<MeshyGenerationSource>("TEXT");
  const [referenceImages, setReferenceImages] = useState<File[]>([]);
  const [imageInputTaskId, setImageInputTaskId] = useState<string>();
  const [apiOptions, setApiOptions] = useState<MeshyTextTo3DOptions>(DEFAULT_MESHY_TEXT_TO_3D_OPTIONS);
  const [preview, setPreview] = useState<MeshyRunPreview>();
  const [run, setRun] = useState<MeshyRun>();
  const [imageMode, setImageMode] = useState<MeshyImageRunMode>("TEXT_TO_IMAGE");
  const [imagePrompt, setImagePrompt] = useState("");
  const [imageModel, setImageModel] = useState<MeshyImageAiModel>("nano-banana");
  const [imageMultiView, setImageMultiView] = useState(false);
  const [imagePose, setImagePose] = useState<"a-pose" | "t-pose" | undefined>();
  const [imageAspectRatio, setImageAspectRatio] = useState<"1:1" | "16:9" | "9:16" | "4:3" | "3:4" | "3:2" | "2:3">("1:1");
  const [imageReferenceFiles, setImageReferenceFiles] = useState<File[]>([]);
  const [imagePreview, setImagePreview] = useState<MeshyImageRunPreview>();
  const [imageRun, setImageRun] = useState<MeshyImageRun>();
  const [history, setHistory] = useState<MeshyHistoryPage>();
  const [historyFilter, setHistoryFilter] = useState<HistoryFilter>("READY");
  const [selectedHistoryTaskId, setSelectedHistoryTaskId] = useState<string>();
  const [libraryQuery, setLibraryQuery] = useState("");
  const [libraryNewestFirst, setLibraryNewestFirst] = useState(true);
  const [viewerArtifact, setViewerArtifact] = useState<MeshyRunArtifact>();
  const [viewerLabel, setViewerLabel] = useState("");
  const [retextureInputTaskId, setRetextureInputTaskId] = useState<string>();
  const [retextureStylePrompt, setRetextureStylePrompt] = useState("");
  const [retexturePreview, setRetexturePreview] = useState<MeshyRetexturePreview>();
  const [retextureRun, setRetextureRun] = useState<MeshyRetextureRun>();
  const [workspaceNavigationCollapsed, setWorkspaceNavigationCollapsed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();

  const perform = async (operation: () => Promise<void>) => {
    setBusy(true);
    setError(undefined);
    try { await operation(); } catch (cause) { setError(errorMessage(cause)); } finally { setBusy(false); }
  };

  useEffect(() => {
    let active = true;
    void bridge.health().then((health) => { if (active) { setRestartSupported(health.restartSupported); setAutomaticPairingSupported(health.automaticPairingSupported); } }).catch(() => undefined);
    return () => { active = false; };
  }, [bridge]);

  const connect = () => void perform(async () => {
    const health = await bridge.health();
    if (
      health.protocolVersion !== MESHY_BRIDGE_PROTOCOL_VERSION
      || health.status !== "READY"
      || !bridgeCapabilitiesMatch(health.capabilities)
    ) {
      throw new Error("The local Meshy Bridge is stale or does not support the required multi-animation and recovery contract. Restart it before creating a paid run.");
    }
    setRestartSupported(health.restartSupported);
    setAutomaticPairingSupported(health.automaticPairingSupported);
    const pairing = health.automaticPairingSupported ? await bridge.pairAutomatically() : await bridge.pair({ pairingCode });
    const [balance, profiles, recentHistory, localRuns] = await Promise.all([
      bridge.balance(pairing.sessionToken),
      bridge.profiles(pairing.sessionToken),
      bridge.listHistory(pairing.sessionToken),
      bridge.listRuns(pairing.sessionToken),
    ]);
    if (!profiles.length) throw new Error("The local Bridge did not expose a supported generation profile.");
    setSessionToken(pairing.sessionToken);
    setAvailableCredits(balance.availableCredits);
    setHistory(recentHistory);
    const remembered = rememberedRunId();
    const recoveredRun = localRuns.find((candidate) => candidate.id === remembered)
      ?? localRuns[0];
    if (recoveredRun) {
      setRun(recoveredRun);
      rememberRun(recoveredRun);
      setScreen("RUN");
    } else {
      setScreen("CONFIGURE");
    }
  });

  const restartLocalBridge = () => void perform(async () => {
    await bridge.restart();
    setSessionToken(undefined);
    setAvailableCredits(undefined);
    setHistory(undefined);
    setPairingCode("");
    setRestartNotice("Bridge is restarting. When it is ready, enter the fresh pairing code from your local terminal.");
  });

  const patchApiOptions = (patch: Partial<MeshyTextTo3DOptions>) => setApiOptions((current) => ({ ...current, ...patch }));
  const patchAnimationAction = (index: number, patch: Partial<MeshyAnimationActionMapping>) => {
    patchApiOptions({
      animationActions: apiOptions.animationActions.map((action, actionIndex) => (
        actionIndex === index ? { ...action, ...patch } : action
      )),
    });
  };
  const addAnimationAction = () => {
    if (apiOptions.animationActions.length >= 10) return;
    const usedActionIds = new Set(apiOptions.animationActions.map(({ actionId }) => actionId));
    const usedClipNames = new Set(apiOptions.animationActions.map(({ clipName }) => clipName));
    let actionId = 0;
    while (usedActionIds.has(actionId)) actionId += 1;
    const preferredClipOrder: readonly MeshyAnimationActionMapping["clipName"][] = [
      "ca1slashl", "ca1slashr", "ca1stab", "cwalk", "crun", "ckdbckdie",
      ...NWN_DIRECT_CREATURE_CLIPS,
    ];
    const clipName = preferredClipOrder.find((candidate) => !usedClipNames.has(candidate));
    if (!clipName) return;
    patchApiOptions({ animationActions: [...apiOptions.animationActions, { actionId, clipName }] });
  };
  const removeAnimationAction = (index: number) => {
    if (apiOptions.animationActions.length <= 1) return;
    patchApiOptions({ animationActions: apiOptions.animationActions.filter((_, actionIndex) => actionIndex !== index) });
  };
  const chooseSource = (source: MeshyGenerationSource) => {
    setGenerationSource(source);
    setImageInputTaskId(undefined);
    if (source === "TEXT") setReferenceImages([]);
    if (source === "MULTI_IMAGE") patchApiOptions({ modelType: "standard", aiModel: "meshy-6" });
    if (source === "TEXT" && apiOptions.modelType === "smart-topology") patchApiOptions({ modelType: "standard", aiModel: "meshy-6" });
  };
  const chooseModelType = (modelType: MeshyTextTo3DOptions["modelType"]) => {
    patchApiOptions({
      modelType,
      aiModel: modelType === "smart-topology" ? "meshy-t2" : "meshy-6",
      targetPolycount: modelType === "smart-topology" ? 4_000 : apiOptions.targetPolycount,
    });
  };
  const chooseReferenceImages = (files: FileList | readonly File[]) => {
    setReferenceImages(Array.from(files).slice(0, generationSource === "MULTI_IMAGE" ? 4 : 1));
  };
  const chooseConceptReferenceImages = (files: FileList | readonly File[]) => {
    setImageReferenceFiles(Array.from(files).slice(0, 5));
  };

  const review = () => void perform(async () => {
    if (!sessionToken) return;
    if (generationSource === "TEXT" && !prompt.trim()) throw new Error("Text to 3D needs a prompt.");
    if (generationSource === "IMAGE" && !imageInputTaskId && referenceImages.length !== 1) throw new Error("Image to 3D needs exactly one reference image.");
    if (generationSource === "MULTI_IMAGE" && !imageInputTaskId && (referenceImages.length < 1 || referenceImages.length > 4)) throw new Error("Multi-image to 3D needs one to four reference images.");
    if (apiOptions.rigHumanoid) {
      const errors = validateMeshyAnimationActions(apiOptions.animationActions);
      if (errors.length) throw new Error(errors.join(" "));
    }
    const imageDataUrls = generationSource === "TEXT" || imageInputTaskId ? undefined : await Promise.all(referenceImages.map(readDataUrl));
    const nextPreview = await bridge.previewRun(sessionToken, {
      profileId: apiOptions.rigHumanoid ? "H1-humanoid-animated/v1" : "S1-static-prop/v1",
      prompt,
      geometryTarget: "BALANCED",
      source: generationSource,
      imageDataUrls,
      ...(imageInputTaskId ? { inputTaskId: imageInputTaskId } : {}),
      apiOptions,
      ...(apiOptions.rigHumanoid ? { h1Preflight: { standardHumanoid: true as const, clearLimbs: true as const, noWeapon: true as const } } : {}),
    });
    setPreview(nextPreview);
    setScreen("REVIEW");
  });

  const generate = () => void perform(async () => {
    if (!sessionToken || !preview) return;
    const created = await bridge.createRun(sessionToken, { previewId: preview.previewId, confirmationNonce: crypto.randomUUID() });
    setRun(created);
    rememberRun(created);
    setScreen("RUN");
  });
  const refresh = () => void perform(async () => {
    if (sessionToken && run) {
      const refreshed = await bridge.getRun(sessionToken, run.id);
      setRun(refreshed);
      rememberRun(refreshed);
    }
  });
  const cancel = () => void perform(async () => {
    if (sessionToken && run) {
      const canceled = await bridge.cancelRun(sessionToken, run.id);
      setRun(canceled);
      rememberRun(canceled);
    }
  });
  const importArtifact = () => void perform(async () => { if (sessionToken && run) { const artifact = await bridge.downloadArtifact(sessionToken, run.id); onImport(artifact.file, artifact.provenance); } });
  const downloadProvenance = () => void perform(async () => {
    if (!sessionToken || !run) return;
    const provenance = await bridge.provenance(sessionToken, run.id);
    const url = URL.createObjectURL(new Blob([JSON.stringify(provenance, null, 2)], { type: "application/json" }));
    const anchor = document.createElement("a");
    anchor.href = url; anchor.download = `meshy-${run.id}-provenance.json`; anchor.click(); URL.revokeObjectURL(url);
  });
  const browseHistory = (pageNum = 1, resetFilter = false, openHistory = true) => void perform(async () => {
    if (!sessionToken) return;
    setHistory(await bridge.listHistory(sessionToken, pageNum));
    setSelectedHistoryTaskId(undefined);
    if (resetFilter) setHistoryFilter("READY");
    if (openHistory) setScreen("HISTORY");
  });
  const recoverHistoryArtifact = (taskId: string) => void perform(async () => {
    if (!sessionToken) return;
    const artifact = await bridge.downloadHistoryArtifact(sessionToken, taskId);
    onImport(artifact.file, artifact.provenance);
  });
  const openHistoryViewer = (item: MeshyHistoryItem) => void perform(async () => {
    if (!sessionToken) return;
    if (!canRecoverHistoryItem(item)) {
      setSelectedHistoryTaskId(item.taskId);
      setHistoryFilter("ALL");
      setScreen("HISTORY");
      return;
    }
    const artifact = await bridge.downloadHistoryArtifact(sessionToken, item.taskId);
    setViewerArtifact(artifact);
    setViewerLabel(item.prompt || "Meshy model");
    setScreen("CONFIGURE");
  });
  const openCurrentViewer = () => void perform(async () => {
    if (!sessionToken || !run || run.status !== "READY") return;
    const artifact = await bridge.downloadArtifact(sessionToken, run.id);
    setViewerArtifact(artifact);
    setViewerLabel(run.prompt || "Meshy model");
    setScreen("CONFIGURE");
  });
  const openRetexture = (inputTaskId: string) => {
    setRetextureInputTaskId(inputTaskId);
    setRetextureStylePrompt("");
    setRetexturePreview(undefined);
    setRetextureRun(undefined);
  };
  const previewRetexture = () => void perform(async () => {
    if (!sessionToken || !retextureInputTaskId || !retextureStylePrompt.trim()) return;
    setRetexturePreview(await bridge.previewRetexture(sessionToken, {
      inputTaskId: retextureInputTaskId,
      textStylePrompt: retextureStylePrompt,
      aiModel: "meshy-6",
      enableOriginalUv: true,
      enablePbr: true,
      hdTexture: false,
      removeLighting: true,
      alphaThumbnail: false,
    }));
  });
  const createRetexture = () => void perform(async () => {
    if (!sessionToken || !retexturePreview) return;
    setRetextureRun(await bridge.createRetexture(sessionToken, { previewId: retexturePreview.previewId, confirmationNonce: crypto.randomUUID() }));
  });
  const refreshRetexture = () => void perform(async () => {
    if (!sessionToken || !retextureRun) return;
    const next = await bridge.getRetexture(sessionToken, retextureRun.id);
    setRetextureRun(next);
    if (next.status === "READY") {
      const artifact = await bridge.downloadRetextureArtifact(sessionToken, next.id);
      setViewerArtifact(artifact);
      setViewerLabel(`${viewerLabel || "Meshy model"} — ReTexture`);
      setRetextureInputTaskId(undefined);
    }
  });
  const reviewImage = () => void perform(async () => {
    if (!sessionToken) return;
    if (!imagePrompt.trim()) throw new Error("Image generation needs a prompt.");
    if (imageMode === "IMAGE_TO_IMAGE" && (imageReferenceFiles.length < 1 || imageReferenceFiles.length > 5)) throw new Error("Image to Image needs one to five reference images.");
    const referenceImageDataUrls = imageMode === "IMAGE_TO_IMAGE" ? await Promise.all(imageReferenceFiles.map(readDataUrl)) : undefined;
    const nextPreview = await bridge.previewImageRun(sessionToken, {
      mode: imageMode,
      prompt: imagePrompt,
      aiModel: imageModel,
      generateMultiView: imageMultiView,
      ...(imageMultiView ? {} : { aspectRatio: imageAspectRatio }),
      ...(imageMode === "TEXT_TO_IMAGE" && imagePose ? { poseMode: imagePose } : {}),
      ...(imageMode === "IMAGE_TO_IMAGE" ? { referenceImageDataUrls } : {}),
    });
    setImagePreview(nextPreview);
    setScreen("IMAGE_REVIEW");
  });
  const generateImage = () => void perform(async () => {
    if (!sessionToken || !imagePreview) return;
    setImageRun(await bridge.createImageRun(sessionToken, { previewId: imagePreview.previewId, confirmationNonce: crypto.randomUUID() }));
    setScreen("IMAGE_RUN");
  });
  const refreshImage = () => void perform(async () => { if (sessionToken && imageRun) setImageRun(await bridge.getImageRun(sessionToken, imageRun.id)); });
  const cancelImage = () => void perform(async () => { if (sessionToken && imageRun) setImageRun(await bridge.cancelImageRun(sessionToken, imageRun.id)); });
  const useImageFor3d = () => {
    if (!imageRun?.taskId || imageRun.status !== "READY") return;
    setGenerationSource(imageRun.generateMultiView ? "MULTI_IMAGE" : "IMAGE");
    setReferenceImages([]);
    setImageInputTaskId(imageRun.taskId);
    setPrompt(imageRun.prompt);
    setScreen("CONFIGURE");
  };

  const readyHistoryItems = history?.items.filter(canRecoverHistoryItem) ?? [];
  const visibleHistoryItems = historyFilter === "READY" ? readyHistoryItems : history?.items ?? [];
  const libraryItems = (history?.items ?? []).filter(canRecoverHistoryItem).filter((item) => item.prompt.toLocaleLowerCase().includes(libraryQuery.trim().toLocaleLowerCase())).sort((left, right) => { const difference = Date.parse(right.createdAt ?? "") - Date.parse(left.createdAt ?? ""); return libraryNewestFirst ? difference : -difference; }).slice(0, 20);
  const selectedHistoryItem = history?.items.find((item) => item.taskId === selectedHistoryTaskId);
  const animationErrors = apiOptions.rigHumanoid ? validateMeshyAnimationActions(apiOptions.animationActions) : [];
  const hasGenerationInput = generationSource === "TEXT" ? Boolean(prompt.trim()) : Boolean(imageInputTaskId || referenceImages.length > 0);
  const canReview = hasGenerationInput && animationErrors.length === 0;

  return <section className="meshy-lab" aria-labelledby="meshy-lab-heading">
    <header className="meshy-lab__header">
      <button type="button" className="meshy-lab__menu-toggle" aria-label="Toggle workspace navigation" aria-expanded={!workspaceNavigationCollapsed} onClick={() => setWorkspaceNavigationCollapsed((current) => !current)}><span /><span /><span /></button>
      <div className="meshy-lab__brand"><MeshyBrandMark /><h1 id="meshy-lab-heading">Meshy</h1></div>
      <div className="meshy-lab__header-navigation" aria-label="Workspace modes" aria-hidden="true" />
      <div className="meshy-lab__header-actions">{availableCredits !== undefined ? <span className="meshy-lab__credits">◉ {availableCredits}</span> : null}<span className="status-badge status-badge--neutral">{sessionToken ? "Paired local bridge" : "Local bridge"}</span><button type="button" className="button button--quiet" onClick={onBack}>Exit workspace</button></div>
    </header>
    {error ? <p className="meshy-lab__error" role="alert">{error}</p> : null}
    {screen === "CONNECT" ? <div className="meshy-lab__connect"><p className="meshy-lab__connect-kicker">Local Bridge</p><h2>Connect your workspace</h2><p>Credentials remain in the local Bridge. This Studio receives only a temporary session.</p>{automaticPairingSupported ? <><p className="meshy-lab__automatic-pairing">This local Compose Bridge can pair this exact Studio origin without exposing a pairing code or API key.</p><button type="button" className="button button--primary" onClick={connect} disabled={busy}>Connect local bridge</button><small>Local session only. It expires automatically and cannot create a paid task without later confirmation.</small></> : <><label htmlFor="meshy-pairing-code">Pairing code</label><input id="meshy-pairing-code" value={pairingCode} onChange={(event) => setPairingCode(event.target.value)} autoComplete="off" placeholder="From local Bridge terminal — not your API key" /><button type="button" className="button button--primary" onClick={connect} disabled={busy || !pairingCode.trim()}>Connect local bridge</button><small>Pairing code is different from your Meshy API key and changes after every Bridge restart.</small><code className="meshy-lab__pairing-command">docker compose --profile meshy logs --tail 1 meshy-bridge</code></>}{restartSupported ? <div className="meshy-lab__restart"><p>Pairing code stale or unavailable?</p><button type="button" className="button button--quiet" onClick={restartLocalBridge} disabled={busy}>Restart local Bridge</button><small>Restarts only the local Bridge; it does not cancel an already-running Meshy task.</small></div> : null}{restartNotice ? <p className="meshy-lab__restart-notice" role="status">{restartNotice}</p> : null}</div> : null}
    {screen !== "CONNECT" ? <div className={`meshy-lab__layout${workspaceNavigationCollapsed ? " meshy-lab__layout--navigation-collapsed" : ""}`}>
      <nav className="meshy-lab__navigation panel" aria-label="Meshy Lab navigation">
        <p className="eyebrow">Workspace</p>
        <button type="button" aria-label="New model" data-workspace-tool="model" className={`meshy-lab__navigation-item${screen === "CONFIGURE" ? " meshy-lab__navigation-item--active" : ""}`} onClick={() => setScreen("CONFIGURE")} aria-current={screen === "CONFIGURE" ? "page" : undefined}><strong><i className="meshy-lab__tool-icon"><MeshyWorkspaceIcon tool="model" /></i>Model</strong><span>Create with every supported control</span></button>
        <button type="button" aria-label="Concept images" data-workspace-tool="image" className={`meshy-lab__navigation-item${screen.startsWith("IMAGE_") ? " meshy-lab__navigation-item--active" : ""}`} onClick={() => setScreen("IMAGE_CONFIGURE")} aria-current={screen.startsWith("IMAGE_") ? "page" : undefined}><strong><i className="meshy-lab__tool-icon"><MeshyWorkspaceIcon tool="image" /></i>Image</strong><span>Create or edit 2D references</span></button>
        <button type="button" aria-label="Generated models" data-workspace-tool="assets" className={`meshy-lab__navigation-item${screen === "HISTORY" ? " meshy-lab__navigation-item--active" : ""}`} onClick={() => browseHistory(1, true)} disabled={busy} aria-current={screen === "HISTORY" ? "page" : undefined}><strong><i className="meshy-lab__tool-icon"><MeshyWorkspaceIcon tool="assets" /></i>Assets</strong><span>Browse and recover verified GLBs</span></button>
        {run ? <button type="button" data-workspace-tool="current" className={`meshy-lab__navigation-item${screen === "RUN" ? " meshy-lab__navigation-item--active" : ""}`} onClick={() => setScreen("RUN")} aria-current={screen === "RUN" ? "page" : undefined}><strong><i className="meshy-lab__tool-icon"><MeshyWorkspaceIcon tool="current" /></i>Current generation</strong><span>{run.status.toLowerCase()}</span></button> : null}
        {imageRun ? <button type="button" data-workspace-tool="current" className={`meshy-lab__navigation-item${screen === "IMAGE_RUN" ? " meshy-lab__navigation-item--active" : ""}`} onClick={() => setScreen("IMAGE_RUN")} aria-current={screen === "IMAGE_RUN" ? "page" : undefined}><strong><i className="meshy-lab__tool-icon"><MeshyWorkspaceIcon tool="current" /></i>Current image</strong><span>{imageRun.status.toLowerCase()}</span></button> : null}
      </nav>
      <div className="meshy-lab__content">
        {screen === "CONFIGURE" ? <main className="meshy-lab__create-workspace" aria-label="New Meshy model">
          <aside className="panel meshy-lab__creator-controls">
            <div className="meshy-lab__configuration-heading"><div><p className="eyebrow">Meshy API generation</p><h2>New model</h2></div><p>{availableCredits ?? "—"} credits</p></div>
            <fieldset className="meshy-lab__source-mode"><legend>Source</legend><label aria-label="Text to 3D"><input type="radio" name="meshy-generation-source" checked={generationSource === "TEXT"} onChange={() => chooseSource("TEXT")} /><i className="meshy-lab__source-icon"><MeshySourceIcon source="TEXT" /></i><span>Text</span></label><label aria-label="Image to 3D"><input type="radio" name="meshy-generation-source" checked={generationSource === "IMAGE"} onChange={() => chooseSource("IMAGE")} /><i className="meshy-lab__source-icon"><MeshySourceIcon source="IMAGE" /></i><span>Image</span></label><label aria-label="Multi-image to 3D"><input type="radio" name="meshy-generation-source" checked={generationSource === "MULTI_IMAGE"} onChange={() => chooseSource("MULTI_IMAGE")} /><i className="meshy-lab__source-icon"><MeshySourceIcon source="MULTI_IMAGE" /></i><span>Multi-image</span></label></fieldset>
            {generationSource === "TEXT" ? <label className="meshy-lab__field" htmlFor="meshy-asset-prompt">Prompt<textarea id="meshy-asset-prompt" value={prompt} onChange={(event) => setPrompt(event.target.value)} maxLength={600} placeholder="Describe the model you want to generate" /></label> : <div className="meshy-lab__reference-upload"><strong className="meshy-lab__reference-title">Reference image{generationSource === "MULTI_IMAGE" ? "s" : ""}</strong><label className="meshy-lab__reference-dropzone" htmlFor="meshy-reference-images" onDragOver={(event) => event.preventDefault()} onDrop={(event) => { event.preventDefault(); chooseReferenceImages(event.dataTransfer.files); }}><input className="meshy-lab__reference-input" id="meshy-reference-images" type="file" accept="image/png,image/jpeg" multiple={generationSource === "MULTI_IMAGE"} onChange={(event) => chooseReferenceImages(event.currentTarget.files ?? [])} /><i className="meshy-lab__reference-dropzone-icon"><MeshySourceIcon source="IMAGE" /></i><strong>{generationSource === "MULTI_IMAGE" ? "Choose up to four reference images" : "Choose a reference image"}</strong><span>PNG or JPEG · stays local until confirmation</span></label><p>{referenceImages.length ? referenceImages.map((file) => file.name).join(", ") : generationSource === "MULTI_IMAGE" ? "One to four images are required." : "One image is required."}</p></div>}
            <div className="meshy-lab__api-grid"><fieldset className="meshy-lab__segmented-control meshy-lab__segmented-control--model-type"><legend>Typ modelu</legend><label><input type="radio" name="meshy-model-type" checked={apiOptions.modelType !== "smart-topology"} onChange={() => chooseModelType("standard")} />Standard</label><label><input type="radio" name="meshy-model-type" checked={apiOptions.modelType === "smart-topology"} onChange={() => chooseModelType("smart-topology")} disabled={generationSource !== "IMAGE"} />Smart topology</label></fieldset><label>Model AI<select value={apiOptions.aiModel} onChange={(event) => patchApiOptions({ aiModel: event.target.value as MeshyTextTo3DOptions["aiModel"] })}>{apiOptions.modelType === "smart-topology" ? <><option value="meshy-t2">Meshy T2</option><option value="meshy-t1">Meshy T1</option></> : <><option value="meshy-6">Meshy 6</option><option value="meshy-5">Meshy 5</option></>}</select></label><label className="meshy-lab__inline-toggle"><span>Automatyczne dzielenie <i className="meshy-lab__setting-info" title="Uses Meshy API decimation_mode when enabled." aria-label="Uses Meshy API decimation mode when enabled.">i</i></span><input className="meshy-lab__switch-input" type="checkbox" checked={apiOptions.decimationMode !== undefined} onChange={(event) => patchApiOptions({ decimationMode: event.target.checked ? 2 : undefined })} disabled={!apiOptions.shouldRemesh || apiOptions.modelType !== "standard"} /><i className="meshy-lab__switch-track" aria-hidden="true" /></label><fieldset className="meshy-lab__segmented-control"><legend>Pozować</legend><label><input type="radio" name="meshy-pose" checked={apiOptions.poseMode === ""} onChange={() => patchApiOptions({ poseMode: "" })} />Bez pozy</label><label><input type="radio" name="meshy-pose" checked={apiOptions.poseMode === "a-pose"} onChange={() => patchApiOptions({ poseMode: "a-pose" })} />A-Pozycja</label><label><input type="radio" name="meshy-pose" checked={apiOptions.poseMode === "t-pose"} onChange={() => patchApiOptions({ poseMode: "t-pose" })} />T-Pozycja</label></fieldset></div>
            {generationSource !== "TEXT" ? <label className="meshy-lab__inline-toggle meshy-lab__inline-toggle--image-enhancement"><span>Ulepszanie obrazu <i className="meshy-lab__setting-info" title="Enhances the reference image before the paid task." aria-label="Enhances the reference image before the paid task.">i</i></span><input className="meshy-lab__switch-input" type="checkbox" checked={apiOptions.imageEnhancement} onChange={(event) => patchApiOptions({ imageEnhancement: event.target.checked })} disabled={!(["latest", "meshy-6"] as const).includes(apiOptions.aiModel as "latest" | "meshy-6")} /><i className="meshy-lab__switch-track" aria-hidden="true" /></label> : null}
            <details className="meshy-lab__advanced"><summary>Geometry</summary><div className="meshy-lab__api-grid"><label>Topology<select value={apiOptions.topology} onChange={(event) => patchApiOptions({ topology: event.target.value as MeshyTextTo3DOptions["topology"] })} disabled={!apiOptions.shouldRemesh || apiOptions.modelType !== "standard"}><option value="triangle">Triangle</option><option value="quad">Quad dominant</option></select></label>{generationSource !== "MULTI_IMAGE" ? <label>Legacy model type<select value={apiOptions.modelType === "lowpoly" ? "lowpoly" : "standard"} onChange={(event) => chooseModelType(event.target.value as "standard" | "lowpoly")}><option value="standard">Current standard</option><option value="lowpoly">Low poly (legacy)</option></select></label> : null}<label>Target polycount<input type="number" min="100" max={apiOptions.modelType === "smart-topology" && apiOptions.aiModel === "meshy-t2" ? 15_000 : AURORA_MODEL_TRIANGLE_BUDGET_V1} value={apiOptions.targetPolycount} onChange={(event) => patchApiOptions({ targetPolycount: Number(event.target.value) })} disabled={apiOptions.modelType === "lowpoly" || apiOptions.aiModel === "meshy-t1" || (apiOptions.modelType === "standard" && !apiOptions.shouldRemesh)} /></label>{apiOptions.decimationMode !== undefined ? <label>Adaptive decimation<select aria-label="Adaptive decimation level" value={apiOptions.decimationMode} onChange={(event) => patchApiOptions({ decimationMode: Number(event.target.value) as 1 | 2 | 3 | 4 })} disabled={!apiOptions.shouldRemesh || apiOptions.modelType !== "standard"}><option value="1">Ultra</option><option value="2">High</option><option value="3">Medium</option><option value="4">Low</option></select></label> : <p className="meshy-lab__hint">Adaptive decimation is off. The value is a Meshy target, not an exact output; the Bridge measures the generated GLB and remeshes safely before rigging when it exceeds {AURORA_MODEL_TRIANGLE_BUDGET_V1.toLocaleString("en-US")} triangles.</p>}</div><div className="meshy-lab__api-toggles"><label><input type="checkbox" checked={apiOptions.shouldRemesh} onChange={(event) => patchApiOptions({ shouldRemesh: event.target.checked })} disabled={apiOptions.modelType !== "standard"} />Remesh</label><label><input type="checkbox" checked={apiOptions.moderation} onChange={(event) => patchApiOptions({ moderation: event.target.checked })} />Moderation</label><label><input type="checkbox" checked={apiOptions.autoSize} onChange={(event) => patchApiOptions({ autoSize: event.target.checked })} />Auto-size</label><label>Origin<select value={apiOptions.originAt} onChange={(event) => patchApiOptions({ originAt: event.target.value as "bottom" | "center" })} disabled={!apiOptions.autoSize}><option value="bottom">Bottom</option><option value="center">Center</option></select></label></div></details>
            <details className="meshy-lab__advanced"><summary>Texture & output</summary><div className="meshy-lab__api-grid"><label>Texture prompt<textarea value={apiOptions.texturePrompt} onChange={(event) => patchApiOptions({ texturePrompt: event.target.value })} maxLength={600} placeholder="Optional texture guidance" /></label><label>Texture image URL<input type="url" value={apiOptions.textureImageUrl} onChange={(event) => patchApiOptions({ textureImageUrl: event.target.value })} placeholder="Optional public image or data URI" /></label></div><div className="meshy-lab__api-toggles"><label><input type="checkbox" checked={apiOptions.shouldTexture} onChange={(event) => patchApiOptions({ shouldTexture: event.target.checked })} />Generate texture</label><label><input type="checkbox" checked={apiOptions.enablePbr} onChange={(event) => patchApiOptions({ enablePbr: event.target.checked })} disabled={!apiOptions.shouldTexture} />PBR maps</label><label><input type="checkbox" checked={apiOptions.hdTexture} onChange={(event) => patchApiOptions({ hdTexture: event.target.checked })} disabled={!apiOptions.shouldTexture} />4K texture</label><label><input type="checkbox" checked={apiOptions.removeLighting} onChange={(event) => patchApiOptions({ removeLighting: event.target.checked })} disabled={!apiOptions.shouldTexture} />Remove baked lighting</label>{generationSource !== "TEXT" ? <label><input type="checkbox" checked={apiOptions.imageEnhancement} onChange={(event) => patchApiOptions({ imageEnhancement: event.target.checked })} disabled={!(["latest", "meshy-6"] as const).includes(apiOptions.aiModel as "latest" | "meshy-6")} />Enhance reference image</label> : null}<label><input type="checkbox" checked={apiOptions.alphaThumbnail} onChange={(event) => patchApiOptions({ alphaThumbnail: event.target.checked })} />Transparent thumbnail</label><label><input type="checkbox" checked={apiOptions.multiViewThumbnails} onChange={(event) => patchApiOptions({ multiViewThumbnails: event.target.checked })} />Four-view thumbnails</label></div><p className="meshy-lab__hint">Studio requests and imports GLB only. Other Meshy export formats need a separate, verified Bridge artifact contract.</p></details>
            <details className="meshy-lab__advanced">
              <summary>Humanoid post-processing</summary>
              <div className="meshy-lab__api-grid">
                <label>Height (m)<input type="number" min="0.5" max="3" step="0.01" value={apiOptions.rigHeightMeters} onChange={(event) => patchApiOptions({ rigHeightMeters: Number(event.target.value) })} disabled={!apiOptions.rigHumanoid} /></label>
              </div>
              <div className="meshy-lab__api-toggles"><label><input type="checkbox" checked={apiOptions.rigHumanoid} onChange={(event) => patchApiOptions({ rigHumanoid: event.target.checked })} />Rig as standard humanoid</label></div>
              {apiOptions.rigHumanoid ? <div className="meshy-lab__animation-mappings">
                <div className="meshy-lab__animation-mappings-heading">
                  <div><strong>Meshy animations</strong><small>Map each paid Meshy action to one native NWN Creature clip.</small></div>
                  <button type="button" className="button button--secondary" onClick={addAnimationAction} disabled={apiOptions.animationActions.length >= 10}>Add Meshy animation</button>
                </div>
                {apiOptions.animationActions.map((action, index) => <div className="meshy-lab__animation-mapping" key={index}>
                  <label>Meshy action ID<input aria-label={`Meshy action ID ${index + 1}`} type="number" min="0" step="1" value={action.actionId} onChange={(event) => patchAnimationAction(index, { actionId: Number(event.target.value) })} /></label>
                  <label>NWN clip<select aria-label={`NWN clip ${index + 1}`} value={action.clipName} onChange={(event) => patchAnimationAction(index, { clipName: event.target.value as MeshyAnimationActionMapping["clipName"] })}>{NWN_DIRECT_CREATURE_CLIPS.map((clipName) => <option key={clipName} value={clipName}>{clipName}</option>)}</select></label>
                  <button type="button" className="button button--quiet" aria-label={`Remove Meshy animation ${index + 1}`} onClick={() => removeAnimationAction(index)} disabled={apiOptions.animationActions.length <= 1}>Remove</button>
                </div>)}
                <small>{apiOptions.animationActions.length}/10 actions. `cpause1` is required once as the source idle.</small>
                {animationErrors.length ? <ul className="meshy-lab__validation-errors">{animationErrors.map((message) => <li key={message}>{message}</li>)}</ul> : null}
              </div> : null}
            </details>
            <div className="meshy-lab__review-cta"><small>Nothing is charged until the confirmation screen.</small><button type="button" className="button button--primary" onClick={review} disabled={busy || !canReview}>Review generation</button></div>
          </aside>
          <section className="panel meshy-lab__creator-canvas">{viewerArtifact ? <div className="meshy-lab__model-viewport"><MeshyModelViewport artifact={viewerArtifact} label={viewerLabel} onError={setError} onClose={() => setViewerArtifact(undefined)} /></div> : <div className="meshy-lab__canvas-empty"><p className="eyebrow">{generationSource.replaceAll("_", " ")} TO 3D</p><h2>What will you create?</h2><p>{generationSource === "TEXT" ? "Describe a model on the left, then review its exact generation settings." : "Add reference images on the left, then review the request before it reaches Meshy."}</p><small>Configure the request on the left. The review action remains visible at the bottom of that panel.</small></div>}</section>
          {retextureInputTaskId ? <section className="panel meshy-lab__retexture" aria-label="Material matching">
            <p className="eyebrow">Material matching</p><h2>ReTexture</h2>
            {!retexturePreview ? <><p>Apply a new style to this verified Meshy model. The Bridge sends only its task identity; signed model URLs never reach Studio.</p><label>Style prompt<textarea value={retextureStylePrompt} onChange={(event) => setRetextureStylePrompt(event.target.value)} maxLength={600} placeholder="Describe the target material and finish" /></label><div className="meshy-lab__actions"><button type="button" className="button button--secondary" onClick={() => setRetextureInputTaskId(undefined)} disabled={busy}>Cancel</button><button type="button" className="button button--primary" onClick={previewRetexture} disabled={busy || !retextureStylePrompt.trim()}>Review ReTexture</button></div></> : null}
            {retexturePreview && !retextureRun ? <><dl><div><dt>Model</dt><dd>{retexturePreview.aiModel}</dd></div><div><dt>Original UV</dt><dd>{retexturePreview.enableOriginalUv ? "Keep" : "Regenerate"}</dd></div><div><dt>PBR maps</dt><dd>{retexturePreview.enablePbr ? "On" : "Off"}</dd></div><div><dt>Maximum cost</dt><dd>{retexturePreview.maximumCredits} credits maximum</dd></div></dl><p>Your account is charged only after confirmation. No ReTexture task exists yet.</p><div className="meshy-lab__actions"><button type="button" className="button button--secondary" onClick={() => setRetexturePreview(undefined)} disabled={busy}>Back</button><button type="button" className="button button--primary" onClick={createRetexture} disabled={busy}>Confirm ReTexture</button></div></> : null}
            {retextureRun ? <><p>{retextureRun.status === "READY" ? "ReTexture is verified and loaded in the viewport." : retextureRun.status === "FAILED" ? "ReTexture failed." : `${retextureRun.status.toLowerCase()} · ${retextureRun.progress}%`}</p>{retextureRun.error ? <p className="meshy-lab__error">{retextureRun.error.message}</p> : null}{retextureRun.status !== "READY" && retextureRun.status !== "FAILED" && retextureRun.status !== "CANCELED" ? <button type="button" className="button button--secondary" onClick={refreshRetexture} disabled={busy}>Refresh ReTexture</button> : null}</> : null}
          </section> : null}
          <MeshyLibraryPanel bridge={bridge} sessionToken={sessionToken} items={libraryItems} query={libraryQuery} busy={busy} page={history?.pageNum ?? 1} hasNext={history?.hasNext ?? false} newestFirst={libraryNewestFirst} retextureTaskId={viewerArtifact?.provenance.taskIds.REFINE ?? viewerArtifact?.provenance.taskIds.PREVIEW} onQueryChange={setLibraryQuery} onToggleSort={() => setLibraryNewestFirst((current) => !current)} onBrowse={() => browseHistory(1, true)} onPreviousPage={() => browseHistory(Math.max(1, (history?.pageNum ?? 1) - 1), false, false)} onNextPage={() => browseHistory((history?.pageNum ?? 1) + 1, false, false)} onOpen={openHistoryViewer} onRetexture={openRetexture} />
        </main> : null}
        {screen === "IMAGE_CONFIGURE" ? <main className="meshy-lab__create-workspace" aria-label="Create Meshy concept image">
          <aside className="panel meshy-lab__creator-controls">
            <div className="meshy-lab__configuration-heading"><div><p className="eyebrow">Meshy API generation</p><h2>Concept images</h2></div><p>{availableCredits ?? "—"} credits</p></div>
            <fieldset className="meshy-lab__source-mode"><legend>Source</legend><label><input type="radio" name="meshy-image-source" checked={imageMode === "TEXT_TO_IMAGE"} onChange={() => setImageMode("TEXT_TO_IMAGE")} />Text to Image</label><label><input type="radio" name="meshy-image-source" checked={imageMode === "IMAGE_TO_IMAGE"} onChange={() => setImageMode("IMAGE_TO_IMAGE")} />Image to Image</label></fieldset>
            <label className="meshy-lab__field" htmlFor="meshy-image-prompt">Prompt<textarea id="meshy-image-prompt" value={imagePrompt} onChange={(event) => setImagePrompt(event.target.value)} maxLength={600} placeholder="Describe the reference image you want" /></label>
            {imageMode === "IMAGE_TO_IMAGE" ? <div className="meshy-lab__reference-upload"><label className="meshy-lab__reference-dropzone" htmlFor="meshy-image-references" onDragOver={(event) => event.preventDefault()} onDrop={(event) => { event.preventDefault(); chooseConceptReferenceImages(event.dataTransfer.files); }}><input className="meshy-lab__reference-input" id="meshy-image-references" type="file" accept="image/png,image/jpeg" multiple onChange={(event) => chooseConceptReferenceImages(event.currentTarget.files ?? [])} /><i className="meshy-lab__reference-dropzone-icon"><MeshySourceIcon source="IMAGE" /></i><strong>Choose reference images</strong><span>One to five PNG or JPEG images</span></label><p>{imageReferenceFiles.length ? imageReferenceFiles.map((file) => file.name).join(", ") : "Images stay local until confirmation."}</p></div> : null}
            <div className="meshy-lab__api-grid"><label>Image model<select value={imageModel} onChange={(event) => setImageModel(event.target.value as MeshyImageAiModel)}><option value="nano-banana">Nano Banana</option><option value="nano-banana-2">Nano Banana 2</option><option value="nano-banana-pro">Nano Banana Pro</option><option value="gpt-image-2">GPT Image 2</option></select></label>{!imageMultiView ? <label>Aspect ratio<select value={imageAspectRatio} onChange={(event) => setImageAspectRatio(event.target.value as typeof imageAspectRatio)}>{(imageModel === "gpt-image-2" ? ["1:1", "3:2", "2:3"] : ["1:1", "16:9", "9:16", "4:3", "3:4"]).map((ratio) => <option key={ratio} value={ratio}>{ratio}</option>)}</select></label> : null}{imageMode === "TEXT_TO_IMAGE" ? <label>Pose<select value={imagePose ?? ""} onChange={(event) => setImagePose(event.target.value ? event.target.value as "a-pose" | "t-pose" : undefined)}><option value="">No pose</option><option value="a-pose">A-pose</option><option value="t-pose">T-pose</option></select></label> : null}</div>
            <div className="meshy-lab__api-toggles"><label><input type="checkbox" checked={imageMultiView} onChange={(event) => setImageMultiView(event.target.checked)} />Generate multi-view reference set</label></div>
          </aside>
          <section className="panel meshy-lab__creator-canvas"><div className="meshy-lab__canvas-empty"><p className="eyebrow">{imageMode.replaceAll("_", " ")}</p><h2>Prepare a 2D reference</h2><p>Create a concept image, then explicitly confirm its charge. The completed Meshy task can be used as the source for a 3D run without revealing its signed image URL.</p><button type="button" className="button button--primary" onClick={reviewImage} disabled={busy || !imagePrompt.trim() || (imageMode === "IMAGE_TO_IMAGE" && !imageReferenceFiles.length)}>Review image generation</button><small>Nothing is charged until the next confirmation screen.</small></div></section>
          <MeshyLibraryPanel bridge={bridge} sessionToken={sessionToken} items={libraryItems} query={libraryQuery} busy={busy} page={history?.pageNum ?? 1} hasNext={history?.hasNext ?? false} newestFirst={libraryNewestFirst} onQueryChange={setLibraryQuery} onToggleSort={() => setLibraryNewestFirst((current) => !current)} onBrowse={() => browseHistory(1, true)} onPreviousPage={() => browseHistory(Math.max(1, (history?.pageNum ?? 1) - 1), false, false)} onNextPage={() => browseHistory((history?.pageNum ?? 1) + 1, false, false)} onOpen={openHistoryViewer} />
        </main> : null}
        {screen === "IMAGE_REVIEW" && imagePreview ? <div className="meshy-lab__review panel"><p className="eyebrow">Explicit paid operation</p><h2>Review image generation</h2><dl><div><dt>Mode</dt><dd>{imagePreview.mode.replaceAll("_", " ")}</dd></div><div><dt>Model</dt><dd>{imagePreview.aiModel}</dd></div><div><dt>Multi-view</dt><dd>{imagePreview.generateMultiView ? "Yes" : "No"}</dd></div><div><dt>Maximum cost</dt><dd>{imagePreview.maximumCredits} credits maximum</dd></div></dl><p>Your Meshy account is charged only after confirmation. No task exists yet. The Bridge keeps the resulting signed image URL private.</p><div className="meshy-lab__actions"><button type="button" className="button button--secondary" onClick={() => setScreen("IMAGE_CONFIGURE")} disabled={busy}>Back to configuration</button><button type="button" className="button button--primary" onClick={generateImage} disabled={busy}>Generate image</button></div></div> : null}
        {screen === "IMAGE_RUN" && imageRun ? <div className="meshy-lab__run panel"><p className="eyebrow">2D task</p><h2>{imageRun.status === "READY" ? "Image ready" : imageRun.status === "FAILED" ? "Image generation failed" : imageRun.status === "CANCELED" ? "Image generation canceled" : "Image generation queued"}</h2><p>{imageRun.progress}% complete</p>{imageRun.error ? <p className="meshy-lab__error">{imageRun.error.message}</p> : null}{imageRun.status === "READY" ? <p>The completed Meshy task is available as a private 3D input.</p> : null}<div className="meshy-lab__actions">{imageRun.status === "READY" ? <button type="button" className="button button--primary" onClick={useImageFor3d}>Use as 3D source</button> : null}<button type="button" className="button button--secondary" onClick={refreshImage} disabled={busy || imageRun.status === "READY" || imageRun.status === "FAILED" || imageRun.status === "CANCELED"}>Refresh status</button><button type="button" className="button button--quiet" onClick={cancelImage} disabled={busy || imageRun.status === "READY" || imageRun.status === "FAILED" || imageRun.status === "CANCELED"}>Cancel</button></div></div> : null}
        {screen === "HISTORY" && history ? <div className="meshy-lab__history panel"><p className="eyebrow">No-cost recovery</p><h2>Meshy history</h2><p>Choose a finished refinement to recover its verified GLB. Preview records remain under All tasks for inspection.</p><div className="meshy-lab__history-toolbar"><p aria-live="polite">Page {history.pageNum} · showing {visibleHistoryItems.length} of {history.items.length} tasks</p><div className="meshy-lab__history-filters" role="group" aria-label="History filter"><button type="button" className={`button button--secondary meshy-lab__history-filter${historyFilter === "READY" ? " meshy-lab__history-filter--selected" : ""}`} onClick={() => { setHistoryFilter("READY"); setSelectedHistoryTaskId(undefined); }} aria-pressed={historyFilter === "READY"}>Ready to recover ({readyHistoryItems.length})</button><button type="button" className={`button button--secondary meshy-lab__history-filter${historyFilter === "ALL" ? " meshy-lab__history-filter--selected" : ""}`} onClick={() => { setHistoryFilter("ALL"); setSelectedHistoryTaskId(undefined); }} aria-pressed={historyFilter === "ALL"}>All tasks ({history.items.length})</button></div></div>{visibleHistoryItems.length ? <><div className="meshy-lab__history-selection" aria-live="polite"><div><strong>{selectedHistoryItem ? (selectedHistoryItem.prompt || "Selected Meshy task") : "Select a model to recover"}</strong><p>{selectedHistoryItem ? (canRecoverHistoryItem(selectedHistoryItem) ? "Its verified GLB is ready to import into Source." : "This preview has no recoverable GLB. Select its finished refinement instead.") : "Choose a card below. The recovery action stays here so the grid remains easy to scan."}</p></div><button type="button" className="button button--primary" onClick={() => selectedHistoryItem && recoverHistoryArtifact(selectedHistoryItem.taskId)} disabled={busy || !selectedHistoryItem || !canRecoverHistoryItem(selectedHistoryItem)}>Recover selected GLB</button></div><div className="meshy-lab__history-grid">{visibleHistoryItems.map((item) => <button key={item.taskId} type="button" className={`meshy-lab__history-card${selectedHistoryTaskId === item.taskId ? " meshy-lab__history-card--selected" : ""}`} data-history-task-id={item.taskId} onClick={() => setSelectedHistoryTaskId(item.taskId)} aria-pressed={selectedHistoryTaskId === item.taskId}><div className="meshy-lab__history-card-summary"><strong>{item.stage} · {item.status}</strong><p>{item.prompt || "No prompt returned by Meshy."}</p><small>{item.createdAt ? `Created ${new Date(item.createdAt).toLocaleString()}` : "Creation time unavailable"}{item.consumedCredits === undefined ? "" : ` · ${item.consumedCredits} credits`}</small></div><span className="meshy-lab__history-card-availability">{canRecoverHistoryItem(item) ? "Verified GLB ready" : "Preview only"}</span></button>)}</div></> : <p>{historyFilter === "READY" ? "No verified GLBs are ready to recover on this page." : "No Text-to-3D tasks are available on this Meshy account page."}</p>}<div className="meshy-lab__actions"><button type="button" className="button button--secondary" onClick={() => setScreen("CONFIGURE")} disabled={busy}>Back to generation</button><button type="button" className="button button--secondary" onClick={() => browseHistory(history.pageNum - 1)} disabled={busy || history.pageNum <= 1}>Previous page</button><button type="button" className="button button--secondary" onClick={() => browseHistory(history.pageNum + 1)} disabled={busy || !history.hasNext}>Next page</button></div></div> : null}
        {screen === "REVIEW" && preview ? <div className="meshy-lab__review panel"><p className="eyebrow">Explicit paid operation</p><h2>Review generation</h2><dl><div><dt>Profile</dt><dd>{profileCode(preview.profile)} {preview.profile.label}</dd></div><div><dt>Source</dt><dd>{generationSource.replaceAll("_", " ")}</dd></div><div><dt>Output</dt><dd>{apiOptions.targetFormats.map((format) => format.toUpperCase()).join(", ")}</dd></div><div><dt>Pipeline</dt><dd>{preview.stages.join(" → ")}</dd></div>{apiOptions.rigHumanoid ? <div><dt>Animation mapping</dt><dd>{apiOptions.animationActions.map(({ actionId, clipName }) => `${actionId} → ${clipName}`).join(" · ")}</dd></div> : null}<div><dt>Maximum cost</dt><dd>{preview.maximumCredits} credits maximum</dd></div></dl><p>Your Meshy account is charged only after confirmation. No task exists yet.</p><div className="meshy-lab__actions"><button type="button" className="button button--secondary" onClick={() => setScreen("CONFIGURE")} disabled={busy}>Back to configuration</button><button type="button" className="button button--primary" onClick={generate} disabled={busy}>Generate model</button></div></div> : null}
        {screen === "RUN" && run ? <div className="meshy-lab__run panel"><p className="eyebrow">Meshy run</p><h2>{run.status === "QUEUED" ? "Generation queued" : `Run ${run.status.toLowerCase()}`}</h2><p>{profileCode(run.profile)} · {run.progress}% · {run.profile.stages.join(" → ")}</p><p>Task IDs remain in local provenance and are never used as credentials.</p><div className="meshy-lab__actions"><button type="button" className="button button--secondary" onClick={refresh} disabled={busy}>Refresh status</button>{run.status !== "READY" && run.status !== "CANCELED" && run.status !== "FAILED" ? <button type="button" className="button button--quiet" onClick={cancel} disabled={busy}>Cancel run</button> : null}{run.status === "READY" ? <><button type="button" className="button button--secondary" onClick={openCurrentViewer} disabled={busy}>Open in Meshy viewport</button><button type="button" className="button button--secondary" onClick={downloadProvenance} disabled={busy}>Download provenance</button><button type="button" className="button button--primary" onClick={importArtifact} disabled={busy}>Import verified GLB to Source</button></> : null}</div><details className="meshy-lab__technical"><summary>Technical details</summary><p>Run {run.id}</p><p>{Object.entries(run.taskIds).map(([stage, taskId]) => `${stage}: ${taskId}`).join(" · ") || "No Meshy task IDs yet."}</p></details></div> : null}
      </div>
    </div> : null}
  </section>;
}
