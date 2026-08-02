import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Studio root element is missing");

const visualQaFixture = import.meta.env.DEV
  && new URLSearchParams(window.location.search).get("visualQaFixture")
    === "vck-authored-sword"
  ? {
      label: "Load Void Crystal Knight visual QA fixture",
      load: async () => {
        const { loadVoidCrystalKnightVisualQaFixture } = await import(
          "./dev/voidCrystalKnightVisualQa"
        );
        return loadVoidCrystalKnightVisualQaFixture();
      },
    }
  : undefined;

createRoot(root).render(
  <StrictMode>
    <App visualQaFixture={visualQaFixture} />
  </StrictMode>,
);
