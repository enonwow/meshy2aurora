import type { ReactNode } from "react";

interface StudioShellProps {
  header: ReactNode;
  workflow: ReactNode;
  inputs: ReactNode;
  aside?: ReactNode;
  expandPrimaryToWorkspace?: boolean;
  workspaceMode?: boolean;
  workflowRail?: boolean;
  workflowLabel?: string;
  debugDrawer: ReactNode;
  children: ReactNode;
}

export function StudioShell({
  header,
  workflow,
  inputs,
  aside,
  expandPrimaryToWorkspace = false,
  workspaceMode = false,
  workflowRail = false,
  workflowLabel = "Conversion workflow",
  debugDrawer,
  children,
}: StudioShellProps) {
  const showInputs = !expandPrimaryToWorkspace
    && inputs !== null
    && inputs !== undefined;
  const workspaceClassName = `studio-shell__workspace${aside ? "" : " studio-shell__workspace--wide-primary"}${expandPrimaryToWorkspace ? " studio-shell__workspace--full-primary" : ""}`;
  const contentClassName = `studio-shell__content${showInputs ? "" : " studio-shell__content--no-inputs"}${aside || !showInputs ? "" : " studio-shell__content--wide-primary"}${expandPrimaryToWorkspace || (!showInputs && !aside) ? " studio-shell__content--full-primary" : ""}`;
  const content = (
    <>
      {showInputs ? (
        <div className="studio-shell__inputs">{inputs}</div>
      ) : null}
      <div className="studio-shell__primary">{children}</div>
      {aside ? <div className="studio-shell__aside">{aside}</div> : null}
    </>
  );

  return (
    <main className={`studio-shell${workspaceMode ? " studio-shell--workspace" : ""}${workflowRail ? " studio-shell--workflow-rail" : ""}`}>
      <div className={`studio-shell__masthead${workspaceMode ? " studio-shell__masthead--workspace" : ""}`}>
        {!workspaceMode ? header : null}
        {!workspaceMode && !workflowRail ? workflow : null}
      </div>
      <div className={workspaceClassName}>
        {workflowRail ? (
          <aside
            className="studio-shell__workflow-rail"
            aria-label={workflowLabel}
          >
            <span>{workflowLabel}</span>
            {workflow}
          </aside>
        ) : null}
        {workflowRail ? (
          <div className={contentClassName}>{content}</div>
        ) : content}
      </div>
      {debugDrawer}
    </main>
  );
}
