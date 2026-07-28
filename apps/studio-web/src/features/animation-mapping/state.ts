import {
  approveFallbackV1,
  rejectFallbackV1,
} from "./fallbacks";
import {
  CREATURE_ANIMATION_AUTHORING_PROFILE_V1,
  type AnimationSourceAssignmentV1,
  type CreatureAnimationAuthoringV1,
  type CustomAnimationDefinitionV1,
  type DirectCreatureBaseSlotV1,
  type DirectCreatureModelTypeV1,
} from "./types";

export type CreatureAnimationAuthoringEventV1 =
  | {
      type: "ANIMATION_SOURCE_ASSIGNED";
      assignment: AnimationSourceAssignmentV1;
    }
  | {
      type: "ANIMATION_SOURCE_CLEARED";
      slot: DirectCreatureBaseSlotV1;
    }
  | {
      type: "ANIMATION_FALLBACK_APPROVED";
      fallbackId: string;
    }
  | {
      type: "ANIMATION_FALLBACK_REJECTED";
      fallbackId: string;
    }
  | {
      type: "CUSTOM_ANIMATION_ADDED";
      custom: CustomAnimationDefinitionV1;
    }
  | {
      type: "CUSTOM_ANIMATION_UPDATED";
      id: string;
      patch: Partial<Omit<CustomAnimationDefinitionV1, "id">>;
    }
  | {
      type: "CUSTOM_ANIMATION_REMOVED";
      id: string;
      confirmed?: boolean;
    };

export function createCreatureAnimationAuthoringV1(
  sourceRevision: string,
  modelType: DirectCreatureModelTypeV1,
): CreatureAnimationAuthoringV1 {
  if (!sourceRevision.trim()) {
    throw new RangeError("Creature animation authoring requires a source revision");
  }
  return {
    schemaVersion: 1,
    profile: CREATURE_ANIMATION_AUTHORING_PROFILE_V1,
    modelType,
    sourceRevision,
    authoringRevision: 1,
    assignments: [],
    fallbacks: [],
    customAnimations: [],
  };
}

export function reduceCreatureAnimationAuthoringV1(
  authoring: CreatureAnimationAuthoringV1,
  event: CreatureAnimationAuthoringEventV1,
): CreatureAnimationAuthoringV1 {
  switch (event.type) {
    case "ANIMATION_SOURCE_ASSIGNED": {
      const existing = authoring.assignments.find(
        ({ targetSlot }) => targetSlot === event.assignment.targetSlot,
      );
      if (existing && JSON.stringify(existing) === JSON.stringify(event.assignment)) {
        return authoring;
      }
      return revise(authoring, {
        assignments: [
          ...authoring.assignments.filter(
            ({ targetSlot }) => targetSlot !== event.assignment.targetSlot,
          ),
          event.assignment,
        ],
      });
    }
    case "ANIMATION_SOURCE_CLEARED": {
      if (!authoring.assignments.some(({ targetSlot }) => targetSlot === event.slot)) {
        return authoring;
      }
      return revise(authoring, {
        assignments: authoring.assignments.filter(
          ({ targetSlot }) => targetSlot !== event.slot,
        ),
      });
    }
    case "ANIMATION_FALLBACK_APPROVED":
      return approveFallbackV1(authoring, event.fallbackId);
    case "ANIMATION_FALLBACK_REJECTED":
      return rejectFallbackV1(authoring, event.fallbackId);
    case "CUSTOM_ANIMATION_ADDED": {
      if (authoring.customAnimations.some(({ id }) => id === event.custom.id)) {
        throw new RangeError(`Duplicate custom animation id: ${event.custom.id}`);
      }
      return revise(authoring, {
        customAnimations: [...authoring.customAnimations, event.custom],
      });
    }
    case "CUSTOM_ANIMATION_UPDATED": {
      const index = authoring.customAnimations.findIndex(({ id }) => id === event.id);
      if (index < 0) throw new RangeError(`Unknown custom animation: ${event.id}`);
      const updated = {
        ...authoring.customAnimations[index],
        ...event.patch,
        id: event.id,
      };
      if (JSON.stringify(updated) === JSON.stringify(authoring.customAnimations[index])) {
        return authoring;
      }
      return revise(authoring, {
        customAnimations: authoring.customAnimations.map((custom, customIndex) => (
          customIndex === index ? updated : custom
        )),
      });
    }
    case "CUSTOM_ANIMATION_REMOVED": {
      if (!authoring.customAnimations.some(({ id }) => id === event.id)) return authoring;
      const inUse = authoring.assignments.some(
        ({ customAnimationId }) => customAnimationId === event.id,
      );
      if (inUse && event.confirmed !== true) {
        throw new RangeError(
          `Custom animation ${event.id} is in use and requires confirmation`,
        );
      }
      return revise(authoring, {
        assignments: inUse
          ? authoring.assignments.filter(
              ({ customAnimationId }) => customAnimationId !== event.id,
            )
          : authoring.assignments,
        customAnimations: authoring.customAnimations.filter(({ id }) => id !== event.id),
      });
    }
  }
}

interface AnimationMappingCurrentSessionV1 {
  readonly revision: number;
  readonly target: "CREATURE" | "PLACEABLE" | "TILE";
  readonly source: { readonly sha256: string | null } | null;
  readonly animationMapping: {
    readonly revision: number;
    readonly value: CreatureAnimationAuthoringV1;
  } | null;
}

export function isAnimationMappingCurrentV1(
  session: AnimationMappingCurrentSessionV1,
): boolean {
  return session.target === "CREATURE"
    && session.source?.sha256 !== null
    && session.source?.sha256 !== undefined
    && session.animationMapping?.revision === session.revision
    && session.animationMapping.value.sourceRevision === session.source.sha256;
}

function revise(
  authoring: CreatureAnimationAuthoringV1,
  patch: Partial<Pick<
    CreatureAnimationAuthoringV1,
    "assignments" | "fallbacks" | "customAnimations"
  >>,
): CreatureAnimationAuthoringV1 {
  return {
    ...authoring,
    ...patch,
    authoringRevision: authoring.authoringRevision + 1,
  };
}
