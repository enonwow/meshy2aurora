import type { ReactNode } from "react";

interface StudioShellProps {
  header: ReactNode;
  workflow: ReactNode;
  inputs: ReactNode;
  aside?: ReactNode;
  expandPrimaryToWorkspace?: boolean;
  workspaceMode?: boolean;
  debugDrawer: ReactNode;
  children: ReactNode;
}

export function StudioShell({ header, workflow, inputs, aside, expandPrimaryToWorkspace = false, workspaceMode = false, debugDrawer, children }: StudioShellProps) {
  return (
    <main className={`studio-shell${workspaceMode ? " studio-shell--workspace" : ""}`}>
      <div className={`studio-shell__masthead${workspaceMode ? " studio-shell__masthead--workspace" : ""}`}>
        {!workspaceMode ? header : null}
        {!workspaceMode ? workflow : null}
      </div>
      <div className={`studio-shell__workspace${aside ? "" : " studio-shell__workspace--wide-primary"}${expandPrimaryToWorkspace ? " studio-shell__workspace--full-primary" : ""}`}>
        {!expandPrimaryToWorkspace ? <div className="studio-shell__inputs">{inputs}</div> : null}
        <div className="studio-shell__primary">{children}</div>
        {aside ? <div className="studio-shell__aside">{aside}</div> : null}
      </div>
      {debugDrawer}
    </main>
  );
}
