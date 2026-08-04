import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { StudioHeader } from "../../app/StudioHeader";
import { StudioShell } from "../../app/StudioShell";
import type { StudioTarget } from "../../app/studioSession";
import type { WorkflowStep } from "../../app/workflow";
import { WORKFLOW_STEPS } from "../../app/workflow";
import { WorkflowStepper } from "../../app/WorkflowStepper";
import { ArtifactDownloads } from "../downloads/ArtifactDownloads";
import { StudioWorkerClient } from "../../worker/client";
import {
  allocateItemNamespace,
  encodeWeaponPartAppearance,
  eulerDegreesToQuaternion,
  initialItemPartDrafts,
  projectItemBaseitemsCatalog,
  quaternionToEulerDegrees,
  requiresEquippedItemProof,
  resolveItemPartDraft,
} from "./domain";
import type { ItemNamespaceAllocation } from "./domain";
import { ItemCompositionViewport } from "./ItemCompositionViewport";
import { ItemIconArtifactPreview } from "./ItemIconArtifactPreview";
import { ItemGenerationPanel } from "./ItemGenerationPanel";
import { serializeItemGenerationSession, type ItemGenerationSessionV1 } from "./itemGeneration";
import type { MeshyArtifactProvenance, MeshyBridgeClient } from "../meshy/bridge";
import type { ItemSeamResult } from "./itemPreview";
import type {
  ItemBaseItemRow,
  ItemBaseItemsCatalog,
  ItemAttachmentProfileV1,
  ItemBuildSnapshot,
  ItemFitReport,
  ItemPartDraft,
  ItemWorkerClient,
  ResolvedItemPartDraft,
} from "./types";
import "./item.css";

const id = () => crypto.randomUUID();

export interface ItemWorkflowProps {
  readonly onTargetChange: (target: StudioTarget) => void;
  readonly client?: ItemWorkerClient;
  readonly meshyBridge?: MeshyBridgeClient;
}

interface ItemUtiNumericDraft {
  readonly cost: number;
  readonly addCost: number;
  readonly charges: number;
  readonly stackSize: number;
  readonly paletteId: number;
  readonly colors: Readonly<Record<string, number>>;
}

interface ItemPropertyDraft {
  readonly propertyName: number;
  readonly subtype: number;
  readonly costTable: number;
  readonly costValue: number;
  readonly param1: number;
  readonly param1Value: number;
  readonly chanceAppear: number;
}

interface ItemReferenceModelDraft {
  readonly field: string;
  readonly modelResref: string;
  readonly file: File;
}

const EMPTY_ITEM_PROPERTY: ItemPropertyDraft = {
  propertyName: 0,
  subtype: 0,
  costTable: 0,
  costValue: 0,
  param1: 255,
  param1Value: 0,
  chanceAppear: 100,
};

const EMPTY_UTI_NUMERIC: ItemUtiNumericDraft = {
  cost: 0,
  addCost: 0,
  charges: 0,
  stackSize: 1,
  paletteId: 0,
  colors: {},
};

function initialUtiNumeric(row: ItemBaseItemRow): ItemUtiNumericDraft {
  return {
    ...EMPTY_UTI_NUMERIC,
    colors: Object.fromEntries(row.colorFields.map((field) => [field, 0])),
  };
}

function initialFitTargetLengths(row: ItemBaseItemRow): number[] {
  return row.capability.meshySourceCount === 3 ? [0.30, 0.10, 0.60] : [1.0];
}

function integerInRange(value: number, maximum: number) {
  return Number.isInteger(value) && value >= 0 && value <= maximum;
}

function validPartTransform(part: ItemPartDraft) {
  return part.translation.every(Number.isFinite)
    && part.rotationDegrees.every(Number.isFinite)
    && part.rotationXyzw.every(Number.isFinite)
    && part.pivot.every(Number.isFinite)
    && part.targetSpaceScaleXyz.every((value) => Number.isFinite(value) && value > 0)
    && Number.isFinite(part.uniformScale)
    && part.uniformScale > 0;
}

function tupleEquals(first: readonly number[], second: readonly number[]) {
  return first.length === second.length
    && first.every((value, index) => value === second[index]);
}

function partMatchesFitContract(
  part: ItemPartDraft,
  fitReport: ItemFitReport | undefined,
) {
  if (part.sourceKind !== "MESHY_GLB") return true;
  const fitted = fitReport?.parts.find((candidate) => candidate.field === part.field);
  return Boolean(
    fitted
    && fitted.sourceNode === (part.sourceNode.trim() || null)
    && fitted.transform.uniformScale === part.uniformScale
    && tupleEquals(fitted.transform.translation, part.translation)
    && tupleEquals(fitted.transform.rotationXyzw, part.rotationXyzw)
    && tupleEquals(fitted.transform.pivot, part.pivot)
    && tupleEquals(fitted.targetSpaceScaleXyz, part.targetSpaceScaleXyz),
  );
}

function itemReferenceScalePercent(
  part: ItemPartDraft,
  fitReport: ItemFitReport | undefined,
  attachmentProfile: ItemAttachmentProfileV1 | undefined,
) {
  const fitted = fitReport?.parts.find((candidate) => candidate.field === part.field);
  if (!fitted || fitted.transform.uniformScale <= 0) return 100;
  const slot = attachmentProfile?.slots.find((candidate) => candidate.field === part.field);
  const referenceLength = slot ? slot.boundsMax[1] - slot.boundsMin[1] : 0;
  const fittedPercent = referenceLength > 0
    ? fitted.targetAxialLength / referenceLength * 100
    : 100;
  return fittedPercent * part.uniformScale / fitted.transform.uniformScale;
}

function itemPartSupportsReferenceScaling(
  part: ItemPartDraft,
  attachmentProfile: ItemAttachmentProfileV1 | undefined,
) {
  const slot = attachmentProfile?.slots.find((candidate) => candidate.field === part.field);
  return !slot || slot.allowAxialExtensionAtMin || slot.allowAxialExtensionAtMax;
}

function stepIndex(step: WorkflowStep) {
  return WORKFLOW_STEPS.indexOf(step);
}

function updateTuple(
  tuple: readonly [number, number, number],
  index: number,
  value: number,
): [number, number, number] {
  const next: [number, number, number] = [...tuple];
  next[index] = value;
  return next;
}

function ProfileOverview({ selected }: { readonly selected?: number }) {
  const profiles = [
    { type: 0, title: "Single MDL", detail: "ModelPart1 · no color channels", count: "1 part" },
    { type: 1, title: "Single MDL + colors", detail: "ModelPart1 · six color fields", count: "1 part" },
    { type: 2, title: "Bottom · Middle · Top", detail: "ModelPart1/B · ModelPart2/M · ModelPart3/T", count: "3 parts" },
    { type: 3, title: "CAPART body assembly", detail: "19 numeric UTI selectors · CAPART + 12 parts_* tables", count: "0 Meshy GLBs" },
  ];
  return (
    <section className="item-profile-overview" aria-label="Aurora item ModelType profiles">
      <header>
        <div><p className="eyebrow">Aurora item anatomy</p><h2>4 formal ModelTypes · 3 geometry schemas</h2></div>
        <div className="item-schema-pills"><span>1 GLB</span><span>3 GLBs</span><span>19 selectors</span></div>
      </header>
      <div className="item-profile-grid">
        {profiles.map((profile) => (
          <article key={profile.type} data-selected={selected === profile.type}>
            <b>{profile.type}</b>
            <div><strong>{profile.title}</strong><span>{profile.detail}</span></div>
            <em>{profile.count}</em>
          </article>
        ))}
      </div>
      <p className="item-contract-rule">
        <strong>DATA CONTRACT</strong>
        UTI.BaseItem → baseitems.2da.ModelType → exact part slots. There is no manual usage-family selector.
      </p>
    </section>
  );
}

function TargetRail({ onTargetChange }: ItemWorkflowProps) {
  return (
    <nav className="item-target-rail" aria-label="Conversion target">
      {([
        ["CREATURE", "Creature", "Rigged or static model"],
        ["PLACEABLE", "Placeable", "World object + collision"],
        ["ITEM", "Item", "Parts + UTI + icon layers"],
        ["TILE", "Tile", "Terrain piece"],
      ] as const).map(([target, label, description]) => (
        <button
          key={target}
          type="button"
          data-active={target === "ITEM"}
          onClick={() => target !== "ITEM" && onTargetChange(target)}
        >
          <span>◇</span><strong>{label}</strong><small>{description}</small>
        </button>
      ))}
      <p><strong>One Item case.</strong> The BaseItem row decides how the runtime assembles resources.</p>
    </nav>
  );
}

function ItemSource({
  onTargetChange,
  meshyBridge,
  baseitems,
  catalog,
  selected,
  parts,
  busy,
  error,
  onBaseitems,
  onSelectRow,
  onPartFile,
  onGeneratedPart,
  onGenerationSessionChange,
  referenceTables,
  onReferenceTable,
  referenceResources,
  onReferenceResources,
  referenceModels,
  attachmentProfile,
  onReferenceModel,
  onContinue,
}: ItemWorkflowProps & {
  readonly baseitems?: File;
  readonly catalog?: ItemBaseItemsCatalog;
  readonly selected?: ItemBaseItemRow;
  readonly parts: readonly ItemPartDraft[];
  readonly busy: boolean;
  readonly error?: string;
  readonly onBaseitems: (file: File) => void;
  readonly onSelectRow: (baseItem: number) => void;
  readonly onPartFile: (field: string, file: File) => void;
  readonly onGeneratedPart: (field: string, file: File, provenance: MeshyArtifactProvenance) => void;
  readonly onGenerationSessionChange: (session: ItemGenerationSessionV1) => void;
  readonly referenceTables: Readonly<Record<string, File>>;
  readonly onReferenceTable: (tableName: string, file: File) => void;
  readonly referenceResources: readonly File[];
  readonly onReferenceResources: (files: File[]) => void;
  readonly referenceModels: Readonly<Record<string, ItemReferenceModelDraft>>;
  readonly attachmentProfile?: ItemAttachmentProfileV1;
  readonly onReferenceModel: (field: string, file: File) => void;
  readonly onContinue: () => void;
}) {
  const requiresRetailResources = Boolean(
    selected
    && ["CAPART_COMPOSITE", "CLOAK_MODEL"].includes(selected.capability.iconProfile),
  );
  const resourceManifestCount = referenceResources.filter(
    (file) => file.name.toLowerCase().endsWith(".json"),
  ).length;
  const resourcePayloadCount = referenceResources.filter(
    (file) => /\.(?:mdl|plt)$/i.test(file.name),
  ).length;
  const ready = Boolean(
    selected
    && parts.length > 0
    && parts.every((part) => part.sourceKind !== "MESHY_GLB" || part.file)
    && selected.capability.requiredReferenceTables.every(
      (tableName) => referenceTables[tableName.toUpperCase()],
    )
    && (
      !requiresRetailResources
      || (resourceManifestCount === 1 && resourcePayloadCount > 0)
    )
    && (
      selected.modelType !== 2
      || selected.partSlots.every((slot) => referenceModels[slot.field])
    )
  );
  return (
    <section className="item-source">
      <header className="item-page-heading">
        <div><p className="eyebrow">Item · Source</p><h1>Create an Aurora item</h1><p>Choose BaseItem first. Its composer decides whether the inputs are Meshy GLBs or retail numeric selectors.</p></div>
        <span className="status-badge status-badge--neutral">Local-first</span>
      </header>
      <div className="item-source-grid">
        <TargetRail onTargetChange={onTargetChange} />
        <div className="item-source-main">
          <ProfileOverview selected={selected?.modelType} />
          <section className="item-resolver panel">
            <label>
              <span>1 · Reference table</span>
              <strong>{baseitems?.name ?? "baseitems.2da required"}</strong>
              <input
                aria-label="Base items table"
                type="file"
                accept=".2da"
                onChange={(event) => {
                  const file = event.currentTarget.files?.[0];
                  if (file) onBaseitems(file);
                }}
              />
            </label>
            <span>→</span>
            <label>
              <span>2 · Printed BaseItem row</span>
              <select
                aria-label="Target BaseItem"
                disabled={!catalog || busy}
                value={selected?.baseItem ?? ""}
                onChange={(event) => onSelectRow(Number(event.currentTarget.value))}
              >
                <option value="">Select BaseItem</option>
                {catalog?.rows.map((row) => (
                  <option key={row.baseItem} value={row.baseItem}>
                    {row.baseItem} · {row.label} · ModelType {row.modelType}
                  </option>
                ))}
              </select>
            </label>
            <span>→</span>
            <div>
              <span>3 · Derived schema</span>
              <strong>{selected ? `ModelType ${selected.modelType} · ${selected.partSlots.length} part${selected.partSlots.length === 1 ? "" : "s"}` : "pending"}</strong>
              <small>{selected?.partSlots.map((slot) => slot.label).join(" · ") ?? "No row selected"}</small>
            </div>
          </section>
          {selected?.modelType === 2 ? (
            <section className="item-reference-frame panel" aria-label="Reference appearance models">
              <header>
                <div>
                  <p className="eyebrow">Separate case · existing BaseItem</p>
                  <h2>Reference appearance and native slot frame</h2>
                  <p>Provide the exact read-only Bottom, Middle and Top MDLs from one Vanilla/HAK/MOD appearance. Their controller positions and model-space bounds define the grip and assembly frame.</p>
                </div>
                <span className={`status-badge ${attachmentProfile ? "status-badge--success" : "status-badge--neutral"}`}>
                  {attachmentProfile ? "Profile locked" : "Reference required"}
                </span>
              </header>
              <div className="item-reference-model-grid">
                {selected.partSlots.map((slot) => {
                  const reference = referenceModels[slot.field];
                  return (
                    <label key={slot.field}>
                      <b>{slot.token?.toUpperCase()}</b>
                      <span>
                        <strong>{slot.label} reference MDL</strong>
                        <small>{reference ? `${reference.modelResref} · ${reference.file.name}` : `${slot.field} · exact .mdl`}</small>
                      </span>
                      <input
                        aria-label={`${slot.field} reference MDL`}
                        type="file"
                        accept=".mdl,application/octet-stream"
                        onChange={(event) => {
                          const file = event.currentTarget.files?.[0];
                          if (file) onReferenceModel(slot.field, file);
                        }}
                      />
                    </label>
                  );
                })}
              </div>
              {attachmentProfile ? (
                <dl className="item-reference-profile-summary">
                  <div><dt>Profile</dt><dd><code>{attachmentProfile.profileSha256.slice(0, 16)}</code></dd></div>
                  <div><dt>Attachment</dt><dd>{attachmentProfile.attachmentRoute}</dd></div>
                  <div><dt>Origin</dt><dd><code>{attachmentProfile.commonOrigin.join(", ")}</code></dd></div>
                  <div><dt>Frame</dt><dd>Y axial · Z width · X depth</dd></div>
                </dl>
              ) : null}
            </section>
          ) : null}
          {selected && selected.capability.meshySourceCount > 0 && catalog ? (
            <ItemGenerationPanel
              row={selected}
              baseitemsSha256={catalog.sourceSha256}
              bridge={meshyBridge}
              onArtifact={onGeneratedPart}
              onSessionChange={onGenerationSessionChange}
            />
          ) : null}
          {selected ? (
            <section className="item-source-parts panel">
              <header><div><p className="eyebrow">Resolved composer inputs</p><h2>{selected.partSlots.length} exact UTI slot{selected.partSlots.length === 1 ? "" : "s"} · {selected.capability.meshySourceCount} Meshy source{selected.capability.meshySourceCount === 1 ? "" : "s"}</h2></div><code>{selected.capability.compositionProfile}</code></header>
              {selected.capability.requiredReferenceTables.length > 0 ? (
                <div className="item-reference-tables">
                  {selected.capability.requiredReferenceTables.map((tableName) => (
                    <label key={tableName}>
                      <span>{tableName}.2da</span>
                      <strong>{referenceTables[tableName.toUpperCase()]?.name ?? "required retail table"}</strong>
                      <input
                        aria-label={`${tableName} reference table`}
                        type="file"
                        accept=".2da"
                        onChange={(event) => {
                          const file = event.currentTarget.files?.[0];
                          if (file) onReferenceTable(tableName, file);
                        }}
                      />
                    </label>
                  ))}
                </div>
              ) : null}
              {requiresRetailResources ? (
                <label className="item-reference-resources">
                  <span>Exact retail resources (.mdl/.plt + manifest.json)</span>
                  <strong>{resourcePayloadCount > 0 ? `${resourcePayloadCount} validated payload candidates · ${resourceManifestCount} manifest` : "required pinned resource inventory"}</strong>
                  <input
                    aria-label="Runtime resource inventory"
                    type="file"
                    accept=".mdl,.plt,.json"
                    multiple
                    onChange={(event) => onReferenceResources(
                      Array.from(event.currentTarget.files ?? []),
                    )}
                  />
                </label>
              ) : null}
              <div className="item-source-part-grid">
                {parts.map((part, index) => (
                  <label key={part.field}>
                    <b>{part.token?.toUpperCase() ?? index + 1}</b>
                    <span><strong>{part.label}</strong><small>{part.field} · BYTE selector · {part.sourceKind}</small></span>
                    <em>{part.sourceKind === "MESHY_GLB" ? part.file?.name ?? "Select GLB" : "Retail reference selection · no GLB"}</em>
                    {part.sourceKind === "MESHY_GLB" ? (
                      <input
                        aria-label={`${part.field} source GLB`}
                        type="file"
                        accept=".glb,model/gltf-binary"
                        onChange={(event) => {
                          const file = event.currentTarget.files?.[0];
                          if (file) onPartFile(part.field, file);
                        }}
                      />
                    ) : null}
                  </label>
                ))}
              </div>
            </section>
          ) : null}
          {error ? <p className="item-error" role="alert">{error}</p> : null}
        </div>
        <aside className="item-contract panel">
          <p className="eyebrow">Resolved item contract</p>
          <h2>{selected ? `BaseItem ${selected.baseItem}` : "Waiting for table"}</h2>
          <dl>
            <div><dt>ItemClass</dt><dd><code>{selected?.itemClass ?? "—"}</code></dd></div>
            <div><dt>ModelType</dt><dd>{selected?.modelType ?? "—"}</dd></div>
            <div><dt>Part fields</dt><dd>{selected?.partSlots.length ?? "—"}</dd></div>
            <div><dt>Color fields</dt><dd>{selected?.colorFields.length ?? "—"}</dd></div>
            <div><dt>Icon canvas</dt><dd>{selected ? `${selected.invSlotWidth * 32} × ${selected.invSlotHeight * 32}` : "—"}</dd></div>
            <div><dt>Geometry budget</dt><dd>Σ parts ≤ 300,000</dd></div>
          </dl>
        </aside>
      </div>
      <footer className="item-action-bar">
        <p><strong>{ready ? "Every required composer input is assigned." : "Assign Meshy GLBs and every required retail reference table."}</strong><span>The schema comes from BaseItem capability and ModelType.</span></p>
        <button type="button" className="button button--primary" disabled={!ready || busy} onClick={onContinue}>Continue to Prepare Item</button>
      </footer>
    </section>
  );
}

function ItemPrepare({
  row,
  parts,
  utiNumeric,
  properties,
  namespaceAllocation,
  occupiedNamespace,
  capartModelPrefix,
  capartGenderCode,
  proofAppearanceRow,
  proofRace,
  proofGender,
  proofPhenotype,
  selectedField,
  preview,
  seamTolerance,
  fitTargetLengths,
  seams,
  fitReport,
  attachmentProfile,
  busy,
  error,
  onSelectPart,
  onPreview,
  onPartChange,
  onPartScalePreview,
  onUtiNumericChange,
  onPropertiesChange,
  onOccupiedNamespaceChange,
  onCapartModelPrefixChange,
  onCapartGenderCodeChange,
  onProofAppearanceRowChange,
  onProofRaceChange,
  onProofGenderChange,
  onProofPhenotypeChange,
  onSeamToleranceChange,
  onFitTargetLengthsChange,
  onAutoFit,
  onDiscardFit,
  onSeams,
  onBack,
  onContinue,
}: {
  readonly row: ItemBaseItemRow;
  readonly parts: readonly ItemPartDraft[];
  readonly utiNumeric: ItemUtiNumericDraft;
  readonly properties: readonly ItemPropertyDraft[];
  readonly namespaceAllocation?: ItemNamespaceAllocation;
  readonly occupiedNamespace: string;
  readonly capartModelPrefix: string;
  readonly capartGenderCode: string;
  readonly proofAppearanceRow: number;
  readonly proofRace: number;
  readonly proofGender: number;
  readonly proofPhenotype: number;
  readonly selectedField: string;
  readonly preview: "COMPOSED" | "EXPLODED" | "ICON";
  readonly seamTolerance: number;
  readonly fitTargetLengths: readonly number[];
  readonly seams: readonly ItemSeamResult[];
  readonly fitReport?: ItemFitReport;
  readonly attachmentProfile?: ItemAttachmentProfileV1;
  readonly busy: boolean;
  readonly error?: string;
  readonly onSelectPart: (field: string) => void;
  readonly onPreview: (preview: "COMPOSED" | "EXPLODED" | "ICON") => void;
  readonly onPartChange: (part: ItemPartDraft) => void;
  readonly onPartScalePreview: (part: ItemPartDraft) => void;
  readonly onUtiNumericChange: (next: ItemUtiNumericDraft) => void;
  readonly onPropertiesChange: (next: ItemPropertyDraft[]) => void;
  readonly onOccupiedNamespaceChange: (next: string) => void;
  readonly onCapartModelPrefixChange: (next: string) => void;
  readonly onCapartGenderCodeChange: (next: string) => void;
  readonly onProofAppearanceRowChange: (next: number) => void;
  readonly onProofRaceChange: (next: number) => void;
  readonly onProofGenderChange: (next: number) => void;
  readonly onProofPhenotypeChange: (next: number) => void;
  readonly onSeamToleranceChange: (next: number) => void;
  readonly onFitTargetLengthsChange: (next: number[]) => void;
  readonly onAutoFit: (targetAxialScaleFactors?: readonly number[]) => void;
  readonly onDiscardFit: () => void;
  readonly onSeams: (next: ItemSeamResult[]) => void;
  readonly onBack: () => void;
  readonly onContinue: () => void;
}) {
  const [nodeNames, setNodeNames] = useState<Record<string, string[]>>({});
  const [showReference, setShowReference] = useState(true);
  const selected = parts.find((part) => part.field === selectedField) ?? parts[0];
  const authorsCustomIconLayers = ["STANDARD", "LAYERED"].includes(
    row.capability.iconProfile,
  );
  const equippedProof = requiresEquippedItemProof(row);
  const resolved = parts.map((part, index) => {
    try {
      return resolveItemPartDraft(row, part, index);
    } catch {
      return undefined;
    }
  });
  const complete = resolved.every(Boolean)
    && Boolean(namespaceAllocation)
    && parts.every(validPartTransform)
    && (!fitReport || parts.every((part) => partMatchesFitContract(part, fitReport)))
    && Number.isFinite(seamTolerance)
    && seamTolerance >= 0
    && (row.modelType === 2
      ? fitReport?.schemaVersion === 4 && fitReport.status === "PASSED"
      : row.capability.meshySourceCount === 0 || (
      fitTargetLengths.length === row.capability.meshySourceCount
      && fitTargetLengths.every((value) => Number.isFinite(value) && value > 0)
      && fitTargetLengths.reduce((sum, value) => sum + value, 0) <= 10
    ))
    && integerInRange(utiNumeric.cost, 0xffff_ffff)
    && integerInRange(utiNumeric.addCost, 0xffff_ffff)
    && integerInRange(utiNumeric.charges, 0xff)
    && integerInRange(utiNumeric.stackSize, 0xffff)
    && integerInRange(utiNumeric.paletteId, 0xff)
    && row.colorFields.every((field) => integerInRange(utiNumeric.colors[field] ?? -1, 0xff))
    && properties.every((property) => (
      integerInRange(property.propertyName, 0xffff)
      && integerInRange(property.subtype, 0xffff)
      && integerInRange(property.costTable, 0xff)
      && integerInRange(property.costValue, 0xffff)
      && integerInRange(property.param1, 0xff)
      && integerInRange(property.param1Value, 0xff)
      && integerInRange(property.chanceAppear, 100)
    ))
    && (row.capability.iconProfile !== "IPRP_SPELL"
      || properties.filter((property) => property.propertyName === 15).length === 1)
    && (!equippedProof
      || (
        integerInRange(proofAppearanceRow, 0xffff)
        && integerInRange(proofRace, 0xff)
        && integerInRange(proofGender, 0xff)
        && integerInRange(proofPhenotype, 0xff)
      ));
  const setNumber = (
    field: "translation" | "rotationDegrees" | "pivot",
    index: number,
    value: string,
  ) => {
    const next = updateTuple(selected[field], index, Number(value));
    onPartChange(field === "rotationDegrees"
      ? { ...selected, rotationDegrees: next, rotationXyzw: eulerDegreesToQuaternion(next) }
      : { ...selected, [field]: next });
  };
  const selectedFitPart = fitReport?.parts.find((part) => part.field === selected.field);
  const fittedUniformScale = selectedFitPart?.transform.uniformScale;
  const selectedSizePercent = itemReferenceScalePercent(
    selected,
    fitReport,
    attachmentProfile,
  );
  const selectedSupportsScaling = itemPartSupportsReferenceScaling(selected, attachmentProfile);
  const manualFitParts = parts.filter((part) => (
    part.sourceKind === "MESHY_GLB" && !partMatchesFitContract(part, fitReport)
  ));
  const hasManualFit = manualFitParts.length > 0;
  const selectedHasManualSize = Boolean(
    selectedFitPart && !partMatchesFitContract(selected, fitReport),
  );
  const setSelectedSizePercent = (nextPercent: number) => {
    if (!fittedUniformScale || !selectedSupportsScaling || !Number.isFinite(nextPercent)) return;
    const clamped = Math.min(200, Math.max(50, nextPercent));
    const fittedPercent = itemReferenceScalePercent(
      { ...selected, uniformScale: fittedUniformScale },
      fitReport,
      attachmentProfile,
    );
    onPartScalePreview({
      ...selected,
      uniformScale: fittedUniformScale * clamped / fittedPercent,
    });
  };
  const targetAxialScaleFactors = parts
    .filter((part) => part.sourceKind === "MESHY_GLB")
    .map((part) => Math.round(
      itemReferenceScalePercent(part, fitReport, attachmentProfile) * 10_000,
    ) / 1_000_000);
  return (
    <section className="item-prepare">
      <header className="item-page-heading">
        <div><p className="eyebrow">Item · Prepare</p><h1>Prepare Item</h1><p>BaseItem {row.baseItem} · {row.label} · {row.capability.compositionProfile}. Meshy sources become independent MDLs; retail selectors stay numeric.</p></div>
        <code>{row.capability.meshySourceCount} source GLB → {row.capability.meshySourceCount} MDL</code>
      </header>
      <div className="item-editor">
        <nav className="item-preview-modes" aria-label="Item preview mode">
          {(["COMPOSED", "EXPLODED", "ICON"] as const).map((mode) => (
            <button key={mode} type="button" data-active={preview === mode} onClick={() => onPreview(mode)}>{mode.toLowerCase()}</button>
          ))}
        </nav>
        <section className="item-composition-stage" data-preview={preview.toLowerCase()}>
          <header><strong>Aurora part assembly</strong><span>shared item origin · no combined MDL</span></header>
          {preview === "ICON" && authorsCustomIconLayers ? (
            <>
              <ItemCompositionViewport
                parts={parts}
                mode="ICON"
                tolerance={seamTolerance}
                referenceProfile={attachmentProfile}
                showReference={false}
                onSeams={onSeams}
                onNodeNames={(field, names) => setNodeNames((current) => (
                  current[field]?.join("\0") === names.join("\0")
                    ? current
                    : { ...current, [field]: names }
                ))}
              />
              <div className="item-part-stack">
                {parts.map((part, index) => {
                  const resource = resolved[index];
                  return (
                    <button
                      key={part.field}
                      type="button"
                      data-selected={part.field === selected.field}
                      style={{ "--part-index": index } as CSSProperties}
                      onClick={() => onSelectPart(part.field)}
                    >
                      <b>{part.token?.toUpperCase() ?? index + 1}</b>
                      <span><strong>{part.label}</strong><small>{part.field} = {part.variant}</small></span>
                      <code>{resource?.iconResref}</code>
                    </button>
                  );
                })}
              </div>
            </>
          ) : preview === "ICON" ? (
            <div className="item-reference-preview">
              <strong>{row.capability.iconProfile}</strong>
              <p>
                The icon is resolved from retail data
                {row.capability.requiredReferenceTables.length > 0
                  ? ` in ${row.capability.requiredReferenceTables.join(" + ")}`
                  : ""}
                . The Studio does not package invented per-selector icon layers.
              </p>
            </div>
          ) : row.capability.meshySourceCount > 0 ? (
            <>
              <ItemCompositionViewport
                parts={parts}
                mode={preview}
                tolerance={seamTolerance}
                referenceProfile={attachmentProfile}
                showReference={showReference}
                onSeams={onSeams}
                onNodeNames={(field, names) => setNodeNames((current) => (
                  current[field]?.join("\0") === names.join("\0")
                    ? current
                    : { ...current, [field]: names }
                ))}
              />
              <div className="item-viewport-part-list">
                {parts.filter((part) => part.sourceKind === "MESHY_GLB").map((part) => (
                  <button key={part.field} type="button" data-selected={part.field === selected.field} onClick={() => onSelectPart(part.field)}>
                    <strong>{part.label}</strong><small>{part.field} = {part.variant} · {part.sourceNode || "default scene"}</small>
                  </button>
                ))}
              </div>
              <details className="item-fit-diagnostics">
                <summary>
                  <strong>Connections</strong>
                  <span data-status={fitReport?.adjacentConnectors.every((connector) => connector.status === "OVERLAPPING") ? "TOUCHING" : "GAP"}>
                    {fitReport?.adjacentConnectors.length ?? 0}/{Math.max(0, parts.length - 1)} connected
                  </span>
                  <span>{attachmentProfile?.attachmentRoute === "HAND" ? "HAND preserved" : attachmentProfile?.attachmentRoute ?? "Reference pending"}</span>
                  <b>Details</b>
                </summary>
                <div className="item-seam-panel">
                {fitReport ? (
                  <>
                    <span data-status={fitReport.status === "PASSED" ? "TOUCHING" : "GAP"}>
                      <b>AUTO-FIT {fitReport.status}</b> {fitReport.algorithm}
                      <code>{fitReport.solutionSha256.slice(0, 16)}...</code>
                    </span>
                    <span data-status={fitReport.orientationFrame.status === "PASSED" ? "TOUCHING" : "GAP"}>
                      <b>FULL FRAME {fitReport.orientationFrame.status}</b> depth X · axial Y · width Z
                      <code>width/depth {fitReport.orientationFrame.widthToDepthRatio.toFixed(3)}</code>
                    </span>
                    {fitReport.adjacentConnectors.map((connector) => (
                      <span key={`${connector.firstField}:${connector.secondField}:connector`} data-status={connector.status === "OVERLAPPING" ? "TOUCHING" : "GAP"}>
                        <b>CONNECTOR {connector.status}</b> {connector.firstField} TOP ↔ {connector.secondField} BOTTOM
                        <code>+Y overlap {connector.axialOverlap.toFixed(4)}</code>
                      </span>
                    ))}
                  </>
                ) : null}
                <label>
                  <span>Authoritative seam tolerance</span>
                  <input aria-label="Seam tolerance" type="number" min="0" step="0.001" value={seamTolerance} onChange={(event) => onSeamToleranceChange(Number(event.currentTarget.value))} />
                </label>
                <fieldset className="item-fit-proportions">
                  {attachmentProfile ? <strong>Reference slot-frame contract</strong> : null}
                  <legend>Axial-length contract · ordered Aurora slots</legend>
                  {attachmentProfile ? attachmentProfile.slots.map((slot) => (
                    <label key={slot.field}>
                      <span>{slot.label}</span>
                      <code>Y {slot.boundsMin[1].toFixed(4)} … {slot.boundsMax[1].toFixed(4)}</code>
                    </label>
                  )) : parts.filter((part) => part.sourceKind === "MESHY_GLB").map((part, index) => (
                    <label key={part.field}>
                      <span>{part.label}</span>
                      <input
                        aria-label={`Target axial length ${part.label}`}
                        type="number"
                        min="0.001"
                        max="10"
                        step="0.01"
                        value={fitTargetLengths[index] ?? 0}
                        onChange={(event) => onFitTargetLengthsChange(fitTargetLengths.map(
                          (value, targetIndex) => targetIndex === index
                            ? Number(event.currentTarget.value)
                            : value,
                        ))}
                      />
                    </label>
                  ))}
                  <small>{attachmentProfile
                    ? `Origin ${attachmentProfile.commonOrigin.join(", ")} · ${attachmentProfile.attachmentRoute}`
                    : `Total composer length: ${fitTargetLengths.reduce((sum, value) => sum + value, 0).toFixed(2)}`}</small>
                  <button type="button" className="button button--secondary" disabled={busy} onClick={() => onAutoFit()}>{attachmentProfile ? "Reapply reference frame" : "Apply proportion contract"}</button>
                </fieldset>
                <small>Preview AABB is advisory. Build recomputes the gate from transformed triangle surfaces.</small>
                {seams.length === 0 ? <small>One part: no inter-part seam.</small> : seams.map((seam) => (
                  <span key={`${seam.firstField}:${seam.secondField}`} data-status={seam.status}>
                    <b>PREVIEW {seam.status}</b> {seam.firstField} ↔ {seam.secondField}
                    <code>{seam.status === "OVERLAP" ? `volume ${seam.overlapVolume.toPrecision(3)}` : `gap ${seam.gap.toPrecision(3)}`}</code>
                  </span>
                ))}
                </div>
              </details>
            </>
          ) : (
            <div className="item-reference-preview">
              <strong>{row.capability.compositionProfile}</strong>
              <p>This composer is resolved by Aurora against {row.capability.requiredReferenceTables.join(" + ")} and creature context. The Studio does not misrepresent its numeric UTI selectors as independent Meshy meshes.</p>
            </div>
          )}
          <footer>
            <span>Composed: runtime resolves all numeric UTI fields</span>
            <strong>
              {preview === "ICON"
                ? authorsCustomIconLayers
                  ? `${parts.length} source-derived 2D layers`
                  : `${row.capability.iconProfile} retail icon resolver`
                : row.capability.meshySourceCount > 0
                  ? `${row.capability.meshySourceCount} independent binary MDLs`
                  : `${parts.length} retail numeric selectors`}
            </strong>
          </footer>
        </section>
        <aside className="item-part-inspector item-fit-panel" aria-label="Selected part fitting">
          <header className="item-fit-panel__header">
            <div><p className="eyebrow">Selected part</p><h2>Part fitting</h2></div>
            <span data-state={hasManualFit ? "dirty" : "valid"}>{hasManualFit ? "Manual fit" : "Validated"}</span>
          </header>
          <p className="item-fit-panel__intro">Scale relative to the Aurora reference slot. Locked ends preserve the hand anchor and part connectors.</p>
          <div className="item-fit-part-tabs" role="tablist" aria-label="Model parts">
            {parts.filter((part) => part.sourceKind === "MESHY_GLB").map((part) => {
              const percent = itemReferenceScalePercent(part, fitReport, attachmentProfile);
              const changed = !partMatchesFitContract(part, fitReport);
              const scalable = itemPartSupportsReferenceScaling(part, attachmentProfile);
              return (
                <button
                  key={part.field}
                  type="button"
                  role="tab"
                  aria-selected={part.field === selected.field}
                  data-selected={part.field === selected.field}
                  onClick={() => onSelectPart(part.field)}
                >
                  <strong>{part.label}</strong>
                  <small>{percent.toFixed(0)}% · {changed ? "changed" : scalable ? "OK" : "reference locked"}</small>
                </button>
              );
            })}
          </div>
          <section className="item-fit-size" aria-label="Selected part size">
            <header><label htmlFor="item-part-size"><strong>Size {selected.label}</strong></label><output>{selectedSizePercent.toFixed(0)}%</output></header>
            <input
              id="item-part-size"
              aria-label="Part size percent"
              type="range"
              min="50"
              max="200"
              step="1"
              disabled={!selectedFitPart || !selectedSupportsScaling || busy}
              value={Math.min(200, Math.max(50, selectedSizePercent))}
              onChange={(event) => setSelectedSizePercent(Number(event.currentTarget.value))}
            />
            <div className="item-part-size-presets" aria-label="Part size presets">
              {[75, 100, 125, 150, 200].map((percent) => (
                <button
                  key={percent}
                  type="button"
                  disabled={!selectedFitPart || !selectedSupportsScaling || busy}
                  data-active={Math.abs(selectedSizePercent - percent) < 0.01}
                  onClick={() => setSelectedSizePercent(percent)}
                >
                  {percent}%
                </button>
              ))}
            </div>
            <div className="item-fit-reference-row">
              <label><input type="checkbox" checked={showReference} onChange={(event) => setShowReference(event.currentTarget.checked)} /> Show 100% reference</label>
              <button type="button" className="button button--secondary" disabled={!selectedFitPart || !selectedSupportsScaling || busy} onClick={() => setSelectedSizePercent(100)}>Reset part</button>
            </div>
            <dl className="item-fit-deltas">
              <div><dt>Length vs auto-fit</dt><dd>{selectedSizePercent >= 100 ? "+" : ""}{(selectedSizePercent - 100).toFixed(0)}%</dd></div>
              <div><dt>Connector anchor</dt><dd>{selectedSupportsScaling ? "Preserved" : "Locked"}</dd></div>
            </dl>
            <p className={selectedHasManualSize ? "item-part-size-warning" : "item-part-size-valid"} role="status">
              <strong>{selectedHasManualSize ? "Manual change pending" : "Fit validated"}</strong>
              {selectedHasManualSize
                ? "Validation will recompute the slot frame, HAND anchor and both connections."
                : selectedSupportsScaling
                  ? "This transform is bound to the validated reference-fit contract."
                  : "This slot has no Aurora extension policy and remains reference locked."}
            </p>
            <div className="item-fit-actions">
              <button type="button" className="button button--secondary" disabled={!hasManualFit || busy} onClick={onDiscardFit}>Discard changes</button>
              <button type="button" className="button button--primary" disabled={!hasManualFit || busy} onClick={() => onAutoFit(targetAxialScaleFactors)}>{busy ? "Validating…" : "Validate fit"}</button>
            </div>
          </section>
          <details className="item-advanced-inspector">
            <summary>Advanced item properties</summary>
            <div className="item-advanced-inspector__body">
          <p className="eyebrow">Selected part</p>
          <h2>{selected.label}</h2>
          <label><span>UTI field</span><input value={selected.field} readOnly /></label>
          {row.modelType === 2 ? (
            <fieldset className="item-weapon-appearance">
              <legend>Aurora weapon-part appearance</legend>
              <label>
                <span>Model</span>
                <input
                  aria-label="Weapon part model"
                  type="number"
                  min="0"
                  max="25"
                  value={selected.weaponModel ?? 0}
                  onChange={(event) => {
                    const weaponModel = Number(event.currentTarget.value);
                    const weaponColor = selected.weaponColor ?? 1;
                    onPartChange({
                      ...selected,
                      weaponModel,
                      weaponColor,
                      variant: encodeWeaponPartAppearance(weaponModel, weaponColor),
                    });
                  }}
                />
              </label>
              <label>
                <span>Color</span>
                <select
                  aria-label="Weapon part color"
                  value={selected.weaponColor ?? 1}
                  onChange={(event) => {
                    const weaponModel = selected.weaponModel ?? 0;
                    const weaponColor = Number(event.currentTarget.value);
                    onPartChange({
                      ...selected,
                      weaponModel,
                      weaponColor,
                      variant: encodeWeaponPartAppearance(weaponModel, weaponColor),
                    });
                  }}
                >
                  <option value="1">Color 1</option>
                  <option value="2">Color 2</option>
                  <option value="3">Color 3</option>
                  <option value="4">Color 4</option>
                </select>
              </label>
              <small>
                Encoded UTI BYTE: {encodeWeaponPartAppearance(
                  selected.weaponModel ?? 0,
                  selected.weaponColor ?? 1,
                )} = model × 10 + color
              </small>
            </fieldset>
          ) : (
            <label><span>Numeric selector (BYTE)</span><input aria-label="Part variant" type="number" min="0" max="255" value={selected.variant} onChange={(event) => onPartChange({ ...selected, variant: Number(event.currentTarget.value) })} /></label>
          )}
          {selected.sourceKind === "MESHY_GLB" ? (
            <>
              <label><span>GLB sourceNode (optional exact default-scene root)</span><input aria-label="Source node" list={`item-source-nodes-${selected.field}`} value={selected.sourceNode} onChange={(event) => onPartChange({ ...selected, sourceNode: event.currentTarget.value })} placeholder="whole default scene" /></label>
              <datalist id={`item-source-nodes-${selected.field}`}>
                {(nodeNames[selected.field] ?? []).map((name) => <option key={name} value={name} />)}
              </datalist>
              <label>
                <span>Aurora texture encoding</span>
                <select aria-label="Texture encoding" value={selected.textureEncoding} onChange={(event) => onPartChange({ ...selected, textureEncoding: event.currentTarget.value as ItemPartDraft["textureEncoding"] })}>
                  <option value="DIRECT_COLOR">Direct-color TGA</option>
                  {row.modelType !== 2 ? (
                    <>
                      <option value="PLT_METAL1">PLT · Metal 1</option>
                      <option value="PLT_METAL2">PLT · Metal 2</option>
                      <option value="PLT_CLOTH1">PLT · Cloth 1</option>
                      <option value="PLT_CLOTH2">PLT · Cloth 2</option>
                      <option value="PLT_LEATHER1">PLT · Leather 1</option>
                      <option value="PLT_LEATHER2">PLT · Leather 2</option>
                    </>
                  ) : null}
                </select>
              </label>
              {row.modelType === 2 ? <small>ModelType 2 colors select concrete MDL/texture variants; they are not standalone PLT fields.</small> : null}
            </>
          ) : (
            <>
              <label><span>Native resolver table</span><input value={selected.referenceTable ?? "CLOAKMODEL"} readOnly /></label>
              <p className="item-reference-note">
                {selected.sourceKind === "CAPART_SELECTION"
                  ? `This BYTE selects a row in ${selected.referenceTable}. CAPART maps ${selected.field} to its native MDLNAME/NODENAME; PARTS_ROBE additionally supplies the 19 hide masks.`
                  : "This BYTE selects one CloakModel row. The exact row-bound MDL and icon must both exist in the supplied resource inventory."}
              </p>
              {selected.sourceKind === "CAPART_SELECTION" ? (
                <fieldset>
                  <legend>Creature model context</legend>
                  <label>
                    <span>Model prefix (for example pmh0)</span>
                    <input
                      aria-label="CAPART model prefix"
                      value={capartModelPrefix}
                      onChange={(event) => onCapartModelPrefixChange(event.currentTarget.value)}
                    />
                  </label>
                  <label>
                    <span>Optional gender-specific code</span>
                    <input
                      aria-label="CAPART gender code"
                      maxLength={1}
                      value={capartGenderCode}
                      onChange={(event) => onCapartGenderCodeChange(event.currentTarget.value)}
                    />
                  </label>
                  <small>Resolver tries `%s%c_%s%03d`, then `%s_%s%03d`, and requires the matching MDL bytes.</small>
                </fieldset>
              ) : null}
              <fieldset>
                <legend>Equipped proof wearer</legend>
                <label>
                  <span>Creature Appearance row</span>
                  <input
                    aria-label="Proof creature Appearance row"
                    type="number"
                    min="0"
                    max="65535"
                    value={proofAppearanceRow}
                    onChange={(event) => onProofAppearanceRowChange(Number(event.currentTarget.value))}
                  />
                </label>
                <label>
                  <span>Race (BYTE)</span>
                  <input
                    aria-label="Proof creature Race"
                    type="number"
                    min="0"
                    max="255"
                    value={proofRace}
                    onChange={(event) => onProofRaceChange(Number(event.currentTarget.value))}
                  />
                </label>
                <label>
                  <span>Gender (BYTE)</span>
                  <input
                    aria-label="Proof creature Gender"
                    type="number"
                    min="0"
                    max="255"
                    value={proofGender}
                    onChange={(event) => onProofGenderChange(Number(event.currentTarget.value))}
                  />
                </label>
                <label>
                  <span>Phenotype (INT)</span>
                  <input
                    aria-label="Proof creature Phenotype"
                    type="number"
                    min="0"
                    max="255"
                    value={proofPhenotype}
                    onChange={(event) => onProofPhenotypeChange(Number(event.currentTarget.value))}
                  />
                </label>
                <small>Build derives the player model prefix from the selected appearance.2da row and requires exact Race, Gender and Phenotype parity.</small>
                <small>The proof MOD equips this UTI on one creature; it does not place the item on the ground.</small>
              </fieldset>
            </>
          )}
          {selected.requiresExplicitResourceResrefs ? (
            <>
              <label><span>Validated MDL resref</span><input aria-label="Explicit model resref" value={selected.explicitModelResref} onChange={(event) => onPartChange({ ...selected, explicitModelResref: event.currentTarget.value })} /></label>
              <label><span>Validated icon resref</span><input aria-label="Explicit icon resref" value={selected.explicitIconResref} onChange={(event) => onPartChange({ ...selected, explicitIconResref: event.currentTarget.value })} /></label>
            </>
          ) : null}
          {selected.sourceKind === "MESHY_GLB" ? (
            <>
              <fieldset><legend>Translation · MDL controller</legend>{["X", "Y", "Z"].map((axis, index) => <label key={axis}><span>{axis}</span><input aria-label={`Translation ${axis}`} type="number" step="0.01" value={selected.translation[index]} onChange={(event) => setNumber("translation", index, event.currentTarget.value)} /></label>)}</fieldset>
              <fieldset><legend>Rotation · MDL controller</legend>{["X", "Y", "Z"].map((axis, index) => <label key={axis}><span>{axis}°</span><input aria-label={`Rotation ${axis}`} type="number" step="1" value={selected.rotationDegrees[index]} onChange={(event) => setNumber("rotationDegrees", index, event.currentTarget.value)} /></label>)}</fieldset>
            </>
          ) : null}
          <fieldset className="item-uti-numeric">
            <legend>Numeric UTI fields</legend>
            <label><span>Cost (DWORD)</span><input aria-label="UTI Cost" type="number" min="0" max="4294967295" value={utiNumeric.cost} onChange={(event) => onUtiNumericChange({ ...utiNumeric, cost: Number(event.currentTarget.value) })} /></label>
            <label><span>AddCost (DWORD)</span><input aria-label="UTI AddCost" type="number" min="0" max="4294967295" value={utiNumeric.addCost} onChange={(event) => onUtiNumericChange({ ...utiNumeric, addCost: Number(event.currentTarget.value) })} /></label>
            <label><span>Charges (BYTE)</span><input aria-label="UTI Charges" type="number" min="0" max="255" value={utiNumeric.charges} onChange={(event) => onUtiNumericChange({ ...utiNumeric, charges: Number(event.currentTarget.value) })} /></label>
            <label><span>StackSize (WORD)</span><input aria-label="UTI StackSize" type="number" min="0" max="65535" value={utiNumeric.stackSize} onChange={(event) => onUtiNumericChange({ ...utiNumeric, stackSize: Number(event.currentTarget.value) })} /></label>
            <label><span>PaletteID (BYTE)</span><input aria-label="UTI PaletteID" type="number" min="0" max="255" value={utiNumeric.paletteId} onChange={(event) => onUtiNumericChange({ ...utiNumeric, paletteId: Number(event.currentTarget.value) })} /></label>
            {row.colorFields.map((field) => (
              <label key={field}>
                <span>{field} (BYTE)</span>
                <input
                  aria-label={`UTI ${field}`}
                  type="number"
                  min="0"
                  max="255"
                  value={utiNumeric.colors[field] ?? 0}
                  onChange={(event) => onUtiNumericChange({
                    ...utiNumeric,
                    colors: {
                      ...utiNumeric.colors,
                      [field]: Number(event.currentTarget.value),
                    },
                  })}
                />
              </label>
            ))}
          </fieldset>
          <fieldset className="item-properties-list">
            <legend>PropertiesList · native GFF fields</legend>
            {properties.map((property, propertyIndex) => (
              <div className="item-property-row" key={propertyIndex}>
                {([
                  ["PropertyName", "propertyName", 0xffff],
                  ["Subtype", "subtype", 0xffff],
                  ["CostTable", "costTable", 0xff],
                  ["CostValue", "costValue", 0xffff],
                  ["Param1", "param1", 0xff],
                  ["Param1Value", "param1Value", 0xff],
                  ["ChanceAppear", "chanceAppear", 100],
                ] as const).map(([label, key, maximum]) => (
                  <label key={key}>
                    <span>{label}</span>
                    <input
                      aria-label={`Property ${propertyIndex + 1} ${label}`}
                      type="number"
                      min="0"
                      max={maximum}
                      value={property[key]}
                      onChange={(event) => onPropertiesChange(properties.map((candidate, index) => (
                        index === propertyIndex
                          ? { ...candidate, [key]: Number(event.currentTarget.value) }
                          : candidate
                      )))}
                    />
                  </label>
                ))}
                <button type="button" className="button button--secondary" onClick={() => onPropertiesChange(properties.filter((_, index) => index !== propertyIndex))}>Remove property</button>
              </div>
            ))}
            <button type="button" className="button button--secondary" onClick={() => onPropertiesChange([...properties, { ...EMPTY_ITEM_PROPERTY }])}>Add property</button>
            {row.capability.iconProfile === "IPRP_SPELL" ? <small>Exactly one PropertyName 15 is required; its Subtype selects IPRP_SPELLS.Icon.</small> : null}
          </fieldset>
          <fieldset className="item-namespace-allocation">
            <legend>Collision-free resource namespace</legend>
            <label>
              <span>Occupied keys · one per line (`type:resref`)</span>
              <textarea
                aria-label="Occupied resource keys"
                value={occupiedNamespace}
                onChange={(event) => onOccupiedNamespaceChange(event.currentTarget.value)}
                placeholder={"2002:sw_b_001\n3:isw_b_001\n2025:m2aitm1\nHAK:m2aihak1"}
              />
            </label>
            {namespaceAllocation ? (
              <div className="item-namespace-report">
                <code>{namespaceAllocation.blueprintResref}.uti</code>
                <code>{namespaceAllocation.hakResref}.hak</code>
                <code>{namespaceAllocation.moduleResref}.mod · Area {namespaceAllocation.areaResref}</code>
                {namespaceAllocation.sourceMaxRange !== null ? (
                  <span>
                    Model selector range {namespaceAllocation.sourceMinRange}..{namespaceAllocation.sourceMaxRange}
                    {namespaceAllocation.baseitemsOverrideRequired
                      ? ` → ${namespaceAllocation.effectiveMaxRange} (baseitems.2da override in HAK)`
                      : " (source range)"}
                  </span>
                ) : null}
                <span>{namespaceAllocation.occupiedKeyCount} occupied keys · {namespaceAllocation.collisionAvoidanceCount} deterministic probes</span>
              </div>
            ) : <small>Namespace allocation is invalid or exhausted.</small>}
          </fieldset>
          <div className="item-transform-contract">
            <span>1</span><p><strong>Studio transform</strong><small>per part</small></p>
            <span>2</span><p><strong>MDL controllers</strong><small>translation + rotation</small></p>
            <span>3</span><p><strong>UTI stays numeric</strong><small>{selected.field} = {selected.variant}</small></p>
          </div>
            </div>
          </details>
          {error ? <p className="item-error" role="alert">{error}</p> : null}
        </aside>
      </div>
      <footer className="item-action-bar">
        <button type="button" className="button button--secondary" onClick={onBack}>Back</button>
        <p><strong>{hasManualFit ? `${manualFitParts.length} manual change${manualFitParts.length === 1 ? "" : "s"} require validation.` : complete ? "Part recipe is ready." : "Complete the Aurora-validated item contract."}</strong><span>{selected.label}: {selectedSizePercent.toFixed(0)}% · camera and HAND anchor remain reference-bound.</span></p>
        <button type="button" className="button button--primary" disabled={!complete} onClick={onContinue}>Continue to Build</button>
      </footer>
    </section>
  );
}

function ItemBuild({
  row,
  parts,
  busy,
  error,
  onBack,
  onBuild,
}: {
  readonly row: ItemBaseItemRow;
  readonly parts: readonly ResolvedItemPartDraft[];
  readonly busy: boolean;
  readonly error?: string;
  readonly onBack: () => void;
  readonly onBuild: () => void;
}) {
  const hasMeshyGeometry = row.capability.meshySourceCount > 0;
  const authorsCustomIconLayers = ["STANDARD", "LAYERED"].includes(
    row.capability.iconProfile,
  );
  return (
    <section className="item-build">
      <header className="item-page-heading"><div><p className="eyebrow">Item · Build</p><h1>Build the resolved item resources</h1><p>{hasMeshyGeometry ? "Each Meshy source runs through the Item MDL profile. UTI, source-derived icons and HAK follow aggregate gates." : "Retail selectors stay numeric in the UTI; reference tables resolve geometry and icons without fabricated Meshy artifacts."}</p></div></header>
      <div className="item-build-grid">
        {[
          ["1", "Resolve BaseItem", `ModelType ${row.modelType} · ${parts.length} fields`],
          ["2", hasMeshyGeometry ? "Convert each Meshy part" : "Resolve retail selectors", hasMeshyGeometry ? "GLB → shared IR → Item MDL profile" : `${row.capability.compositionProfile} via ${row.capability.requiredReferenceTables.join(" + ")}`],
          ["3", hasMeshyGeometry ? "Segment streams" : "Validate selector domain", hasMeshyGeometry ? "≤ 65,535 indices per stream · no triangle deletion" : "UTI BYTE fields · no independent MDL emission"],
          ["4", "Validate item budget", "Σ all part triangles ≤ 300,000"],
          ["5", "Write UTI + icons", authorsCustomIconLayers ? "numeric fields · source-derived icon layers" : `${row.capability.iconProfile} retail icon resolution`],
          ["6", "Package HAK + MOD", hasMeshyGeometry ? "MDL + texture + icon + UTI + test module · own readback" : "UTI + test module · retail resources remain references"],
        ].map(([index, title, detail]) => <article key={index}><b>{index}</b><div><strong>{title}</strong><span>{detail}</span></div></article>)}
      </div>
      <section className="panel item-build-resources">
        {parts.map((part) => (
          <div key={part.field}>
            <strong>{part.field} = {part.variant}</strong>
            {part.sourceKind === "MESHY_GLB" ? (
              <>
                <code>{part.modelResref}.mdl</code>
                <code>{authorsCustomIconLayers ? `${part.iconResref}.tga` : row.capability.iconProfile}</code>
                {part.weaponColorways.length > 0 ? (
                  <small>{part.weaponColorways.length} concrete MDL/texture colorways · one Meshy geometry source</small>
                ) : null}
              </>
            ) : (
              <>
                <code>{part.sourceKind}</code>
                <code>{part.referenceTable ?? row.capability.requiredReferenceTables.join(" + ")}</code>
              </>
            )}
          </div>
        ))}
      </section>
      {error ? <p className="item-error" role="alert">{error}</p> : null}
      <footer className="item-action-bar">
        <button type="button" className="button button--secondary" disabled={busy} onClick={onBack}>Back</button>
        <p><strong>{busy ? "Building locally in the canonical Worker…" : "Ready for the independent Item build."}</strong><span>No Toolset or NWN session is started.</span></p>
        <button type="button" className="button button--primary" disabled={busy} onClick={onBuild}>{busy ? "Building…" : "Build Item package"}</button>
      </footer>
    </section>
  );
}

function ItemReview({
  snapshot,
  onDownload,
  onError,
}: {
  readonly snapshot: ItemBuildSnapshot;
  readonly onDownload: () => void;
  readonly onError: (message: string) => void;
}) {
  return (
    <section className="item-review">
      <header className="item-page-heading"><div><p className="eyebrow">Item · Review Output</p><h1>Verify every part, UTI field and icon layer</h1><p>Offline package admission passed. Visual composition remains explicitly untested.</p></div><span className="status-badge status-badge--neutral">OWNER PROOF PENDING</span></header>
      <div className="item-review-summary">
        <article><strong>{snapshot.report.meshyPartCount}</strong><span>authored Meshy MDL parts</span></article>
        <article><strong>{snapshot.report.referenceSelectorCount}</strong><span>retail numeric selectors</span></article>
        <article><strong>{snapshot.report.iconLayerCount}</strong><span>authored 2D icon layers</span></article>
        {snapshot.report.weaponColorwayCoverage.status === "COMPLETE" ? (
          <article><strong>{snapshot.report.weaponColorwayCoverage.emittedResourceCount}</strong><span>concrete weapon colorways (1..4)</span></article>
        ) : null}
        {snapshot.report.itemPropertiesModelConformance.status === "PASSED" ? (
          <article><strong>{snapshot.report.itemPropertiesModelConformance.checkedMdlCount}</strong><span>MDLs: controllerless root + Trimesh transforms</span></article>
        ) : null}
        {snapshot.report.itemIconConformance.status === "PASSED" ? (
          <article><strong>{snapshot.report.itemIconConformance.colorways.length}</strong><span>native layered icon composites passed</span></article>
        ) : null}
        <article><strong>{snapshot.utiReport.partCount}</strong><span>numeric UTI part fields</span></article>
        <article><strong>{snapshot.report.triangleBudget.triangleCount.toLocaleString()}</strong><span>/ 300,000 triangles</span></article>
      </div>
      <section className="panel item-readback-table">
        <header><h2>Item resource readback</h2><span>No composed-model artifact exists</span></header>
        {snapshot.partReadbacks.map((part) => (
          <div key={`${part.field}:${part.variant}`}>
            <b>PASS</b><strong>{part.field} = {part.variant}</strong><code>{part.modelResref}.mdl</code><span>{part.readback.nodeTree?.nodeCount ?? "?"} nodes</span>
          </div>
        ))}
        {snapshot.report.referenceSelectorCount > 0 ? (
          <div><b>PASS</b><strong>Retail selector fields</strong><code>{snapshot.report.referenceSelectorCount} numeric BYTE values</code><span>no fabricated MDL/icon payloads</span></div>
        ) : null}
        <div><b>PASS</b><strong>UTI numeric readback</strong><code>BaseItem {snapshot.utiReport.baseItem} · ModelType {snapshot.utiReport.modelType}</code><span>{snapshot.utiReport.semanticReadbackStatus}</span></div>
      </section>
      {snapshot.report.itemPropertiesModelConformance.status === "PASSED" ? (
        <section className="panel item-seam-readback">
          <header><h2>Aurora ModelType 2 append contract</h2><span>{snapshot.report.itemPropertiesModelConformance.algorithm}</span></header>
          {snapshot.report.itemPropertiesModelConformance.colorways.map((colorway) => (
            <div key={colorway.color}>
              <b>PASS</b>
              <strong>Color {colorway.color} · {snapshot.report.itemPropertiesModelConformance.appendOrder.join(" → ")}</strong>
              <code>{colorway.parts.map((part) => `${part.field}:${part.transformControllerOwner}`).join(" · ")}</code>
            </div>
          ))}
        </section>
      ) : null}
      <ItemIconArtifactPreview artifacts={snapshot.artifacts} />
      <section className="panel item-seam-readback">
        <header><h2>Persisted seam gate</h2><span>tolerance {snapshot.report.seamValidation.tolerance}</span></header>
        {snapshot.report.seamValidation.results.length === 0 ? (
          <p>No inter-part Meshy seam is required for this composer.</p>
        ) : snapshot.report.seamValidation.results.map((seam) => (
          <div key={`${seam.firstField}:${seam.secondField}`}>
            <b>PASS</b>
            <strong>{seam.firstField} ↔ {seam.secondField}</strong>
            <code>gap={seam.gap.toPrecision(3)} · overlap={String(seam.overlap)} · {seam.measurementSha256.slice(0, 12)}</code>
          </div>
        ))}
      </section>
      <section className="item-proof-boundary panel">
        <strong>Not ready for owner proof</strong>
        <p>{snapshot.report.proofBlocker}</p>
        <code>{snapshot.report.iconLayerMode} · iconRuntimeParity={snapshot.report.iconRuntimeParity}</code>
        <code>modelVisibility=not_tested · proofCompleteness=missing</code>
      </section>
      <ArtifactDownloads artifacts={snapshot.artifacts} onError={onError} />
      <footer className="item-action-bar"><p><strong>Offline Item package complete.</strong><span>The app does not claim Toolset/NWN visibility.</span></p><button type="button" className="button button--primary" onClick={onDownload}>Continue to Download</button></footer>
    </section>
  );
}

export function ItemWorkflow({ onTargetChange, client, meshyBridge }: ItemWorkflowProps) {
  const workerRef = useRef<ItemWorkerClient | undefined>(client);
  const requestEpoch = useRef(0);
  const [step, setStep] = useState<WorkflowStep>("SOURCE");
  const [baseitems, setBaseitems] = useState<File>();
  const [catalog, setCatalog] = useState<ItemBaseItemsCatalog>();
  const [selectedBaseItem, setSelectedBaseItem] = useState<number>();
  const [parts, setParts] = useState<ItemPartDraft[]>([]);
  const [generationProvenance, setGenerationProvenance] = useState<Record<string, MeshyArtifactProvenance>>({});
  const [generationSession, setGenerationSession] = useState<ItemGenerationSessionV1>();
  const [referenceTables, setReferenceTables] = useState<Record<string, File>>({});
  const [referenceResources, setReferenceResources] = useState<File[]>([]);
  const [referenceModels, setReferenceModels] = useState<Record<string, ItemReferenceModelDraft>>({});
  const [attachmentProfile, setAttachmentProfile] = useState<ItemAttachmentProfileV1>();
  const [attachmentProfileJson, setAttachmentProfileJson] = useState<string>();
  const [properties, setProperties] = useState<ItemPropertyDraft[]>([]);
  const [occupiedNamespace, setOccupiedNamespace] = useState("");
  const [capartModelPrefix, setCapartModelPrefix] = useState("pmh0");
  const [capartGenderCode, setCapartGenderCode] = useState("");
  const [proofAppearanceRow, setProofAppearanceRow] = useState(6);
  const [proofRace, setProofRace] = useState(11);
  const [proofGender, setProofGender] = useState(0);
  const [proofPhenotype, setProofPhenotype] = useState(0);
  const [selectedField, setSelectedField] = useState("");
  const [preview, setPreview] = useState<"COMPOSED" | "EXPLODED" | "ICON">("COMPOSED");
  const [seamTolerance, setSeamTolerance] = useState(0.01);
  const [fitTargetLengths, setFitTargetLengths] = useState<number[]>([1.0]);
  const [seams, setSeams] = useState<ItemSeamResult[]>([]);
  const [fitReport, setFitReport] = useState<ItemFitReport>();
  const [utiNumeric, setUtiNumeric] = useState<ItemUtiNumericDraft>(EMPTY_UTI_NUMERIC);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [build, setBuild] = useState<ItemBuildSnapshot>();

  useEffect(() => {
    const activeClient = client ?? new StudioWorkerClient();
    workerRef.current = activeClient;
    return () => {
      requestEpoch.current += 1;
      if (workerRef.current === activeClient) workerRef.current = undefined;
      if (!client) activeClient.dispose?.();
    };
  }, [client]);

  const selected = catalog?.rows.find((row) => row.baseItem === selectedBaseItem);
  const namespaceAllocation = useMemo(() => {
    if (!selected) return undefined;
    try {
      return allocateItemNamespace(
        selected,
        parts,
        occupiedNamespace.split(/[\s,]+/).filter(Boolean),
      );
    } catch {
      return undefined;
    }
  }, [occupiedNamespace, parts, selected]);
  const resolvedParts = namespaceAllocation?.parts ?? [];
  const maxStep = build
    ? "DOWNLOAD"
    : step === "REVIEW"
      ? "REVIEW"
      : step;
  const visited = WORKFLOW_STEPS.filter((candidate) => stepIndex(candidate) <= stepIndex(maxStep));
  const completed = WORKFLOW_STEPS.filter((candidate) => stepIndex(candidate) < stepIndex(step));
  const blocked = WORKFLOW_STEPS.filter((candidate) => stepIndex(candidate) > stepIndex(maxStep));

  const selectRow = (baseItem: number) => {
    const row = catalog?.rows.find((candidate) => candidate.baseItem === baseItem);
    if (!row) return;
    const next = initialItemPartDrafts(row);
    setSelectedBaseItem(baseItem);
    setParts(next);
    setGenerationProvenance({});
    setGenerationSession(undefined);
    setReferenceResources([]);
    setReferenceModels({});
    setAttachmentProfile(undefined);
    setAttachmentProfileJson(undefined);
    setUtiNumeric(initialUtiNumeric(row));
    setProperties(row.capability.iconProfile === "IPRP_SPELL"
      ? [{ ...EMPTY_ITEM_PROPERTY, propertyName: 15 }]
      : []);
    setProofAppearanceRow(6);
    setProofRace(11);
    setProofGender(0);
    setProofPhenotype(0);
    setSelectedField(next[0]?.field ?? "");
    setSeams([]);
    setFitReport(undefined);
    setFitTargetLengths(initialFitTargetLengths(row));
    setBuild(undefined);
    setError(undefined);
  };

  const inspectBaseitems = (file: File) => {
    const worker = workerRef.current;
    if (!worker) {
      setError("The local Item Worker is still initializing.");
      return;
    }
    const epoch = ++requestEpoch.current;
    if (file.name.toLowerCase() !== "baseitems.2da") {
      setError("Select the exact baseitems.2da reference table.");
      return;
    }
    setBusy(true);
    setError(undefined);
    setCatalog(undefined);
    setSelectedBaseItem(undefined);
    setParts([]);
    setGenerationProvenance({});
    setGenerationSession(undefined);
    setReferenceTables({});
    setReferenceResources([]);
    setReferenceModels({});
    setAttachmentProfile(undefined);
    setAttachmentProfileJson(undefined);
    setProperties([]);
    setOccupiedNamespace("");
    setCapartModelPrefix("pmh0");
    setCapartGenderCode("");
    setProofAppearanceRow(6);
    setProofRace(11);
    setProofGender(0);
    setProofPhenotype(0);
    setSeams([]);
    setFitReport(undefined);
    setFitTargetLengths([1.0]);
    setUtiNumeric(EMPTY_UTI_NUMERIC);
    void file.arrayBuffer()
      .then((bytes) => worker.request({
        requestId: id(),
        type: "INSPECT_ITEM_BASEITEMS",
        baseitemsTwoDa: bytes,
      }, [bytes]))
      .then((response) => {
        if (epoch !== requestEpoch.current) return;
        if (!response.ok || response.type !== "ITEM_BASEITEMS_INSPECTED") {
          throw new Error("Unexpected Item catalog response");
        }
        const next = projectItemBaseitemsCatalog(response.catalogJson);
        setBaseitems(file);
        setCatalog(next);
        if (next.rows[0]) {
          const drafts = initialItemPartDrafts(next.rows[0]);
          setSelectedBaseItem(next.rows[0].baseItem);
          setParts(drafts);
          setUtiNumeric(initialUtiNumeric(next.rows[0]));
          setProperties(next.rows[0].capability.iconProfile === "IPRP_SPELL"
            ? [{ ...EMPTY_ITEM_PROPERTY, propertyName: 15 }]
            : []);
          setSelectedField(drafts[0]?.field ?? "");
          setSeams([]);
          setFitReport(undefined);
          setFitTargetLengths(initialFitTargetLengths(next.rows[0]));
        }
      })
      .catch((reason: unknown) => {
        if (epoch === requestEpoch.current) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      })
      .finally(() => {
        if (epoch === requestEpoch.current) setBusy(false);
      });
  };

  const updatePart = (next: ItemPartDraft) => {
    setParts((current) => current.map((part) => part.field === next.field ? next : part));
    setSeams([]);
    setFitReport(undefined);
    setBuild(undefined);
    setError(undefined);
  };

  const updatePartScalePreview = (next: ItemPartDraft) => {
    setParts((current) => current.map((part) => part.field === next.field ? next : part));
    setSeams([]);
    setBuild(undefined);
    setError(undefined);
  };

  const updateReferenceModel = (field: string, file: File) => {
    const modelResref = file.name.replace(/\.mdl$/i, "").toLowerCase();
    if (!file.name.toLowerCase().endsWith(".mdl") || !/^[a-z0-9_-]{1,16}$/.test(modelResref)) {
      setError("Reference model filename must be a valid 1..16 character Aurora .mdl resref.");
      return;
    }
    setReferenceModels((current) => ({
      ...current,
      [field]: { field, modelResref, file },
    }));
    setAttachmentProfile(undefined);
    setAttachmentProfileJson(undefined);
    setFitReport(undefined);
    setBuild(undefined);
    setError(undefined);
  };

  const startAutoFit = (targetAxialScaleFactors?: readonly number[]) => {
    const worker = workerRef.current;
    const fitParts = resolvedParts.filter((part) => part.sourceKind === "MESHY_GLB");
    if (!worker || !selected || fitParts.length !== selected.capability.meshySourceCount || fitParts.some((part) => !part.file)) {
      setError("Every resolved Meshy Item slot requires a GLB before deterministic auto-fit.");
      return;
    }
    if (selected.modelType === 2 && (
      !baseitems
      || selected.partSlots.some((slot) => !referenceModels[slot.field])
    )) {
      setError("ModelType 2 requires one exact reference MDL for Bottom, Middle and Top before fitting.");
      return;
    }
    if (selected.modelType === 2 && targetAxialScaleFactors && (
      targetAxialScaleFactors.length !== fitParts.length
      || targetAxialScaleFactors.some((value) => !Number.isFinite(value) || value < 0.5 || value > 2)
    )) {
      setError("Reference-relative part scales must contain one value from 50% through 200% per Meshy slot.");
      return;
    }
    if (selected.modelType !== 2 && (
      fitTargetLengths.length !== fitParts.length
      || fitTargetLengths.some((value) => !Number.isFinite(value) || value <= 0)
      || fitTargetLengths.reduce((sum, value) => sum + value, 0) > 10
    )) {
      setError("Target axial lengths must contain one positive value per Meshy part and total at most 10.");
      return;
    }
    const epoch = ++requestEpoch.current;
    setBusy(true);
    setError(undefined);
    void (async () => {
      let currentProfileJson = attachmentProfileJson;
      if (selected.modelType === 2 && !currentProfileJson) {
        const baseitemsTwoDa = await baseitems!.arrayBuffer();
        const models = await Promise.all(selected.partSlots.map(async (slot) => {
          const reference = referenceModels[slot.field];
          return {
            field: slot.field,
            modelResref: reference.modelResref,
            bytes: await reference.file.arrayBuffer(),
          };
        }));
        const profileResponse = await worker.request({
          requestId: id(),
          type: "BUILD_ITEM_ATTACHMENT_PROFILE",
          baseitemsTwoDa,
          baseItem: selected.baseItem,
          referenceKind: "EXPLICIT_VARIANTS",
          referenceId: models.map((model) => model.modelResref).join("/"),
          models,
        }, [baseitemsTwoDa, ...models.map((model) => model.bytes)]);
        if (!profileResponse.ok || profileResponse.type !== "ITEM_ATTACHMENT_PROFILE_BUILT") {
          throw new Error("Unexpected Item attachment profile response");
        }
        currentProfileJson = profileResponse.attachmentProfileJson;
        const profile = JSON.parse(currentProfileJson) as ItemAttachmentProfileV1;
        if (
          profile.schemaVersion !== 1
          || profile.algorithm !== "AURORA_ITEM_REFERENCE_PROFILE_V1"
          || profile.identity.baseItem !== selected.baseItem
          || profile.slots.length !== selected.partSlots.length
        ) {
          throw new Error("Reference attachment profile does not match the selected BaseItem.");
        }
        setAttachmentProfile(profile);
        setAttachmentProfileJson(currentProfileJson);
      }
      const requestParts = await Promise.all(fitParts.map(async (part) => ({
        field: part.field,
        modelResref: part.modelResref,
        sourceGlb: await part.file!.arrayBuffer(),
        sourceNode: part.sourceNode.trim() || null,
      })));
      return worker.request({
        requestId: id(),
        type: "FIT_ITEM_PARTS",
        tolerance: seamTolerance,
        targetAxialLengths: selected.modelType === 2 ? undefined : fitTargetLengths,
        targetAxialScaleFactors: selected.modelType === 2 && targetAxialScaleFactors
          ? [...targetAxialScaleFactors]
          : undefined,
        attachmentProfileJson: currentProfileJson,
        parts: requestParts,
      }, requestParts.map((part) => part.sourceGlb));
    })().then((response) => {
      if (epoch !== requestEpoch.current) return;
      if (!response.ok || response.type !== "ITEM_PARTS_FITTED") {
        throw new Error("Unexpected Item auto-fit response");
      }
      const report = JSON.parse(response.fitReportJson) as ItemFitReport;
      const expectsReferenceFit = selected.modelType === 2;
      if (
        (expectsReferenceFit
          ? report.schemaVersion !== 4 || report.algorithm !== "ITEM_REFERENCE_SLOT_FRAME_FIT_V1"
          : report.schemaVersion !== 3 || report.algorithm !== "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX")
        || report.parts.length !== fitParts.length
      ) {
        throw new Error("Item auto-fit report does not match the resolved slot count.");
      }
      const transforms = new Map(report.parts.map((part) => [part.field, part]));
      setParts((current) => current.map((part) => {
        if (part.sourceKind !== "MESHY_GLB") return part;
        const fitted = transforms.get(part.field);
        if (!fitted || fitted.sourceNode !== (part.sourceNode.trim() || null)) {
          throw new Error(`${part.field} auto-fit binding differs from the active sourceNode.`);
        }
        return {
          ...part,
          translation: fitted.transform.translation,
          rotationXyzw: fitted.transform.rotationXyzw,
          rotationDegrees: quaternionToEulerDegrees(fitted.transform.rotationXyzw),
          uniformScale: fitted.transform.uniformScale,
          pivot: fitted.transform.pivot,
          targetSpaceScaleXyz: fitted.targetSpaceScaleXyz,
        };
      }));
      setFitReport(report);
      if (selected.modelType === 2 && report.parts.length === 3) {
        setSelectedField(report.parts[2].field);
      }
      setSeams(report.adjacentSeams.map((seam) => ({
        firstField: seam.firstField,
        secondField: seam.secondField,
        status: seam.status,
        gap: seam.gap,
        overlapVolume: seam.overlap ? 1 : 0,
      })));
      setBuild(undefined);
      setStep("INSPECT");
      if (report.status !== "PASSED") {
        setError("Auto-fit proposed deterministic transforms, but the authoritative seam gate requires manual adjustment.");
      }
    }).catch((reason: unknown) => {
      if (epoch === requestEpoch.current) setError(reason instanceof Error ? reason.message : String(reason));
    }).finally(() => {
      if (epoch === requestEpoch.current) setBusy(false);
    });
  };

  const discardFitPreview = () => {
    if (!fitReport) return;
    const fittedByField = new Map(fitReport.parts.map((part) => [part.field, part]));
    setParts((current) => current.map((part) => {
      if (part.sourceKind !== "MESHY_GLB") return part;
      const fitted = fittedByField.get(part.field);
      if (!fitted) return part;
      return {
        ...part,
        translation: fitted.transform.translation,
        rotationXyzw: fitted.transform.rotationXyzw,
        rotationDegrees: quaternionToEulerDegrees(fitted.transform.rotationXyzw),
        uniformScale: fitted.transform.uniformScale,
        pivot: fitted.transform.pivot,
        targetSpaceScaleXyz: fitted.targetSpaceScaleXyz,
      };
    }));
    setSeams(fitReport.adjacentSeams.map((seam) => ({
      firstField: seam.firstField,
      secondField: seam.secondField,
      status: seam.status,
      gap: seam.gap,
      overlapVolume: seam.overlap ? 1 : 0,
    })));
    setBuild(undefined);
    setError(undefined);
  };

  const startBuild = () => {
    if (
      !baseitems
      || !selected
      || resolvedParts.length !== parts.length
      || parts.some((part) => part.sourceKind === "MESHY_GLB" && !part.file)
      || (selected.modelType === 2 && (!attachmentProfileJson || fitReport?.schemaVersion !== 4))
      || selected.capability.requiredReferenceTables.some(
        (tableName) => !referenceTables[tableName.toUpperCase()],
      )
      || (
        requiresEquippedItemProof(selected)
        && (
          !namespaceAllocation?.proofCreatureResref
          || !integerInRange(proofAppearanceRow, 0xffff)
          || !integerInRange(proofRace, 0xff)
          || !integerInRange(proofGender, 0xff)
          || !integerInRange(proofPhenotype, 0xff)
        )
      )
    ) return;
    const worker = workerRef.current;
    if (!worker) {
      setError("The local Item Worker is still initializing.");
      return;
    }
    const epoch = ++requestEpoch.current;
    setBusy(true);
    setError(undefined);
    if (!namespaceAllocation) {
      setBusy(false);
      setError("The Item resource namespace is invalid or exhausted.");
      return;
    }
    const {
      blueprintResref,
      hakResref,
      moduleResref,
      areaResref,
      proofCreatureResref,
    } = namespaceAllocation;
    void (async () => {
      const baseitemsTwoDa = await baseitems.arrayBuffer();
      const requestParts = await Promise.all(resolvedParts.map(async (part) => ({
        field: part.field,
        variant: part.variant,
        sourceKind: part.sourceKind,
        modelResref: part.modelResref,
        iconResref: part.iconResref,
        textureResref: part.textureResref,
        weaponColorways: [...part.weaponColorways],
        sourceGlb: part.sourceKind === "MESHY_GLB"
          ? await part.file!.arrayBuffer()
          : undefined,
        sourceNode: part.sourceNode.trim() || null,
        textureEncoding: part.textureEncoding,
        transformJson: JSON.stringify({
          translation: part.translation,
          rotationXyzw: part.rotationXyzw,
          uniformScale: part.uniformScale,
          pivot: part.pivot,
        }),
        targetSpaceScaleXyz: [...part.targetSpaceScaleXyz] as [number, number, number],
      })));
      const requestReferenceTables = await Promise.all(
        selected.capability.requiredReferenceTables.map(async (tableName) => ({
          tableName: tableName.toUpperCase(),
          fileName: referenceTables[tableName.toUpperCase()].name,
          bytes: await referenceTables[tableName.toUpperCase()].arrayBuffer(),
        })),
      );
      const manifestFiles = referenceResources.filter(
        (file) => file.name.toLowerCase().endsWith(".json"),
      );
      const resourceFiles = referenceResources.filter(
        (file) => /\.(?:mdl|plt)$/i.test(file.name),
      );
      if (manifestFiles.length > 1) {
        throw new Error("Select exactly one retail resource manifest JSON.");
      }
      const requestReferenceResourceManifest = manifestFiles[0]
        ? {
            fileName: manifestFiles[0].name,
            bytes: await manifestFiles[0].arrayBuffer(),
          }
        : null;
      const requestReferenceResources = await Promise.all(resourceFiles.map(async (file) => {
        const lowerName = file.name.toLowerCase();
        const extension = lowerName.endsWith(".mdl") ? ".mdl" : ".plt";
        return {
          resourceType: extension === ".mdl" ? 2002 as const : 6 as const,
          resref: lowerName.slice(0, -extension.length),
          fileName: file.name,
          bytes: await file.arrayBuffer(),
        };
      }));
      const blueprintJson = JSON.stringify({
        schemaVersion: 1,
        templateResref: blueprintResref,
        tag: blueprintResref.toUpperCase(),
        localizedName: selected.label,
        description: `Generated Item BaseItem ${selected.baseItem}`,
        identifiedDescription: `Generated Item BaseItem ${selected.baseItem}`,
        comment: "Generated by Meshy2Aurora Item V1.",
        parts: requestParts.map((part) => ({ field: part.field, value: part.variant })),
        properties,
        colors: {
          leather1Color: selected.colorFields.includes("Leather1Color") ? utiNumeric.colors.Leather1Color : null,
          leather2Color: selected.colorFields.includes("Leather2Color") ? utiNumeric.colors.Leather2Color : null,
          cloth1Color: selected.colorFields.includes("Cloth1Color") ? utiNumeric.colors.Cloth1Color : null,
          cloth2Color: selected.colorFields.includes("Cloth2Color") ? utiNumeric.colors.Cloth2Color : null,
          metal1Color: selected.colorFields.includes("Metal1Color") ? utiNumeric.colors.Metal1Color : null,
          metal2Color: selected.colorFields.includes("Metal2Color") ? utiNumeric.colors.Metal2Color : null,
        },
        cost: utiNumeric.cost,
        addCost: utiNumeric.addCost,
        charges: utiNumeric.charges,
        stackSize: utiNumeric.stackSize,
        paletteId: utiNumeric.paletteId,
        identified: true,
        stolen: false,
        cursed: false,
        plot: false,
      });
      const transfer = [
        baseitemsTwoDa,
        ...requestReferenceTables.map((table) => table.bytes),
        ...(requestReferenceResourceManifest
          ? [requestReferenceResourceManifest.bytes]
          : []),
        ...requestReferenceResources.map((resource) => resource.bytes),
        ...requestParts.flatMap((part) => part.sourceGlb ? [part.sourceGlb] : []),
      ];
      const generatedRequestParts = requestParts.filter(
        (part) => part.sourceKind === "MESHY_GLB",
      );
      const generationReady = Boolean(generationSession)
        && generatedRequestParts.length > 0
        && generatedRequestParts.every((part) => generationProvenance[part.field]);
      const generationArtifactsJson = generationReady
        ? JSON.stringify({
            schemaVersion: 1,
            artifacts: generatedRequestParts.map((part) => {
              const provenance = generationProvenance[part.field];
              return {
                field: part.field,
                profileId: provenance.profileId,
                bridgeProtocolVersion: provenance.bridgeProtocolVersion,
                sha256: provenance.sha256,
                byteLength: provenance.byteLength,
                taskIds: { PREVIEW: provenance.taskIds.PREVIEW },
                consumedCredits: provenance.consumedCredits,
                createdAt: provenance.createdAt,
                finishedAt: provenance.finishedAt,
              };
            }),
          })
        : null;
      return worker.request({
        requestId: id(),
        type: "BUILD_ITEM_PACKAGE",
        baseitemsTwoDa,
        baseItem: selected.baseItem,
        hakResref,
        hakFileName: `${hakResref}.hak`,
        moduleResref,
        moduleFileName: `${moduleResref}.mod`,
        moduleName: `Meshy2Aurora Item ${selected.baseItem} candidate`,
        areaResref,
        areaName: "Meshy2Aurora Item Assembly Proof",
        blueprintResref,
        blueprintJson,
        generationSessionJson: generationReady && generationSession
          ? serializeItemGenerationSession(generationSession)
          : null,
        generationArtifactsJson,
        fitReportJson: fitReport ? JSON.stringify(fitReport) : null,
        attachmentProfileJson: attachmentProfileJson ?? null,
        occupiedResourceKeys: occupiedNamespace
          .split(/[\s,]+/)
          .map((key) => key.trim().toLowerCase())
          .filter((key) => /^(?:[0-9]+|hak|mod):[a-z0-9_]{1,16}$/.test(key)),
        seamValidation: {
          tolerance: seamTolerance,
        },
        referenceTables: requestReferenceTables,
        referenceResources: requestReferenceResources,
        referenceResourceManifest: requestReferenceResourceManifest,
        capartContext: selected.capability.iconProfile === "CAPART_COMPOSITE"
          ? {
              schemaVersion: 1,
              modelPrefix: capartModelPrefix.trim().toLowerCase(),
              genderCode: capartGenderCode.trim().toLowerCase() || null,
            }
          : null,
        equippedProofContext: requiresEquippedItemProof(selected)
          ? {
              creatureResref: proofCreatureResref!,
              appearanceRow: proofAppearanceRow,
              race: proofRace,
              gender: proofGender,
              phenotype: proofPhenotype,
            }
          : null,
        parts: requestParts,
      }, transfer);
    })().then((response) => {
      if (epoch !== requestEpoch.current) return;
      if (!response.ok || response.type !== "ITEM_PACKAGE_BUILT") {
        throw new Error("Unexpected Item build response");
      }
      setBuild({
        report: JSON.parse(response.reportJson),
        partReadbacks: JSON.parse(response.partReadbacksJson),
        utiReport: JSON.parse(response.utiReportJson),
        artifacts: response.artifacts,
      });
      setStep("REVIEW");
    }).catch((reason: unknown) => {
      if (epoch === requestEpoch.current) {
        setError(reason instanceof Error ? reason.message : String(reason));
      }
    }).finally(() => {
      if (epoch === requestEpoch.current) setBusy(false);
    });
  };

  const contents = step === "SOURCE" ? (
    <ItemSource
      onTargetChange={onTargetChange}
      meshyBridge={meshyBridge}
      baseitems={baseitems}
      catalog={catalog}
      selected={selected}
      parts={parts}
      busy={busy}
      error={error}
      onBaseitems={inspectBaseitems}
      onSelectRow={selectRow}
      referenceTables={referenceTables}
      onReferenceTable={(tableName, file) => {
        if (!file.name.toLowerCase().endsWith(".2da")) {
          setError(`${tableName} requires a .2da reference table.`);
          return;
        }
        setReferenceTables((current) => ({
          ...current,
          [tableName.toUpperCase()]: file,
        }));
        setBuild(undefined);
        setError(undefined);
      }}
      referenceResources={referenceResources}
      referenceModels={referenceModels}
      attachmentProfile={attachmentProfile}
      onReferenceModel={updateReferenceModel}
      onReferenceResources={(files) => {
        const invalid = files.find((file) => !/\.(mdl|tga)$/i.test(file.name));
        if (invalid) {
          setError(`${invalid.name} is not a supported .mdl/.tga runtime resource.`);
          return;
        }
        setReferenceResources(files);
        setBuild(undefined);
        setError(undefined);
      }}
      onPartFile={(field, file) => {
        if (!file.name.toLowerCase().endsWith(".glb")) {
          setError(`${field} requires a .glb source.`);
          return;
        }
        const part = parts.find((candidate) => candidate.field === field);
        if (part) {
          setGenerationProvenance((current) => {
            const next = { ...current };
            delete next[field];
            return next;
          });
          updatePart({ ...part, file });
        }
      }}
      onGeneratedPart={(field, file, provenance) => {
        const part = parts.find((candidate) => candidate.field === field);
        if (!part) return;
        setGenerationProvenance((current) => ({ ...current, [field]: provenance }));
        updatePart({ ...part, file });
      }}
      onGenerationSessionChange={setGenerationSession}
      onContinue={() => startAutoFit()}
    />
  ) : step === "INSPECT" && selected ? (
    <ItemPrepare
      row={selected}
      parts={parts}
      utiNumeric={utiNumeric}
      properties={properties}
      namespaceAllocation={namespaceAllocation}
      occupiedNamespace={occupiedNamespace}
      capartModelPrefix={capartModelPrefix}
      capartGenderCode={capartGenderCode}
      proofAppearanceRow={proofAppearanceRow}
      proofRace={proofRace}
      proofGender={proofGender}
      proofPhenotype={proofPhenotype}
      selectedField={selectedField}
      preview={preview}
      seamTolerance={seamTolerance}
      fitTargetLengths={fitTargetLengths}
      seams={seams}
      fitReport={fitReport}
      attachmentProfile={attachmentProfile}
      busy={busy}
      error={error}
      onSelectPart={setSelectedField}
      onPreview={setPreview}
      onPartChange={updatePart}
      onPartScalePreview={updatePartScalePreview}
      onUtiNumericChange={(next) => {
        setUtiNumeric(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onPropertiesChange={(next) => {
        setProperties(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onOccupiedNamespaceChange={(next) => {
        setOccupiedNamespace(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onCapartModelPrefixChange={(next) => {
        setCapartModelPrefix(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onCapartGenderCodeChange={(next) => {
        setCapartGenderCode(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onProofAppearanceRowChange={(next) => {
        setProofAppearanceRow(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onProofRaceChange={(next) => {
        setProofRace(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onProofGenderChange={(next) => {
        setProofGender(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onProofPhenotypeChange={(next) => {
        setProofPhenotype(next);
        setBuild(undefined);
        setError(undefined);
      }}
      onSeamToleranceChange={(next) => {
        setSeamTolerance(next);
        setSeams([]);
        setFitReport(undefined);
        setBuild(undefined);
        setError(undefined);
      }}
      onFitTargetLengthsChange={(next) => {
        setFitTargetLengths(next);
        setSeams([]);
        setFitReport(undefined);
        setBuild(undefined);
        setError(undefined);
      }}
      onAutoFit={startAutoFit}
      onDiscardFit={discardFitPreview}
      onSeams={(next) => {
        setSeams((current) => (
          JSON.stringify(current) === JSON.stringify(next) ? current : next
        ));
      }}
      onBack={() => setStep("SOURCE")}
      onContinue={() => setStep("BUILD")}
    />
  ) : step === "BUILD" && selected ? (
    <ItemBuild
      row={selected}
      parts={resolvedParts}
      busy={busy}
      error={error}
      onBack={() => setStep("INSPECT")}
      onBuild={startBuild}
    />
  ) : build ? (
    <>
      <ItemReview
        snapshot={build}
        onDownload={() => setStep("DOWNLOAD")}
        onError={setError}
      />
      {step === "DOWNLOAD" ? (
        <section className="item-download-complete panel">
          <h2>Item package downloads</h2>
          <p>Runtime HAK/MOD/UTI/MDL/textures/icons and the exact source GLBs with canonical manifest.yaml remain separate, hash-verified artifacts.</p>
          <ArtifactDownloads artifacts={build.artifacts} onError={setError} />
        </section>
      ) : null}
    </>
  ) : null;

  return (
    <StudioShell
      header={<StudioHeader version="v0.2.0" environment="local" theme="dark" />}
      workflow={(
        <WorkflowStepper
          itemMode
          currentStep={step}
          visitedSteps={visited}
          completedSteps={completed}
          blockedSteps={blocked}
          onStepSelect={(next) => {
            if (stepIndex(next) <= stepIndex(maxStep)) setStep(next);
          }}
        />
      )}
      inputs={null}
      expandPrimaryToWorkspace
      debugDrawer={(
        <section className="debug-drawer-placeholder" aria-label="Debug Drawer">
          <strong>Item Debug Drawer</strong>
          <span>{error ?? "BaseItem resolver · part MDL readback · UTI numeric readback"}</span>
        </section>
      )}
    >
      {contents}
    </StudioShell>
  );
}
