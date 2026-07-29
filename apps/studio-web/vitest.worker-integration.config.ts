import { createHash } from "node:crypto";
import { writeFile } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";
import type { BrowserCommand } from "vitest/node";
import viteConfig from "./vite.config";

const repositoryRoot = fileURLToPath(new URL("../..", import.meta.url));
const worktreesDirectory = dirname(repositoryRoot);
const canonicalRepositoryRoot =
  basename(worktreesDirectory).toLowerCase() === ".worktrees"
    ? dirname(worktreesDirectory)
    : repositoryRoot;
const p300kCaptureDirectory = resolve(
  repositoryRoot,
  "proof-output/tlc-stoneback-brute-p300k-geometry-ab-v5/studio-export",
);
const p300kCaptureFileNames = new Set([
  "m2p3jd0eeb135c.mod",
  "m2p3jh0eeb135c.hak",
  "m2p3jm0eeb135c.mdl",
  "m2p3jt0eeb135c.tga",
  "inspection.json",
  "conversion-manifest.json",
  "summary.json",
]);
const p300kCaptureBuffers = new Map<
  string,
  {
    chunks: Buffer[];
    expectedSha256: string;
    receivedBytes: number;
    totalBytes: number;
  }
>();
const captureP300kArtifact: BrowserCommand<
  [
    fileName: string,
    chunkBase64: string,
    offset: number,
    totalBytes: number,
    expectedSha256: string,
  ],
  { complete: boolean; receivedBytes: number; sha256?: string }
> = async (
  _context,
  fileName,
  chunkBase64,
  offset,
  totalBytes,
  expectedSha256,
) => {
  if (process.env.M2A_CAPTURE_P300K_ARTIFACTS !== "1") {
    throw new Error("P300K artifact capture is disabled");
  }
  if (!p300kCaptureFileNames.has(fileName)) {
    throw new Error(`P300K artifact capture rejected filename: ${fileName}`);
  }
  const outputPath = resolve(p300kCaptureDirectory, fileName);
  if (dirname(outputPath) !== p300kCaptureDirectory) {
    throw new Error(`P300K artifact capture escaped its exact directory: ${fileName}`);
  }
  if (
    !Number.isSafeInteger(offset)
    || !Number.isSafeInteger(totalBytes)
    || offset < 0
    || totalBytes <= 0
    || totalBytes > 100_000_000
    || !/^[a-f0-9]{64}$/.test(expectedSha256)
  ) {
    throw new Error(`P300K artifact capture rejected metadata: ${fileName}`);
  }
  const chunk = Buffer.from(chunkBase64, "base64");
  const capture = p300kCaptureBuffers.get(fileName) ?? {
    chunks: [],
    expectedSha256,
    receivedBytes: 0,
    totalBytes,
  };
  if (
    capture.expectedSha256 !== expectedSha256
    || capture.totalBytes !== totalBytes
    || capture.receivedBytes !== offset
    || capture.receivedBytes + chunk.byteLength > totalBytes
  ) {
    throw new Error(`P300K artifact capture rejected chunk order: ${fileName}`);
  }
  capture.chunks.push(chunk);
  capture.receivedBytes += chunk.byteLength;
  p300kCaptureBuffers.set(fileName, capture);
  if (capture.receivedBytes < totalBytes) {
    return { complete: false, receivedBytes: capture.receivedBytes };
  }
  const payload = Buffer.concat(capture.chunks, totalBytes);
  const sha256 = createHash("sha256").update(payload).digest("hex");
  if (sha256 !== expectedSha256) {
    throw new Error(`P300K artifact capture SHA-256 mismatch: ${fileName}`);
  }
  await writeFile(outputPath, payload, { flag: "wx" });
  p300kCaptureBuffers.delete(fileName);
  return { complete: true, receivedBytes: payload.byteLength, sha256 };
};

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
      allow: [repositoryRoot, canonicalRepositoryRoot],
    },
  },
  test: {
    include: ["tests/browser/**/*.integration.{ts,tsx}"],
    fileParallelism: false,
    browser: {
      commands: {
        captureP300kArtifact,
      },
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
