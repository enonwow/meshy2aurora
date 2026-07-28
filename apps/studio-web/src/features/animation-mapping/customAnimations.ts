import {
  FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1,
} from "../source/directCreatureAnimationProfile";
import { reduceCreatureAnimationAuthoringV1 } from "./state";
import type {
  AnimationMappingDiagnosticV1,
  CreatureAnimationAuthoringV1,
  CustomAnimationDefinitionV1,
} from "./types";

export type CreateCustomAnimationInputV1 =
  Omit<CustomAnimationDefinitionV1, "id"> & { id?: string };

export function createCustomAnimationV1(
  input: CreateCustomAnimationInputV1,
): CustomAnimationDefinitionV1 {
  const id = input.id ?? `custom-${crypto.randomUUID()}`;
  const custom: CustomAnimationDefinitionV1 = { ...input, id };
  const diagnostics = [
    ...validateCustomAnimationNameV1(custom.name, []),
    ...validateCustomAnimationPhasesV1(custom),
  ];
  if (diagnostics.length > 0) {
    throw new RangeError(diagnostics.map(({ message }) => message).join(" "));
  }
  return custom;
}

export function validateCustomAnimationNameV1(
  name: string,
  existingNames: readonly string[],
): AnimationMappingDiagnosticV1[] {
  const diagnostics: AnimationMappingDiagnosticV1[] = [];
  const normalized = name.trim().toLocaleLowerCase("en-US");
  if (!/^[a-z][a-z0-9_]{0,15}$/.test(normalized)) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-CUSTOM-NAME",
      "customAnimation.name",
      "Custom animation name must be an ASCII resref-like identifier with at most 16 characters.",
      "Use letters, digits or underscore and start with a letter.",
    ));
  }
  if (FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.some((slot) => slot === normalized)) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-CUSTOM-NAME-BASE-SLOT",
      "customAnimation.name",
      `${name} collides with a Base 42 slot.`,
      "Choose a name outside the Base 42 namespace.",
    ));
  }
  if (existingNames.some((existing) => existing.toLocaleLowerCase("en-US") === normalized)) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-CUSTOM-NAME-DUPLICATE",
      "customAnimation.name",
      `${name} duplicates another custom animation.`,
      "Choose a unique custom animation name.",
    ));
  }
  return diagnostics;
}

export function validateCustomAnimationPhasesV1(
  custom: CustomAnimationDefinitionV1,
): AnimationMappingDiagnosticV1[] {
  if (custom.playback === "ONE_SHOT") {
    return custom.sourceClipName?.trim() && custom.phases.length === 0
      ? []
      : [diagnostic(
          "M2A-ANIMATION-CUSTOM-ONE-SHOT-PHASES",
          `customAnimations.${custom.id}`,
          "One-shot custom animation requires one source clip and no START/LOOP/END phases.",
          "Remove phases or switch to LOOPING_PHASED.",
        )];
  }

  const counts = new Map<string, number>();
  custom.phases.forEach(({ phase }) => counts.set(phase, (counts.get(phase) ?? 0) + 1));
  const invalid = custom.sourceClipName !== null
    || counts.get("LOOP") !== 1
    || (counts.get("START") ?? 0) > 1
    || (counts.get("END") ?? 0) > 1
    || custom.phases.some(({ sourceClipName }) => !sourceClipName.trim());
  const diagnostics = invalid
    ? [diagnostic(
        "M2A-ANIMATION-CUSTOM-LOOP-PHASES",
        `customAnimations.${custom.id}`,
        "Looping custom animation requires exactly one LOOP and at most one START and END source.",
        "Correct the phased clip configuration.",
      )]
    : [];
  if (custom.name.length > 14) {
    diagnostics.push(diagnostic(
      "M2A-ANIMATION-CUSTOM-PHASE-NAME",
      `customAnimations.${custom.id}.name`,
      "A phased custom name must have at most 14 characters so _s and _e fit Aurora's 16-character resref boundary.",
      "Shorten the custom animation name.",
    ));
  }
  return diagnostics;
}

export function validateCustomAnimationOutputNamesV1(
  customs: readonly CustomAnimationDefinitionV1[],
): AnimationMappingDiagnosticV1[] {
  const claimed = new Map(
    FULL_NATIVE_DIRECT_CREATURE_CLIPS_V1.map((name) => [
      name.toLocaleLowerCase("en-US"),
      `Base 42 slot ${name}`,
    ]),
  );
  const diagnostics: AnimationMappingDiagnosticV1[] = [];
  customs.forEach((custom, index) => {
    let conflict: { outputName: string; owner: string } | undefined;
    for (const outputName of customOutputNames(custom)) {
      const folded = outputName.toLocaleLowerCase("en-US");
      const owner = claimed.get(folded);
      if (owner && !conflict) conflict = { outputName, owner };
      if (!owner) claimed.set(folded, `custom animation ${custom.name}`);
    }
    if (conflict) {
      diagnostics.push(diagnostic(
        "M2A-ANIMATION-CUSTOM-OUTPUT-NAME-CONFLICT",
        `customAnimations[${index}].name`,
        `Materialized custom clip ${conflict.outputName} conflicts with ${conflict.owner}.`,
        "Rename one custom animation so every materialized clip name is unique.",
      ));
    }
  });
  return diagnostics;
}

export function addCustomAnimationV1(
  authoring: CreatureAnimationAuthoringV1,
  custom: CustomAnimationDefinitionV1,
): CreatureAnimationAuthoringV1 {
  const diagnostics = [
    ...validateCustomAnimationNameV1(
      custom.name,
      authoring.customAnimations.map(({ name }) => name),
    ),
    ...validateCustomAnimationPhasesV1(custom),
    ...validateCustomAnimationOutputNamesV1([
      ...authoring.customAnimations,
      custom,
    ]),
  ];
  if (diagnostics.length > 0) throw new RangeError(diagnostics[0].message);
  return reduceCreatureAnimationAuthoringV1(authoring, {
    type: "CUSTOM_ANIMATION_ADDED",
    custom,
  });
}

export function updateCustomAnimationV1(
  authoring: CreatureAnimationAuthoringV1,
  id: string,
  patch: Partial<Omit<CustomAnimationDefinitionV1, "id">>,
): CreatureAnimationAuthoringV1 {
  const existing = authoring.customAnimations.find((custom) => custom.id === id);
  if (!existing) throw new RangeError(`Unknown custom animation: ${id}`);
  const updated = { ...existing, ...patch, id };
  const diagnostics = [
    ...validateCustomAnimationNameV1(
      updated.name,
      authoring.customAnimations.filter((custom) => custom.id !== id).map(({ name }) => name),
    ),
    ...validateCustomAnimationPhasesV1(updated),
    ...validateCustomAnimationOutputNamesV1(
      authoring.customAnimations.map((custom) => (
        custom.id === id ? updated : custom
      )),
    ),
  ];
  if (diagnostics.length > 0) throw new RangeError(diagnostics[0].message);
  return reduceCreatureAnimationAuthoringV1(authoring, {
    type: "CUSTOM_ANIMATION_UPDATED",
    id,
    patch,
  });
}

export function removeCustomAnimationV1(
  authoring: CreatureAnimationAuthoringV1,
  id: string,
  confirmed = false,
): CreatureAnimationAuthoringV1 {
  const inUse = authoring.assignments.some(
    ({ customAnimationId }) => customAnimationId === id,
  );
  if (inUse && !confirmed) {
    throw new RangeError(`Custom animation ${id} is in use and requires confirmation`);
  }
  const withoutAssignments = inUse
    ? {
        ...authoring,
        assignments: authoring.assignments.filter(
          ({ customAnimationId }) => customAnimationId !== id,
        ),
      }
    : authoring;
  return reduceCreatureAnimationAuthoringV1(withoutAssignments, {
    type: "CUSTOM_ANIMATION_REMOVED",
    id,
  });
}

function customOutputNames(custom: CustomAnimationDefinitionV1): string[] {
  if (custom.playback === "ONE_SHOT") return [custom.name];
  return custom.phases.map(({ phase }) => (
    phase === "START"
      ? `${custom.name}_s`
      : phase === "END"
        ? `${custom.name}_e`
        : custom.name
  ));
}

function diagnostic(
  code: string,
  path: string,
  message: string,
  action: string,
): AnimationMappingDiagnosticV1 {
  return {
    schemaVersion: 1,
    code,
    path,
    level: "BLOCKING",
    message,
    action,
  };
}
