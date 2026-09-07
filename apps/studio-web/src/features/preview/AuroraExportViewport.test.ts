import { describe, expect, it } from "vitest";
import type { WorkerArtifact } from "../../worker/types";
import { projectCanonicalReadback } from "../results/projectReadback";
import {
  exactAuroraExportLineageV1,
  exactAuroraExportMaterialsV1,
  exactAuroraExportTexturesV1,
} from "./AuroraExportViewport";

const onePixelTga = new Uint8Array([
  0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  1, 0, 1, 0, 24, 32,
  30, 20, 10,
]);

function textureArtifact(bytes = onePixelTga): WorkerArtifact {
  return {
    artifactId: "diffuse-tga",
    kind: "TEXTURE",
    fileName: "M2AD0.TGA",
    mediaType: "image/x-tga",
    byteLength: bytes.byteLength,
    sha256: "853f32d5ae24754cabf599fbc30b9e8b82f1b19a73fa12ddb1ca6dec4e62571d",
    bytes: bytes.slice().buffer,
    provenance: "M2A_WASM_WORKER",
  };
}

async function exactArtifact(
  artifactId: string,
  kind: WorkerArtifact["kind"],
  fileName: string,
  bytes: Uint8Array,
): Promise<WorkerArtifact> {
  const payload = bytes.slice().buffer as ArrayBuffer;
  const digest = await crypto.subtle.digest("SHA-256", payload);
  return {
    artifactId,
    kind,
    fileName,
    mediaType: kind === "JSON_REPORT" ? "application/json" : "application/octet-stream",
    byteLength: bytes.byteLength,
    sha256: [...new Uint8Array(digest)]
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join(""),
    bytes: payload,
    provenance: "M2A_WASM_WORKER",
  };
}

describe("Aurora Export exact resource resolver", () => {
  it("loads a hash-verified exported TGA under its canonical lowercase resref", async () => {
    const textures = await exactAuroraExportTexturesV1([textureArtifact()]);
    const texture = textures.get("m2ad0");

    expect(texture).toBeDefined();
    expect(texture?.image.width).toBe(1);
    expect(texture?.image.height).toBe(1);
    expect(texture?.userData).toEqual({
      provenance: "AURORA_EXPORT_EXACT_COLOR_TGA_V1",
      artifactId: "diffuse-tga",
      sha256: "853f32d5ae24754cabf599fbc30b9e8b82f1b19a73fa12ddb1ca6dec4e62571d",
    });
  });

  it("parses hash-verified MTR render semantics used by the export viewport", async () => {
    const bytes = new TextEncoder().encode([
      "texture0 m2ad0",
      "texture1 m2an0",
      "renderhint normalandspecmapped",
      "transparency 0",
      "twosided 1",
      "blending punchthrough",
      "",
    ].join("\n"));
    const hash = await crypto.subtle.digest("SHA-256", bytes);
    const sha256 = [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    const artifact: WorkerArtifact = {
      artifactId: "material-mtr",
      kind: "MATERIAL",
      fileName: "m2am0.mtr",
      mediaType: "text/plain",
      byteLength: bytes.byteLength,
      sha256,
      bytes: bytes.buffer,
      provenance: "M2A_WASM_WORKER",
    };

    const materials = await exactAuroraExportMaterialsV1([artifact]);
    expect(materials.get("m2am0")).toMatchObject({
      renderHint: "normalandspecmapped",
      transparency: false,
      twoSided: true,
      blending: "punchthrough",
    });
    expect(materials.get("m2am0")?.textures.get(1)).toBe("m2an0");
  });

  it("fails closed when artifact bytes do not match their declared hash", async () => {
    const corrupted = textureArtifact(new Uint8Array([...onePixelTga.slice(0, -1), 11]));
    await expect(exactAuroraExportTexturesV1([corrupted])).rejects.toThrow("SHA-256 mismatch");
  });

  it("rejects duplicate texture resrefs instead of silently choosing one payload", async () => {
    const duplicate = { ...textureArtifact(), artifactId: "duplicate-diffuse-tga" };
    await expect(exactAuroraExportTexturesV1([textureArtifact(), duplicate]))
      .rejects.toThrow("AURORA-EXPORT-TEXTURE-DUPLICATE: m2ad0");
  });

  it("binds a Placeable preview to one hash-verified MDL and its exact readback artifact", async () => {
    const readbackJson = JSON.stringify({
      schemaVersion: 1,
      format: "nwn1-binary-mdl",
      nodeTree: { roots: [{ offset: 12, number: 1, name: "ship", controllers: [], children: [] }] },
      animations: [],
      diagnostics: [],
    });
    const report = projectCanonicalReadback(readbackJson);
    const model = await exactArtifact(
      "placeable-model-mdl",
      "MODEL",
      "ship.mdl",
      new Uint8Array([1, 2, 3]),
    );
    const readback = await exactArtifact(
      "placeable-model-readback-json",
      "JSON_REPORT",
      "placeable-model-readback.json",
      new TextEncoder().encode(readbackJson),
    );

    await expect(exactAuroraExportLineageV1(report, [model, readback])).resolves.toEqual({
      modelSha256: model.sha256,
      readbackSha256: readback.sha256,
    });

    const otherReadbackJson = readbackJson.replace('"name":"ship"', '"name":"other"');
    const otherReadback = await exactArtifact(
      "placeable-model-readback-json",
      "JSON_REPORT",
      "placeable-model-readback.json",
      new TextEncoder().encode(otherReadbackJson),
    );
    await expect(exactAuroraExportLineageV1(report, [model, otherReadback]))
      .rejects.toThrow("AURORA-EXPORT-READBACK-MISMATCH");
  });
});
