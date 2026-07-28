import type { KeyboardEvent } from "react";

export type AnimationMappingModeV1 = "MAP" | "EDIT";

export function AnimationMappingModeSwitch({
  value,
  onChange,
}: {
  value: AnimationMappingModeV1;
  onChange: (value: AnimationMappingModeV1) => void;
}) {
  return (
    <div
      className="animation-studio-mode-switch"
      role="tablist"
      aria-label="Animation Mapping mode"
    >
      <button
        type="button"
        role="tab"
        aria-selected={value === "MAP"}
        tabIndex={value === "MAP" ? 0 : -1}
        onClick={() => onChange("MAP")}
        onKeyDown={moveModeTabFocus}
      >
        Map animations
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={value === "EDIT"}
        tabIndex={value === "EDIT" ? 0 : -1}
        onClick={() => onChange("EDIT")}
        onKeyDown={moveModeTabFocus}
      >
        Create &amp; edit <span className="animation-studio-beta">Beta</span>
      </button>
    </div>
  );
}

function moveModeTabFocus(event: KeyboardEvent<HTMLButtonElement>) {
  if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
  const tabs = Array.from(
    event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>(
      ':scope > [role="tab"]',
    ) ?? [],
  );
  const current = tabs.indexOf(event.currentTarget);
  if (current < 0 || tabs.length === 0) return;
  event.preventDefault();
  const next = event.key === "Home"
    ? 0
    : event.key === "End"
      ? tabs.length - 1
      : event.key === "ArrowRight"
        ? (current + 1) % tabs.length
        : (current - 1 + tabs.length) % tabs.length;
  tabs[next]?.focus();
  tabs[next]?.click();
}
