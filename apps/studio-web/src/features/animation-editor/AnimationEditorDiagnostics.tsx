import type { AnimationStudioDiagnosticV1 } from "../animation-studio/types";

export function AnimationEditorDiagnostics({
  diagnostics,
}: {
  diagnostics: readonly AnimationStudioDiagnosticV1[];
}) {
  const hasBlocking = diagnostics.some(({ level }) => level === "BLOCKING");
  return (
    <section
      className="animation-editor-diagnostics"
      aria-labelledby="animation-diagnostics-title"
    >
      <h3 id="animation-diagnostics-title">Editor diagnostics</h3>
      <div
        role={hasBlocking ? "alert" : "status"}
        aria-live={hasBlocking ? "assertive" : "polite"}
        aria-atomic="false"
      >
        {diagnostics.length === 0 ? (
          <p>Aurora MDL constraints active · no current blocker.</p>
        ) : (
          <ul aria-label="Animation Studio diagnostics">
            {diagnostics.map((item) => (
              <li key={`${item.code}:${item.path}`}>
                <strong>{item.code}</strong>: {item.message}{" "}
                <span>{item.action}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
