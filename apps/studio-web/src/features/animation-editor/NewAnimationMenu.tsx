import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

export function NewAnimationMenu({
  onCreateBlank,
  onCreateProcedural,
}: {
  onCreateBlank: () => void;
  onCreateProcedural?: () => void;
}) {
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuId = useId();
  useEffect(() => {
    if (open) {
      menuRef.current?.querySelector<HTMLButtonElement>('[role="menuitem"]')
        ?.focus();
    }
  }, [open]);
  const closeAndRestoreFocus = () => {
    setOpen(false);
    queueMicrotask(() => triggerRef.current?.focus());
  };
  const choose = (action: () => void) => {
    closeAndRestoreFocus();
    action();
  };
  return (
    <div className="new-animation-menu">
      <button
        type="button"
        ref={triggerRef}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={open ? menuId : undefined}
        onClick={() => setOpen((value) => !value)}
        onKeyDown={(event) => {
          if (event.key === "ArrowDown") {
            event.preventDefault();
            setOpen(true);
          }
        }}
      >
        + New animation
      </button>
      {open ? (
        <div
          ref={menuRef}
          id={menuId}
          role="menu"
          aria-label="New animation"
          onKeyDown={(event) => moveMenuFocus(event, closeAndRestoreFocus)}
        >
          <button
            type="button"
            role="menuitem"
            onClick={() => choose(onCreateBlank)}
          >
            From current pose
          </button>
          {onCreateProcedural ? (
            <button
              type="button"
              role="menuitem"
              onClick={() => choose(onCreateProcedural)}
            >
              From procedural template
            </button>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}

function moveMenuFocus(
  event: KeyboardEvent<HTMLDivElement>,
  closeAndRestoreFocus: () => void,
) {
  if (event.key === "Escape") {
    event.preventDefault();
    closeAndRestoreFocus();
    return;
  }
  if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
  const items = Array.from(
    event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="menuitem"]'),
  );
  const current = items.indexOf(document.activeElement as HTMLButtonElement);
  if (items.length === 0) return;
  event.preventDefault();
  const next = event.key === "Home"
    ? 0
    : event.key === "End"
      ? items.length - 1
      : event.key === "ArrowDown"
        ? (Math.max(current, -1) + 1) % items.length
        : (current <= 0 ? items.length - 1 : current - 1);
  items[next]?.focus();
}
