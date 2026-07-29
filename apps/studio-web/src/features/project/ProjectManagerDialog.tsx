import { useEffect, useId, useRef, useState, type ChangeEvent } from "react";
import type { StudioProjectPersistence } from "../../app/StudioHeader";
import type { Meshy2AuroraProjectV1 } from "./schema";
import type { ProjectRecoveryRecordV1 } from "./persistence";
import "./project.css";

export interface ProjectManagerDialogProps {
  readonly project: Meshy2AuroraProjectV1;
  readonly persistence: StudioProjectPersistence;
  readonly recoveryRecords: readonly ProjectRecoveryRecordV1[];
  readonly busy?: boolean;
  readonly error?: string;
  readonly onClose: () => void;
  readonly onRename: (name: string) => void;
  readonly onNew: () => void;
  readonly onExport: () => void;
  readonly onImport: (file: File) => void;
  readonly onDuplicate: () => void;
  readonly onDelete: () => void;
  readonly onRecover: (projectId: string) => void;
  readonly onDeleteRecovery: (projectId: string) => void;
}

function selectImport(
  event: ChangeEvent<HTMLInputElement>,
  onImport: (file: File) => void,
) {
  const file = event.currentTarget.files?.[0];
  if (file) onImport(file);
  event.currentTarget.value = "";
}

export function ProjectManagerDialog({
  project,
  persistence,
  recoveryRecords,
  busy = false,
  error,
  onClose,
  onRename,
  onNew,
  onExport,
  onImport,
  onDuplicate,
  onDelete,
  onRecover,
  onDeleteRecovery,
}: ProjectManagerDialogProps) {
  const headingId = useId();
  const importId = useId();
  const nameId = useId();
  const cardRef = useRef<HTMLDivElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  const [name, setName] = useState(project.identity.name);
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    closeRef.current?.focus();
  }, []);

  useEffect(() => {
    setName(project.identity.name);
    setConfirmDelete(false);
  }, [project.identity.projectId, project.identity.name]);

  return (
    <div
      className="project-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby={headingId}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          onClose();
          return;
        }
        if (event.key !== "Tab") return;
        const focusable = Array.from(cardRef.current?.querySelectorAll<HTMLElement>(
          "button:not(:disabled), input:not(:disabled), [href], [tabindex]:not([tabindex='-1'])",
        ) ?? []);
        if (focusable.length === 0) return;
        const first = focusable[0]!;
        const last = focusable.at(-1)!;
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first.focus();
        }
      }}
    >
      <div ref={cardRef} className="project-dialog__card">
        <header className="project-dialog__header">
          <div>
            <p className="project-dialog__eyebrow">Local-first project</p>
            <h2 id={headingId}>Project manager</h2>
          </div>
          <button ref={closeRef} type="button" onClick={onClose}>Close</button>
        </header>

        <section className="project-dialog__current" aria-label="Current project">
          <div className="project-dialog__identity">
            <label htmlFor={nameId}>Project name</label>
            <div>
              <input
                id={nameId}
                value={name}
                maxLength={120}
                onChange={(event) => setName(event.currentTarget.value)}
              />
              <button
                type="button"
                disabled={busy || !name.trim() || name.trim() === project.identity.name}
                onClick={() => onRename(name)}
              >
                Rename
              </button>
            </div>
          </div>
          <dl className="project-dialog__metadata">
            <div><dt>Project ID</dt><dd><code>{project.identity.projectId}</code></dd></div>
            <div><dt>Revision</dt><dd>{project.revision}</dd></div>
            <div><dt>Target</dt><dd>{project.target}</dd></div>
            <div><dt>Local state</dt><dd>{persistence.replace("_", " ")}</dd></div>
            <div>
              <dt>Source binding</dt>
              <dd>
                {project.files.sourceGlb
                  ? <code title={project.files.sourceGlb.sha256}>
                      {project.files.sourceGlb.name} · {project.files.sourceGlb.sha256.slice(0, 12)}
                    </code>
                  : "Not selected"}
              </dd>
            </div>
          </dl>
          <div className="project-dialog__actions" aria-label="Project actions">
            <button type="button" disabled={busy} onClick={onNew}>New project</button>
            <button type="button" disabled={busy} onClick={onExport}>Export project</button>
            <label className="button" aria-disabled={busy} htmlFor={importId}>
              Import project
            </label>
            <input
              id={importId}
              className="project-dialog__file"
              type="file"
              accept=".json,.m2a-project.json,application/json"
              aria-label="Import project backup file"
              disabled={busy}
              onChange={(event) => selectImport(event, onImport)}
            />
            <button type="button" disabled={busy} onClick={onDuplicate}>Duplicate project</button>
            {confirmDelete ? (
              <>
                <button
                  type="button"
                  className="project-dialog__danger"
                  disabled={busy}
                  onClick={onDelete}
                >
                  Confirm delete
                </button>
                <button type="button" onClick={() => setConfirmDelete(false)}>Cancel</button>
              </>
            ) : (
              <button
                type="button"
                className="project-dialog__danger"
                disabled={busy}
                onClick={() => setConfirmDelete(true)}
              >
                Delete project
              </button>
            )}
          </div>
          <p className="project-dialog__privacy">
            Backups contain metadata and authoring JSON only. Rebind the exact
            GLB/2DA files by SHA-256 after import or recovery.
          </p>
          {error ? <p className="project-dialog__error" role="alert">{error}</p> : null}
        </section>

        <section className="project-dialog__recovery" aria-labelledby={`${headingId}-recovery`}>
          <header>
            <h3 id={`${headingId}-recovery`}>Local recovery</h3>
            <span>{recoveryRecords.length} records</span>
          </header>
          {recoveryRecords.length === 0 ? (
            <p>No local recovery records yet.</p>
          ) : (
            <ul>
              {recoveryRecords.map((record) => (
                <li key={record.projectId} data-recoverable={record.recoverable}>
                  <div>
                    <strong>{record.name}</strong>
                    <span>
                      {record.target ?? "Invalid"} · revision {record.revision ?? "?"}
                    </span>
                    <code>{record.projectId}</code>
                    {!record.recoverable ? (
                      <span role="alert">
                        {record.diagnostics[0]?.message ?? "Record is not recoverable."}
                      </span>
                    ) : null}
                  </div>
                  <div>
                    <button
                      type="button"
                      disabled={busy || !record.recoverable}
                      onClick={() => onRecover(record.projectId)}
                    >
                      Recover
                    </button>
                    <button
                      type="button"
                      className="project-dialog__danger"
                      disabled={busy}
                      onClick={() => onDeleteRecovery(record.projectId)}
                    >
                      Delete record
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}
