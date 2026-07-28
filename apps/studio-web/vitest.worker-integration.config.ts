import { fileURLToPath } from "node:url";
import { basename, dirname } from "node:path";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";
import viteConfig from "./vite.config";

const worktreeRepositoryRoot = fileURLToPath(new URL("../..", import.meta.url));
const worktreesDirectory = dirname(worktreeRepositoryRoot);
const canonicalRepositoryRoot =
  basename(worktreesDirectory).toLowerCase() === ".worktrees"
    ? dirname(worktreesDirectory)
    : worktreeRepositoryRoot;

export default defineConfig({
  ...viteConfig,
  resolve: {
    alias: {
      "@m2a-wasm": fileURLToPath(
        new URL("../../crates/m2a-wasm/pkg/m2a_wasm.js", import.meta.url),
      ),
      "@m2a-canonical-repository": canonicalRepositoryRoot,
    },
  },
  server: {
    fs: {
      allow: [worktreeRepositoryRoot, canonicalRepositoryRoot],
    },
  },
  test: {
    include: ["tests/browser/**/*.integration.{ts,tsx}"],
    fileParallelism: false,
    browser: {
      enabled: true,
      headless: true,
      provider: playwright({
        launchOptions: {
          channel: process.env.M2A_BROWSER_CHANNEL ?? "chrome",
          args: [
            "--disable-background-networking",
            "--disable-component-update",
            "--disable-default-apps",
            "--disable-sync",
          ],
        },
      }),
      instances: [{ browser: "chromium" }],
    },
  },
});
