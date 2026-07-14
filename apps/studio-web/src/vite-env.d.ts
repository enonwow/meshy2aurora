/// <reference types="vite/client" />

declare module "@m2a-wasm" {
  export default function init(): Promise<unknown>;
  export function ingestGlbJson(bytes: Uint8Array): string;
  export function validateM7CorpusManifestV1Json(manifestJson: string): string;
  export function inspectM7CorpusIntakeV1Json(
    manifestJson: string,
    payloadBlob: Uint8Array,
    descriptorsJson: string,
  ): string;
  export function buildM7CorpusBatchV1(
    manifestJson: string,
    payloadBlob: Uint8Array,
    descriptorsJson: string,
  ): string;
  export function buildM6ModelPackageV1(
    sourceGlb: Uint8Array,
    appearanceTwoDa: Uint8Array,
  ): {
    readonly reportJson: string;
    readonly manifestJson: string;
    readonly summaryJson: string;
    readonly readbackJson: string;
    takeHakBytes(): Uint8Array;
    takeModelBytes(): Uint8Array;
    free(): void;
  };
}
