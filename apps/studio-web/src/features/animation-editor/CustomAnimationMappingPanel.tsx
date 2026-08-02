import { useState } from "react";
import type { DirectCreatureBaseSlotV1 } from "../animation-mapping/types";
import type {
  AnimationStudioDocumentV1,
  CreatureAnimationAuthoringV2,
  CustomAnimationDefinitionV2,
} from "../animation-studio/types";
import { CustomAnimationAssignmentCard } from "./CustomAnimationAssignmentCard";
import { CustomAnimationPicker } from "./CustomAnimationPicker";
import { CustomAnimationProvenanceCard } from "./CustomAnimationProvenanceCard";
import { PhasedCustomAnimationCreator } from "./PhasedCustomAnimationCreator";
import {
  CUSTOM_ATTACK_DEMO_BASE_SLOTS_V1,
  applyCustomAttackDemoRouteV1,
  assignCustomAnimationToBaseSlotV2,
  clearCustomAnimationFromBaseSlotV2,
  openCustomAnimationInEditorV1,
  projectCustomAnimationLibraryV1,
  type AnimationStudioLibrarySourceV1,
  type CustomAttackDemoContractV1,
} from "./editing";
import { resolveEffectiveAnimationSourceV1 } from "../animation-mapping/fallbacks";

export function CustomAnimationMappingPanel({
  slot,
  authoring,
  studio,
  sourceInventory = [],
  onAuthoringChange,
  onCreate,
  onOpenClip,
}: {
  slot: DirectCreatureBaseSlotV1;
  authoring: CreatureAnimationAuthoringV2;
  studio: AnimationStudioDocumentV1;
  sourceInventory?: readonly AnimationStudioLibrarySourceV1[];
  onAuthoringChange: (authoring: CreatureAnimationAuthoringV2) => void;
  onCreate: () => void;
  onOpenClip: (clipId: string) => void;
}) {
  const items = projectCustomAnimationLibraryV1(
    authoring,
    studio,
    sourceInventory,
  );
  const assignment = authoring.assignments.find(({ targetSlot }) => targetSlot === slot);
  const [firstAssignable] = items.filter(({ assignable }) => assignable);
  const [preferredCustomId, setPreferredCustomId] = useState<string | null>(null);
  const [attackDemoPreview, setAttackDemoPreview] = useState<{
    sourceAuthoringRevision: number;
    authoring: CreatureAnimationAuthoringV2;
    contract: CustomAttackDemoContractV1;
  } | null>(null);
  const [attackDemoUndo, setAttackDemoUndo] = useState<{
    appliedAuthoringRevision: number;
    previousAuthoring: CreatureAnimationAuthoringV2;
  } | null>(null);
  const selectedId = preferredCustomId && items.some(({ id }) => id === preferredCustomId)
    ? preferredCustomId
    : assignment?.sourceKind === "CUSTOM"
      ? assignment.customAnimationId
      : firstAssignable?.id ?? null;
  const selected = items.find(({ id }) => id === selectedId) ?? null;
  const attackDemoHasBase42 = [
    "cpause1",
    ...CUSTOM_ATTACK_DEMO_BASE_SLOTS_V1,
  ].every((requiredSlot) => {
    try {
      resolveEffectiveAnimationSourceV1(
        requiredSlot as DirectCreatureBaseSlotV1,
        authoring.assignments,
        authoring.fallbacks,
      );
      return true;
    } catch {
      return false;
    }
  });
  const custom = authoring.customAnimations.find(({ id }) => id === selectedId) ?? null;
  const openSelected = (customId: string) => onOpenClip(
    openCustomAnimationInEditorV1(customId, authoring, studio),
  );
  const addPhased = (definition: CustomAnimationDefinitionV2) => {
    setPreferredCustomId(definition.id);
    onAuthoringChange({
      ...authoring,
      authoringRevision: authoring.authoringRevision + 1,
      customAnimations: [...authoring.customAnimations, definition],
    });
  };
  return (
    <section className="custom-animation-mapping-panel" aria-label={`Custom realization for ${slot}`}>
      <PhasedCustomAnimationCreator
        studio={studio}
        existingCustomIds={authoring.customAnimations.map(({ id }) => id)}
        existingCustomNames={authoring.customAnimations.map(({ name }) => name)}
        onCreate={addPhased}
      />
      <CustomAnimationPicker
        items={items}
        selectedId={selectedId}
        onSelect={(id) => {
          const item = items.find((candidate) => candidate.id === id);
          if (!item?.assignable) return;
          setPreferredCustomId(id);
        }}
        onCreate={onCreate}
        onOpenSelected={openSelected}
      />
      <CustomAnimationAssignmentCard
        slot={slot}
        item={selected}
        onAssign={() => {
          if (!selected) return;
          onAuthoringChange(assignCustomAnimationToBaseSlotV2(
            authoring,
            slot,
            selected.id,
            studio,
            sourceInventory,
          ));
        }}
        onClear={() => onAuthoringChange(
          clearCustomAnimationFromBaseSlotV2(authoring, slot),
        )}
      />
      <section
        className="custom-animation-attack-demo"
        aria-label="Custom attack demo routing"
      >
        <h3>Attack demo</h3>
        <p>
          Route the selected Custom animation through all three native attack
          states. The production <code>cpause1</code> idle stays unchanged.
        </p>
        <button
          type="button"
          disabled={!selected?.assignable || !attackDemoHasBase42}
          title={!attackDemoHasBase42
            ? "Complete the Base 42 mapping before preparing an attack demo."
            : undefined}
          onClick={() => {
            if (!selected) return;
            const preview = applyCustomAttackDemoRouteV1(
              authoring,
              selected.id,
              studio,
              sourceInventory,
            );
            setAttackDemoPreview({
              sourceAuthoringRevision: authoring.authoringRevision,
              ...preview,
            });
          }}
        >
          Preview attack demo (3 slots)
        </button>
        {attackDemoPreview ? (
          <div className="custom-animation-attack-demo__preview" role="status">
            <p>
              Preview only: <strong>{attackDemoPreview.contract.customAnimationId}</strong>
              {" "}will route to{" "}
              <code>{attackDemoPreview.contract.runtimeBaseSlots.join(", ")}</code>.
              No production mapping has changed.
            </p>
            <button
              type="button"
              disabled={
                attackDemoPreview.sourceAuthoringRevision
                !== authoring.authoringRevision
              }
              title={
                attackDemoPreview.sourceAuthoringRevision
                !== authoring.authoringRevision
                  ? "The mapping changed after this preview. Create a fresh preview."
                  : undefined
              }
              onClick={() => {
                setAttackDemoUndo({
                  appliedAuthoringRevision:
                    attackDemoPreview.authoring.authoringRevision,
                  previousAuthoring: authoring,
                });
                onAuthoringChange(attackDemoPreview.authoring);
                setAttackDemoPreview(null);
              }}
            >
              Apply attack demo routing
            </button>
            <button
              type="button"
              onClick={() => setAttackDemoPreview(null)}
            >
              Discard preview
            </button>
          </div>
        ) : null}
        {attackDemoUndo ? (
          <button
            type="button"
            disabled={
              attackDemoUndo.appliedAuthoringRevision
              !== authoring.authoringRevision
            }
            title={
              attackDemoUndo.appliedAuthoringRevision
              !== authoring.authoringRevision
                ? "The mapping changed after the attack demo was applied; use the global undo history."
                : undefined
            }
            onClick={() => {
              onAuthoringChange({
                ...attackDemoUndo.previousAuthoring,
                authoringRevision: authoring.authoringRevision + 1,
              });
              setAttackDemoUndo(null);
            }}
          >
            Revert applied attack demo
          </button>
        ) : null}
      </section>
      {custom ? (
        <CustomAnimationProvenanceCard
          custom={custom}
          sourceRevision={studio.sourceRevision}
        />
      ) : null}
    </section>
  );
}
