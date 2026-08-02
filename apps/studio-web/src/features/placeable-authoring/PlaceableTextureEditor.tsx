import { useEffect, useReducer, useState } from "react";
import {
  createTextureOverrideBindingV1,
  type PlaceableTextureAuthoringBootstrap,
  type PlaceableTextureEditorSnapshot,
  type ResolvedPlaceableTextures,
} from "./textureTypes";
import {
  createPlaceableTextureEditorState,
  placeableTextureEditorReducer,
} from "./textureState";

interface Props {
  readonly bootstrap: PlaceableTextureAuthoringBootstrap;
  readonly resolved?: ResolvedPlaceableTextures;
  readonly onSnapshotChange: (snapshot: PlaceableTextureEditorSnapshot) => void;
  readonly onError?: (message: string) => void;
}

function OverrideThumbnail({ file }: { readonly file?: File }) {
  const [url, setUrl] = useState<string>();
  useEffect(() => {
    if (!file) {
      setUrl(undefined);
      return;
    }
    const next = URL.createObjectURL(file);
    setUrl(next);
    return () => URL.revokeObjectURL(next);
  }, [file]);
  return url ? <img src={url} alt="Replacement texture preview" /> : <span>Source GLB</span>;
}

export function PlaceableTextureEditor({
  bootstrap,
  resolved,
  onSnapshotChange,
  onError,
}: Props) {
  const [state, dispatch] = useReducer(
    placeableTextureEditorReducer,
    bootstrap.document,
    createPlaceableTextureEditorState,
  );
  const [files, setFiles] = useState<ReadonlyMap<string, File>>(() => new Map());
  const [busySlot, setBusySlot] = useState<number | null>(null);

  useEffect(() => {
    onSnapshotChange({
      document: state.present,
      files,
      preview: state.preview,
      selectedMaterialSlot: state.selectedMaterialSlot,
    });
  }, [files, onSnapshotChange, state.present, state.preview, state.selectedMaterialSlot]);

  return (
    <section className="placeable-textures panel" aria-label="Placeable material textures">
      <header className="placeable-textures__header">
        <div>
          <strong>Material textures</strong>
          <span>{state.present.bindings.length} independent slots · UV0 preserved</span>
        </div>
        <div className="placeable-textures__actions">
          <button
            type="button"
            data-active={state.preview === "SOURCE"}
            onClick={() => dispatch({ type: "SET_PREVIEW", preview: "SOURCE" })}
          >Source</button>
          <button
            type="button"
            data-active={state.preview === "EDITED"}
            onClick={() => dispatch({ type: "SET_PREVIEW", preview: "EDITED" })}
          >Edited</button>
          <button type="button" disabled={!state.past.length} onClick={() => dispatch({ type: "UNDO" })}>Undo</button>
          <button type="button" disabled={!state.future.length} onClick={() => dispatch({ type: "REDO" })}>Redo</button>
          <button type="button" onClick={() => dispatch({ type: "RESET_ALL" })}>Reset all</button>
        </div>
      </header>
      <p className="placeable-textures__policy">
        PNG/JPEG → exact TGA. Experimental V1 accepts only fully opaque images; UVs and geometry are not regenerated.
      </p>
      <div className="placeable-textures__grid">
        {bootstrap.inspection.materials.map((material) => {
          const binding = state.present.bindings.find(
            (candidate) => candidate.materialSlot === material.materialSlot,
          );
          if (!binding) return null;
          const file = binding.overrideAssetId ? files.get(binding.overrideAssetId) : undefined;
          const output = resolved?.bindings.find(
            (candidate) => candidate.materialSlot === material.materialSlot,
          );
          return (
            <article
              key={material.materialSlot}
              data-selected={state.selectedMaterialSlot === material.materialSlot}
              onClick={() => dispatch({ type: "SELECT_MATERIAL", materialSlot: material.materialSlot })}
            >
              <div className="placeable-textures__thumbnail"><OverrideThumbnail file={file} /></div>
              <div className="placeable-textures__identity">
                <strong>{material.sourceMaterialName || `Material ${material.sourceMaterialId}`}</strong>
                <span>slot {material.materialSlot} · primitives {material.primitiveIds.join(", ") || "—"}</span>
                <span>{material.hasUv0 ? "UV0 present" : "UV0 missing"} · {material.sourceMimeType}</span>
                <span>
                  alpha {material.alphaMode}
                  {material.normalTexturePresent || material.metallicRoughnessTexturePresent || material.emissiveTexturePresent
                    ? " · source PBR maps ignored by diffuse override"
                    : " · diffuse only"}
                </span>
                <span className="placeable-textures__hash">source {material.sourceImageSha256.slice(0, 12)}…</span>
                <span>{binding.mode === "OVERRIDE" ? file?.name ?? "Override payload missing" : "Original GLB texture"}</span>
                <span>{output ? `${output.outputResref}.tga · ${output.outputSha256.slice(0, 12)}…` : "Resolving exact output…"}</span>
              </div>
              <div className="placeable-textures__row-actions">
                <label className="placeable-textures__replace">
                  {busySlot === material.materialSlot ? "Reading…" : "Replace"}
                  <input
                    type="file"
                    accept="image/png,image/jpeg,.png,.jpg,.jpeg"
                    disabled={busySlot !== null}
                    onChange={(event) => {
                      const input = event.currentTarget;
                      const selected = input.files?.[0];
                      input.value = "";
                      if (!selected) return;
                      setBusySlot(material.materialSlot);
                      void createTextureOverrideBindingV1(binding, selected)
                        .then(({ assetId, binding: next }) => {
                          setFiles((current) => new Map(current).set(assetId, selected));
                          dispatch({ type: "REPLACE", binding: next });
                          dispatch({ type: "SET_PREVIEW", preview: "EDITED" });
                          onError?.("");
                        })
                        .catch((error: unknown) => onError?.(
                          error instanceof Error ? error.message : String(error),
                        ))
                        .finally(() => setBusySlot(null));
                    }}
                  />
                </label>
                <button
                  type="button"
                  disabled={binding.mode === "SOURCE"}
                  onClick={() => dispatch({ type: "USE_SOURCE", materialSlot: material.materialSlot })}
                >Use source</button>
                <button
                  type="button"
                  onClick={() => dispatch({ type: "RESET_MATERIAL", materialSlot: material.materialSlot })}
                >Reset</button>
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}
