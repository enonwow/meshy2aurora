import type { ReactNode } from "react";

interface StudioShellProps {
  header: ReactNode;
  workflow: ReactNode;
  inputs: ReactNode;
  aside?: ReactNode;
  debugDrawer: ReactNode;
  children: ReactNode;
}

export function StudioShell({ header, workflow, inputs, aside, debugDrawer, children }: StudioShellProps) {
  return (
    <main className="studio-shell">
      <div className="studio-shell__masthead">
        {header}
        {workflow}
      </div>
      <div className={`studio-shell__workspace${aside ? "" : " studio-shell__workspace--wide-primary"}`}>
        <div className="studio-shell__inputs">{inputs}</div>
        <div className="studio-shell__primary">{children}</div>
        {aside ? <div className="studio-shell__aside">{aside}</div> : null}
      </div>
      {debugDrawer}
    </main>
  );
}
