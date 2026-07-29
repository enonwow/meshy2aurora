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
  assignCustomAnimationToBaseSlotV2,
  clearCustomAnimationFromBaseSlotV2,
  openCustomAnimationInEditorV1,
  projectCustomAnimationLibraryV1,
  type AnimationStudioLibrarySourceV1,
} from "./editing";

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
  const selectedId = preferredCustomId && items.some(({ id }) => id === preferredCustomId)
    ? preferredCustomId
    : assignment?.sourceKind === "CUSTOM"
      ? assignment.customAnimationId
      : firstAssignable?.id ?? null;
  const selected = items.find(({ id }) => id === selectedId) ?? null;
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
      {custom ? (
        <CustomAnimationProvenanceCard
          custom={custom}
          sourceRevision={studio.sourceRevision}
        />
      ) : null}
    </section>
  );
}
