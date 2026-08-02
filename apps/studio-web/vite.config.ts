import { fileURLToPath, URL } from "node:url";
import { basename, dirname } from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const workspaceRoot = fileURLToPath(new URL("../..", import.meta.url));
const workspaceParent = dirname(workspaceRoot);
const canonicalRepositoryRoot = basename(workspaceParent) === ".worktrees"
  ? dirname(workspaceParent)
  : workspaceRoot;

export default defineConfig({
  base: "./",
  plugins: [react()],
  define: {
    __M2A_CANONICAL_REPOSITORY_ROOT__: JSON.stringify(
      canonicalRepositoryRoot.replaceAll("\\", "/"),
    ),
  },
  resolve: {
    alias: {
      "@m2a-wasm": fileURLToPath(
        new URL("../../crates/m2a-wasm/pkg/m2a_wasm.js", import.meta.url),
      ),
    },
  },
  server: {
    fs: {
      // The web-WASM package is built in the canonical workspace, outside
      // apps/studio-web. Keep Vite's dev-server file boundary explicit.
      allow: [workspaceRoot, canonicalRepositoryRoot],
    },
  },
  worker: { format: "es" },
  build: {
    // E6 measured product budget. A separate executable gate validates the
    // initial, lazy-feature, aggregate CSS/JS and WASM limits.
    chunkSizeWarningLimit: 620,
  },
});
