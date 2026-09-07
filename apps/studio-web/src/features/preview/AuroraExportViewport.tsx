import { useCallback, useMemo } from "react";
import * as THREE from "three";
import { TGALoader } from "three/addons/loaders/TGALoader.js";
import type { WorkerArtifact } from "../../worker/types";
import { verifyWorkerArtifactV1 } from "../downloads/ArtifactDownloads";
import { projectCanonicalReadback } from "../results/projectReadback";
import {
  buildAuroraReadbackAsset,
  type AuroraReadbackMaterialResolver,
} from "./AuroraReadbackViewport";
import { SceneViewport } from "./SceneViewport";
import type { BinaryMdlInspectionReport, ModelPartRef, ReadbackNode } from "./types";

interface Props {
  readonly report: BinaryMdlInspectionReport;
  readonly artifacts: readonly WorkerArtifact[];
  readonly selectedPart?: ModelPartRef;
  readonly onSelectPart: (part?: ModelPartRef) => void;
  readonly onError?: (message: string) => void;
}

function resref(fileName: string) {
  return fileName.replace(/\.[^.]+$/, "").toLowerCase();
}

function firstController(node: ReadbackNode, name: string) {
  return node.controllers.find((controller) => controller.controllerName === name)?.values[0];
}

export interface AuroraExportMtrStateV1 {
  readonly textures: ReadonlyMap<number, string>;
  readonly renderHint: "normal" | "normalandspecmapped" | "normaltangents";
  readonly transparency: boolean;
  readonly twoSided: boolean;
  readonly blending?: "punchthrough";
}

function parseAuroraExportMtrV1(bytes: ArrayBuffer, fileName: string): AuroraExportMtrStateV1 {
  const text = new TextDecoder("ascii", { fatal: true }).decode(bytes);
  if (!text.endsWith("\n") || text.includes("\r") || text.includes("\0")) {
    throw new Error(`AURORA-EXPORT-MTR-NON-CANONICAL: ${fileName}`);
  }
  const textures = new Map<number, string>();
  let renderHint: AuroraExportMtrStateV1["renderHint"] | undefined;
  let transparency: boolean | undefined;
  let twoSided: boolean | undefined;
  let blending: AuroraExportMtrStateV1["blending"];
  for (const [index, line] of text.trimEnd().split("\n").entries()) {
    const fields = line.split(" ");
    if (fields.length !== 2 || fields.some((field) => field.length === 0)) {
      throw new Error(`AURORA-EXPORT-MTR-NON-CANONICAL: ${fileName}:${index + 1}`);
    }
    const [directive, value] = fields;
    const textureMatch = /^texture(\d|10)$/.exec(directive);
    if (textureMatch) {
      const slot = Number(textureMatch[1]);
      if (textures.has(slot) || !/^[a-z0-9_]{1,16}$/.test(value)) {
        throw new Error(`AURORA-EXPORT-MTR-TEXTURE-INVALID: ${fileName}:${index + 1}`);
      }
      textures.set(slot, value);
    } else if (directive === "renderhint" && renderHint === undefined
      && ["normal", "normalandspecmapped", "normaltangents"].includes(value)) {
      renderHint = value as AuroraExportMtrStateV1["renderHint"];
    } else if (directive === "transparency" && transparency === undefined && (value === "0" || value === "1")) {
      transparency = value === "1";
    } else if (directive === "twosided" && twoSided === undefined && (value === "0" || value === "1")) {
      twoSided = value === "1";
    } else if (directive === "blending" && blending === undefined && value === "punchthrough") {
      blending = "punchthrough";
    } else {
      throw new Error(`AURORA-EXPORT-MTR-DIRECTIVE-INVALID: ${fileName}:${index + 1}`);
    }
  }
  if (renderHint === undefined || transparency === undefined || twoSided === undefined) {
    throw new Error(`AURORA-EXPORT-MTR-INCOMPLETE: ${fileName}`);
  }
  return { textures, renderHint, transparency, twoSided, blending };
}

export async function exactAuroraExportMaterialsV1(artifacts: readonly WorkerArtifact[]) {
  const materials = new Map<string, AuroraExportMtrStateV1>();
  for (const artifact of artifacts.filter((item) => item.kind === "MATERIAL")) {
    await verifyWorkerArtifactV1(artifact);
    const key = resref(artifact.fileName);
    if (materials.has(key)) throw new Error(`AURORA-EXPORT-MTR-DUPLICATE: ${key}`);
    materials.set(key, parseAuroraExportMtrV1(artifact.bytes, artifact.fileName));
  }
  return materials;
}

export async function exactAuroraExportTexturesV1(artifacts: readonly WorkerArtifact[]) {
  const textures = new Map<string, THREE.DataTexture>();
  const loader = new TGALoader();
  for (const artifact of artifacts.filter((item) => item.kind === "TEXTURE" && item.fileName.toLowerCase().endsWith(".tga"))) {
    await verifyWorkerArtifactV1(artifact);
    const key = resref(artifact.fileName);
    if (textures.has(key)) throw new Error(`AURORA-EXPORT-TEXTURE-DUPLICATE: ${key}`);
    const parsed = loader.parse(artifact.bytes);
    const texture = new THREE.DataTexture(parsed.data, parsed.width, parsed.height, parsed.format, parsed.type);
    const isDataMap = /_[ns][0-9a-f]+\.tga$/i.test(artifact.fileName);
    texture.colorSpace = isDataMap ? THREE.NoColorSpace : THREE.SRGBColorSpace;
    texture.wrapS = THREE.RepeatWrapping;
    texture.wrapT = THREE.RepeatWrapping;
    texture.minFilter = THREE.LinearMipmapLinearFilter;
    texture.magFilter = THREE.LinearFilter;
    texture.generateMipmaps = true;
    texture.needsUpdate = true;
    texture.name = artifact.fileName;
    texture.userData = {
      provenance: isDataMap ? "AURORA_EXPORT_EXACT_DATA_TGA_V1" : "AURORA_EXPORT_EXACT_COLOR_TGA_V1",
      artifactId: artifact.artifactId,
      sha256: artifact.sha256,
    };
    textures.set(key, texture);
  }
  return textures;
}

export async function exactAuroraExportLineageV1(
  report: BinaryMdlInspectionReport,
  artifacts: readonly WorkerArtifact[],
) {
  const models = artifacts.filter((artifact) => artifact.kind === "MODEL");
  if (models.length !== 1) {
    throw new Error(`AURORA-EXPORT-MODEL-INVENTORY: expected 1 MODEL artifact, received ${models.length}`);
  }
  const model = models[0];
  await verifyWorkerArtifactV1(model);

  if (model.artifactId !== "placeable-model-mdl") {
    return { modelSha256: model.sha256 };
  }
  const readbacks = artifacts.filter(
    (artifact) => artifact.artifactId === "placeable-model-readback-json",
  );
  if (readbacks.length !== 1) {
    throw new Error(
      `AURORA-EXPORT-READBACK-INVENTORY: expected 1 Placeable readback artifact, received ${readbacks.length}`,
    );
  }
  const readback = readbacks[0];
  await verifyWorkerArtifactV1(readback);
  let exactReport: BinaryMdlInspectionReport;
  try {
    exactReport = projectCanonicalReadback(
      new TextDecoder("utf-8", { fatal: true }).decode(readback.bytes),
    );
  } catch {
    throw new Error("AURORA-EXPORT-READBACK-INVALID");
  }
  if (JSON.stringify(exactReport) !== JSON.stringify(report)) {
    throw new Error("AURORA-EXPORT-READBACK-MISMATCH");
  }
  return { modelSha256: model.sha256, readbackSha256: readback.sha256 };
}

export function AuroraExportViewport({
  report,
  artifacts,
  selectedPart,
  onSelectPart,
  onError,
}: Props) {
  const identity = useMemo(() => artifacts
    .filter((artifact) => ["MODEL", "TEXTURE", "MATERIAL", "TEXTURE_INFO"].includes(artifact.kind)
      || artifact.artifactId === "placeable-model-readback-json")
    .map((artifact) => `${artifact.artifactId}:${artifact.sha256}`)
    .sort()
    .join("|"), [artifacts]);
  const buildRoot = useCallback(async () => {
    await exactAuroraExportLineageV1(report, artifacts);
    const [textures, materials] = await Promise.all([
      exactAuroraExportTexturesV1(artifacts),
      exactAuroraExportMaterialsV1(artifacts),
    ]);
    const resolver: AuroraReadbackMaterialResolver = (mesh, node) => {
      const materialResref = mesh.textures?.[3]?.toLowerCase() ?? "";
      const materialState = materialResref ? materials.get(materialResref) : undefined;
      if (materialResref && !materialState) {
        throw new Error(`AURORA-EXPORT-MTR-MISSING: ${materialResref}.mtr is not present in exact worker artifacts`);
      }
      const textureResref = mesh.textures?.[0]?.toLowerCase() ?? "";
      const map = textures.get(textureResref);
      const normalResref = mesh.textures?.[1]?.toLowerCase() ?? "";
      const specularResref = mesh.textures?.[2]?.toLowerCase() ?? "";
      const normalMap = normalResref ? textures.get(normalResref) : undefined;
      const specularMap = specularResref ? textures.get(specularResref) : undefined;
      if (textureResref && !map) {
        throw new Error(`AURORA-EXPORT-TEXTURE-MISSING: ${textureResref}.tga is not present in exact worker artifacts`);
      }
      if (normalResref && !normalMap) throw new Error(`AURORA-EXPORT-TEXTURE-MISSING: ${normalResref}.tga`);
      if (specularResref && !specularMap) throw new Error(`AURORA-EXPORT-TEXTURE-MISSING: ${specularResref}.tga`);
      for (const [slot, mdlResref] of [[0, textureResref], [1, normalResref], [2, specularResref]] as const) {
        const mtrResref = materialState?.textures.get(slot) ?? "";
        if (mtrResref && mdlResref && mtrResref !== mdlResref) {
          throw new Error(`AURORA-EXPORT-MATERIAL-SEMANTIC-MISMATCH: texture${slot} is ${mdlResref} in MDL and ${mtrResref} in MTR`);
        }
      }
      const alpha = firstController(node, "alpha")?.[0] ?? 1;
      const selfIllum = firstController(node, "selfIllumination") ?? [0, 0, 0];
      const diffuse = mesh.diffuse ?? [1, 1, 1];
      const specular = mesh.specular ?? [0, 0, 0];
      const materialOptions: THREE.MeshPhongMaterialParameters = {
        map,
        color: new THREE.Color(...diffuse),
        emissive: new THREE.Color(selfIllum[0] ?? 0, selfIllum[1] ?? 0, selfIllum[2] ?? 0),
        specular: new THREE.Color(...specular),
        shininess: mesh.shininess ?? 1,
        opacity: alpha,
        transparent: alpha < 1 || materialState?.transparency === true,
        alphaTest: materialState?.blending === "punchthrough" ? 0.5 : 0,
        side: materialState?.twoSided === true ? THREE.DoubleSide : THREE.FrontSide,
      };
      if (normalMap) materialOptions.normalMap = normalMap;
      if (specularMap) materialOptions.specularMap = specularMap;
      const material = new THREE.MeshPhongMaterial(materialOptions);
      material.name = `aurora-export:${textureResref || "untextured"}`;
      material.userData = {
        provenance: "FINAL_BINARY_MDL_READBACK_PLUS_EXACT_TGA_V1",
        textureResref,
        renderHint: mesh.renderHint ?? 0,
        normalResref,
        specularResref,
        materialResref,
        materialState,
      };
      return material;
    };
    return buildAuroraReadbackAsset(report, selectedPart, undefined, resolver);
  }, [artifacts, report, selectedPart]);
  return (
    <section aria-label="Aurora Export viewport">
      <header className="aurora-export-viewport__header">
        <strong>Aurora Export</strong>
        <span>Final binary MDL readback + exact exported TGA artifacts</span>
      </header>
      <SceneViewport
        provenance="READBACK"
        detail="Aurora classic material preview reconstructed only from final binary-MDL readback and hash-verified export resources"
        dependency={`aurora-export:${report.format}:${report.schemaVersion}:${identity}:${selectedPart?.kind}:${selectedPart?.id}`}
        buildRoot={buildRoot}
        onSelectPart={onSelectPart}
        onError={onError}
        tools={{ animationPlayback: true, overlays: false }}
      />
    </section>
  );
}
