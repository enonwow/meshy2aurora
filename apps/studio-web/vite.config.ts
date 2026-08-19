import { createReadStream } from "node:fs";
import { resolve, sep } from "node:path";
import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

const workspaceRoot = fileURLToPath(new URL("../..", import.meta.url));
const linkedWorktreeMarker = `${sep}.worktrees${sep}`;
const linkedWorktreeIndex = workspaceRoot.indexOf(linkedWorktreeMarker);
const canonicalRepositoryRoot = linkedWorktreeIndex >= 0
  ? workspaceRoot.slice(0, linkedWorktreeIndex)
  : workspaceRoot;

function exactHextechShotgunProofAssets(): Plugin {
  const assetRoot = resolve(
    canonicalRepositoryRoot,
    "sample-3d",
    "tlc-hextech-shotgun-parts-v1",
  );
  const allowed = new Map([
    ["bottom.glb", resolve(assetRoot, "bottom.glb")],
    ["middle.glb", resolve(assetRoot, "middle.glb")],
    ["top.glb", resolve(assetRoot, "top.glb")],
  ]);
  const prefix = "/__m2a-proof/hextech-shotgun/";

  return {
    name: "m2a-exact-hextech-shotgun-proof-assets",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const requestUrl = request.url ? new URL(request.url, "http://localhost") : undefined;
        if (!requestUrl?.pathname.startsWith(prefix)) {
          next();
          return;
        }
        const fileName = requestUrl.pathname.slice(prefix.length);
        const source = allowed.get(fileName);
        if (!source) {
          response.statusCode = 404;
          response.end("Unknown proof asset");
          return;
        }
        response.statusCode = 200;
        response.setHeader("Content-Type", "model/gltf-binary");
        response.setHeader("Cache-Control", "no-store");
        createReadStream(source)
          .on("error", () => {
            if (!response.headersSent) response.statusCode = 404;
            response.end("Canonical proof asset unavailable");
          })
          .pipe(response);
      });
    },
  };
}

export default defineConfig({
  base: "./",
  plugins: [exactHextechShotgunProofAssets(), react()],
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
});
