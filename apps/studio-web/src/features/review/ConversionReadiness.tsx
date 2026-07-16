import type { BinaryMdlInspectionReport } from "../preview/types";
import type { CanonicalResultSnapshot } from "../results/projectCanonicalResult";
import { projectConversionReadiness } from "./projectConversionReadiness";
import "./ConversionReadiness.css";

export function ConversionReadiness({ result, readback }: {
  result: CanonicalResultSnapshot;
  readback: BinaryMdlInspectionReport;
}) {
  const projection = projectConversionReadiness(result, readback);
  const blocking = !projection.conversionEligible || projection.items.some(({ status }) => status === "FAIL");
  const warning = projection.items.some(({ status }) => status === "WARNING");
  const unchecked = projection.items.some(({ status }) => status === "NOT_CHECKED");
  const headline = blocking
    ? "FAILED"
    : warning
      ? "ELIGIBLE WITH WARNINGS"
      : unchecked
        ? "PARTIALLY CHECKED"
        : "STRUCTURAL CHECKS PASS";

  return (
    <section className="conversion-readiness" aria-labelledby="conversion-readiness-heading">
      <header>
        <div>
          <p className="eyebrow">Canonical checks</p>
          <h2 id="conversion-readiness-heading">Conversion Readiness</h2>
        </div>
        <strong data-status={blocking ? "fail" : warning ? "warning" : unchecked ? "not_checked" : "pass"}>{headline}</strong>
      </header>

      <ul className="conversion-readiness__categories">
        {projection.items.map((item) => (
          <li key={item.id} data-status={item.status.toLowerCase()}>
            <div><strong>{item.label}</strong><small>{item.detail}</small></div>
            <span>{item.statusLabel}</span>
            <output>{item.checkCount} check(s)</output>
          </li>
        ))}
      </ul>

      <section className="conversion-validation" aria-labelledby="conversion-validation-heading">
        <header><h3 id="conversion-validation-heading">Validation</h3><span>{projection.validation.length} evidence item(s)</span></header>
        {projection.validation.length === 0 ? (
          <p>No warning, failure or informational evidence was emitted by the canonical contracts.</p>
        ) : (
          <div className="conversion-validation__table-wrap">
            <table>
              <thead><tr><th>Severity</th><th>Code</th><th>Path</th><th>Expected</th><th>Actual</th><th>Message</th></tr></thead>
              <tbody>{projection.validation.map((item) => (
                <tr key={item.id} data-severity={item.severity.toLowerCase()}>
                  <td>{item.severity}</td><td><code>{item.code}</code></td><td><code>{item.path}</code></td>
                  <td>{item.expected ?? "—"}</td><td>{item.actual ?? "—"}</td><td>{item.message}</td>
                </tr>
              ))}</tbody>
            </table>
          </div>
        )}
      </section>
    </section>
  );
}
