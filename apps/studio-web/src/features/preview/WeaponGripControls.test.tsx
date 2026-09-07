// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { defaultCreatureWeaponGripOptionsV1 } from "../source/weaponGrip";
import { WeaponGripControls } from "./WeaponGripControls";

const roots: Root[] = [];

afterEach(async () => {
  await act(async () => roots.splice(0).forEach((root) => root.unmount()));
  document.body.replaceChildren();
});

describe("WeaponGripControls", () => {
  it("exposes independent right/left roll, pitch, and yaw only in manual mode", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    let value = defaultCreatureWeaponGripOptionsV1();
    const onChange = vi.fn((next) => { value = next; });

    const render = async () => act(async () => root.render(
      <WeaponGripControls value={value} onChange={onChange} />,
    ));
    await render();
    expect(container.querySelectorAll("fieldset:disabled")).toHaveLength(2);

    const family = container.querySelector<HTMLSelectElement>('select[aria-label="Item grip family"]');
    await act(async () => {
      if (!family) throw new Error("item family select unavailable");
      family.value = "SPEAR_POLEARM";
      family.dispatchEvent(new Event("change", { bubbles: true }));
    });
    expect(value.itemFamily).toBe("SPEAR_POLEARM");

    const mode = container.querySelector<HTMLSelectElement>('select[aria-label="Weapon rotation mode"]');
    await act(async () => {
      if (!mode) throw new Error("weapon mode select unavailable");
      mode.value = "AUTO_PLUS_OFFSETS";
      mode.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await render();
    expect(value.mode).toBe("AUTO_PLUS_OFFSETS");
    expect(container.querySelectorAll("fieldset:disabled")).toHaveLength(0);

    const rightRoll = container.querySelector<HTMLInputElement>('input[aria-label="Right Roll (blade +Y)"]');
    await act(async () => {
      if (!rightRoll) throw new Error("right roll input unavailable");
      const valueSetter = Object.getOwnPropertyDescriptor(
        window.HTMLInputElement.prototype,
        "value",
      )?.set;
      if (!valueSetter) throw new Error("native input value setter unavailable");
      valueSetter.call(rightRoll, "179.5");
      rightRoll.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(value.rightHand.rollDegrees).toBe(179.5);
    expect(value.leftHand).toEqual({ rollDegrees: 0, pitchDegrees: 0, yawDegrees: 0 });
  });

  it("resets mode and both hands atomically", async () => {
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    roots.push(root);
    const onChange = vi.fn();
    await act(async () => root.render(
      <WeaponGripControls
        value={{
          schemaVersion: 1,
          mode: "AUTO_PLUS_OFFSETS",
          rightHand: { rollDegrees: 10, pitchDegrees: 20, yawDegrees: 30 },
          leftHand: { rollDegrees: -10, pitchDegrees: -20, yawDegrees: -30 },
        }}
        onChange={onChange}
      />,
    ));
    const reset = [...container.querySelectorAll("button")].find((button) => button.textContent === "Reset to Auto");
    await act(async () => reset?.click());
    expect(onChange).toHaveBeenLastCalledWith(defaultCreatureWeaponGripOptionsV1());
  });
});
