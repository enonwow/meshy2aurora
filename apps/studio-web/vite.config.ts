import { fileURLToPath, URL } from "node:url";
import { createReadStream, statSync } from "node:fs";
import { resolve, sep } from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

const workspaceRoot = fileURLToPath(new URL("../..", import.meta.url));
const localNwnReferenceRoot = process.env.M2A_NWN_REFERENCE_ROOT;
const localNwnUserReferenceRoot = process.env.M2A_NWN_USER_ROOT;

function localNwnReferencePlugin(
  rootPath: string | undefined,
  mountPath = "/__m2a_nwn_reference",
  name = "meshy2aurora-local-nwn-reference",
): Plugin {
  return {
    name,
    configureServer(server) {
      if (!rootPath) return;
      const root = resolve(rootPath);
      server.middlewares.use(mountPath, (request, response) => {
        try {
          const pathname = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname)
            .replace(/\\/g, "/");
          const parts = pathname.split("/").filter(Boolean);
          if (!parts.length || parts.some((part) => part === "." || part === "..")) {
            response.statusCode = 400;
            response.end("Unsafe NWN reference path");
            return;
          }
          const filePath = resolve(root, ...parts);
          if (filePath !== root && !filePath.startsWith(`${root}${sep}`)) {
            response.statusCode = 403;
            response.end("NWN reference path escaped its root");
            return;
          }
          const size = statSync(filePath).size;
          const range = request.headers.range?.match(/^bytes=(\d+)-(\d+)$/);
          response.setHeader("Accept-Ranges", "bytes");
          response.setHeader("Content-Type", "application/octet-stream");
          if (range) {
            const start = Number(range[1]);
            const end = Number(range[2]);
            if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || end < start || end >= size) {
              response.statusCode = 416;
              response.setHeader("Content-Range", `bytes */${size}`);
              response.end();
              return;
            }
            response.statusCode = 206;
            response.setHeader("Content-Length", String(end - start + 1));
            response.setHeader("Content-Range", `bytes ${start}-${end}/${size}`);
            createReadStream(filePath, { start, end }).pipe(response);
            return;
          }
          response.statusCode = 200;
          response.setHeader("Content-Length", String(size));
          createReadStream(filePath).pipe(response);
        } catch {
          response.statusCode = 404;
          response.end("NWN reference resource not found");
        }
      });
    },
  };
}

export default defineConfig({
  base: "./",
  plugins: [
    react(),
    localNwnReferencePlugin(localNwnReferenceRoot),
    localNwnReferencePlugin(
      localNwnUserReferenceRoot,
      "/__m2a_nwn_user_reference",
      "meshy2aurora-local-nwn-user-reference",
    ),
  ],
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
      allow: [workspaceRoot],
    },
  },
  worker: { format: "es" },
});
