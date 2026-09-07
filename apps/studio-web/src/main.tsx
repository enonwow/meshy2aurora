import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { MaterialSeparationDemo } from "./features/material-separation/MaterialSeparationDemo";
import { RetailHakSnakeLibrary } from "./features/supermodels/RetailHakSnakeLibrary";
import { RetailSupermodelDiagnostic } from "./features/supermodels/RetailSupermodelDiagnostic";
import { SavedSupermodelDiagnostic } from "./features/supermodels/SavedSupermodelDiagnostic";
import { CreatureAutomationWorkbench } from "./features/creature-automation/CreatureAutomationWorkbench";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Studio root element is missing");

const params = new URLSearchParams(window.location.search);
const savedSupermodelDiagnostic = params.get("supermodelDiagnostic");
const retailSupermodel = params.get("retailSupermodel");
const nwnRoot = params.get("nwnRoot");
const retailHakSnakes = params.get("retailHakSnakes");
const hakRoot = params.get("hakRoot");
const hakPath = params.get("hakPath");
const textureHakPath = params.get("textureHakPath") ?? "hak/cep3_core0.hak";

createRoot(root).render(
  <StrictMode>
    {params.get("creatureAgent") === "1"
      ? <CreatureAutomationWorkbench />
      : retailHakSnakes === "1" && hakRoot && hakPath
      ? <RetailHakSnakeLibrary hakRootUrl={hakRoot} hakPath={hakPath} textureHakPath={textureHakPath} />
      : retailSupermodel && nwnRoot
      ? <RetailSupermodelDiagnostic nwnRootUrl={nwnRoot} resref={retailSupermodel} />
      : savedSupermodelDiagnostic
      ? <SavedSupermodelDiagnostic baseUrl={savedSupermodelDiagnostic} />
      : params.get("materialDemo") === "1"
      ? <MaterialSeparationDemo />
      : <App />}
  </StrictMode>,
);
