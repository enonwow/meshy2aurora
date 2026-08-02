import { afterEach, describe, expect, it } from "vitest";
import sourceUrl from "../.generated/owned-package/generated/source.glb?url";
import baseitemsUrl from "../fixtures/baseitems.2da?url";
import { StudioWorkerClient } from "../../src/worker/client";
import initWasm, { buildMeshyItemPartWithOptionsV2 } from "@m2a-wasm";

const clients: StudioWorkerClient[] = [];

async function fetchBytes(url: string): Promise<ArrayBuffer> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`fixture fetch failed: ${response.status} ${url}`);
  return response.arrayBuffer();
}

async function sha256(bytes: ArrayBuffer): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

let wasmReady: Promise<unknown> | undefined;

async function referenceMdlTemplate(sourceGlb: ArrayBuffer): Promise<ArrayBuffer> {
  wasmReady ??= initWasm();
  await wasmReady;
  const result = buildMeshyItemPartWithOptionsV2(
    new Uint8Array(sourceGlb),
    "retail_template",
    "retail_texture",
    JSON.stringify({
      schemaVersion: 1,
      transform: {
        translation: [0, 0, 0],
        rotationXyzw: [0, 0, 0, 1],
        uniformScale: 1,
        pivot: [0, 0, 0],
      },
      sourceNode: null,
      textureEncoding: "DIRECT_COLOR",
      iconSize: null,
      iconProjectionBounds: null,
    }),
  );
  try {
    return result.takeMdlBytes().slice().buffer;
  } finally {
    result.free();
  }
}

function renameBinaryMdl(template: ArrayBuffer, resref: string): ArrayBuffer {
  const bytes = new Uint8Array(template.slice(0));
  bytes.fill(0, 20, 84);
  bytes.set(new TextEncoder().encode(resref), 20);
  return bytes.buffer;
}

function onePixelRetailPlt(): ArrayBuffer {
  const bytes = new Uint8Array(26);
  bytes.set(new TextEncoder().encode("PLT V1  "), 0);
  const view = new DataView(bytes.buffer);
  view.setUint32(8, 10, true);
  view.setUint32(16, 1, true);
  view.setUint32(20, 1, true);
  return bytes.buffer;
}

function appearanceFixture(): ArrayBuffer {
  return twoDa([
    "2DA V2.0",
    "",
    "LABEL RACE MODELTYPE RACIALTYPE",
    "0 Dwarf D P 0",
    "1 Elf E P 1",
    "2 Gnome G P 2",
    "3 Halfling A P 3",
    "4 Half_Elf H P 4",
    "5 Half_Orc O P 5",
    "6 Human H P 6",
    "",
  ].join("\n"));
}

async function retailManifest(
  resources: Array<{
    resourceType: 6 | 2002;
    resref: string;
    fileName: string;
    bytes: ArrayBuffer;
  }>,
): Promise<ArrayBuffer> {
  const entries = await Promise.all(resources.map(async (resource, index) => ({
    resourceType: resource.resourceType,
    resref: resource.resref,
    fileName: resource.fileName,
    sha256: await sha256(resource.bytes),
    locator: `bif:1:resource:${index + 1}`,
  })));
  return new TextEncoder().encode(JSON.stringify({
    schemaVersion: 1,
    sourceKind: "NWN_BASE_KEY",
    sourceContainerFileName: "nwn_base.key",
    sourceContainerSha256: "09cdafb6dbfb9cdb544993154c514c65ef9fd5be4e0c1b8fd971ed05aff90935",
    resources: entries,
  })).buffer;
}

function asStaticItemPart(glb: ArrayBuffer): ArrayBuffer {
  const input = new Uint8Array(glb);
  const view = new DataView(glb);
  const jsonLength = view.getUint32(12, true);
  const jsonEnd = 20 + jsonLength;
  const root = JSON.parse(new TextDecoder().decode(input.slice(20, jsonEnd))) as {
    skins: unknown[];
    animations: unknown[];
    scenes: Array<{ nodes: number[] }>;
    nodes: Array<Record<string, unknown>>;
    meshes: Array<{ primitives: Array<{ attributes: Record<string, unknown> }> }>;
  };
  root.skins = [];
  root.animations = [];
  root.scenes[0].nodes = [0];
  root.nodes = [{ name: "item-part-source-root", mesh: 0 }];
  delete root.meshes[0].primitives[0].attributes.JOINTS_0;
  delete root.meshes[0].primitives[0].attributes.WEIGHTS_0;
  const json = new TextEncoder().encode(JSON.stringify(root));
  const paddedLength = (json.byteLength + 3) & ~3;
  const result = new Uint8Array(20 + paddedLength + input.byteLength - jsonEnd);
  result.set(new TextEncoder().encode("glTF"), 0);
  const outputView = new DataView(result.buffer);
  outputView.setUint32(4, 2, true);
  outputView.setUint32(8, result.byteLength, true);
  outputView.setUint32(12, paddedLength, true);
  result.set(new TextEncoder().encode("JSON"), 16);
  result.fill(0x20, 20, 20 + paddedLength);
  result.set(json, 20);
  result.set(input.slice(jsonEnd), 20 + paddedLength);
  return result.buffer;
}

const capartRows = [
  ["RFoot", "FOOTR", "rfoot_g"], ["LFoot", "FOOTL", "lfoot_g"],
  ["RShin", "SHINR", "rshin_g"], ["LShin", "SHINL", "lshin_g"],
  ["LThigh", "LEGL", "lthigh_g"], ["RThigh", "LEGR", "rthigh_g"],
  ["Pelvis", "PELVIS", "pelvis_g"], ["Torso", "CHEST", "torso_g"],
  ["Belt", "BELT", "belt_g"], ["Neck", "NECK", "neck_g"],
  ["RFArm", "FORER", "rforearm_g"], ["LFArm", "FOREL", "lforearm_g"],
  ["RBicep", "BICEPR", "rbicep_g"], ["LBicep", "BICEPL", "lbicep_g"],
  ["RShoul", "SHOR", "rshoulder_g"], ["LShoul", "SHOL", "lshoulder_g"],
  ["RHand", "HANDR", "rhand_g"], ["LHand", "HANDL", "lhand_g"],
  ["Robe", "ROBE", "root"],
] as const;

const capartPartTables = [
  "PARTS_FOOT", "PARTS_SHIN", "PARTS_LEGS", "PARTS_PELVIS", "PARTS_CHEST",
  "PARTS_BELT", "PARTS_NECK", "PARTS_FOREARM", "PARTS_BICEP", "PARTS_SHOULDER",
  "PARTS_HAND",
] as const;

function twoDa(text: string) {
  return new TextEncoder().encode(text).buffer;
}

function capartFixture() {
  return twoDa([
    "2DA V2.0",
    "",
    "NAME MDLNAME NODENAME",
    ...capartRows.map(([name, mdlName, nodeName], index) => (
      `${index} ${name} ${mdlName} ${nodeName}`
    )),
    "",
  ].join("\n"));
}

function genericPartsFixture() {
  return twoDa("2DA V2.0\n\nCOSTMODIFIER ACBONUS\n0 0 0.00\n1 0 0.10\n");
}

function robePartsFixture() {
  const columns = [
    "COSTMODIFIER", "ACBONUS",
    "HIDEFOOTR", "HIDEFOOTL", "HIDESHINR", "HIDESHINL", "HIDELEGR", "HIDELEGL",
    "HIDEPELVIS", "HIDECHEST", "HIDEBELT", "HIDENECK", "HIDEFORER", "HIDEFOREL",
    "HIDEBICEPR", "HIDEBICEPL", "HIDESHOR", "HIDESHOL", "HIDEHANDR", "HIDEHANDL",
    "HIDEHEAD",
  ];
  return twoDa([
    "2DA V2.0",
    "",
    columns.join(" "),
    `0 0 0.00 ${columns.slice(2).map(() => 0).join(" ")}`,
    `1 0 0.10 ${columns.slice(2).map((_, index) => index === 0 ? 1 : 0).join(" ")}`,
    "",
  ].join("\n"));
}

afterEach(() => {
  while (clients.length) clients.pop()?.dispose();
});

describe("Item Worker/WASM integration", () => {
  it("builds ModelType 2 as three independent MDLs, icon layers and a numeric UTI", async () => {
    const baseitemsTwoDa = await fetchBytes(baseitemsUrl);
    const client = new StudioWorkerClient();
    clients.push(client);

    const inspected = await client.request({
      requestId: "item-baseitems-inspect",
      type: "INSPECT_ITEM_BASEITEMS",
      baseitemsTwoDa: baseitemsTwoDa.slice(0),
    });
    expect(inspected).toMatchObject({ ok: true, type: "ITEM_BASEITEMS_INSPECTED" });
    if (!inspected.ok || inspected.type !== "ITEM_BASEITEMS_INSPECTED") {
      throw new Error("real Worker did not resolve baseitems.2da");
    }
    expect(JSON.parse(inspected.catalogJson)).toMatchObject({
      rows: [{
        baseItem: 4,
        modelType: 2,
        partSlots: [
          { field: "ModelPart1", token: "b" },
          { field: "ModelPart2", token: "m" },
          { field: "ModelPart3", token: "t" },
        ],
      }],
    });

    const fields = ["ModelPart1", "ModelPart2", "ModelPart3"];
    const modelResrefs = ["sw_b_007", "sw_m_007", "sw_t_007"];
    const iconResrefs = ["isw_b_007", "isw_m_007", "isw_t_007"];
    const sources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    const itemRequest = {
      requestId: "item-build",
      type: "BUILD_ITEM_PACKAGE",
      baseitemsTwoDa,
      baseItem: 4,
      hakResref: "m2aitemhak4",
      hakFileName: "m2aitemhak4.hak",
      moduleResref: "m2aitemmod4",
      moduleFileName: "m2aitemmod4.mod",
      moduleName: "Meshy2Aurora Item candidate",
      areaResref: "m2aitemarea4",
      areaName: "Meshy2Aurora Item Assembly Proof",
      blueprintResref: "m2aitemuti4",
      occupiedResourceKeys: [],
      seamValidation: {
        tolerance: 10,
      },
      referenceTables: [],
      referenceResources: [],
      referenceResourceManifest: null,
      capartContext: null,
      equippedProofContext: null,
      blueprintJson: JSON.stringify({
        schemaVersion: 1,
        templateResref: "m2aitemuti4",
        tag: "M2AITEMUTI4",
        localizedName: "Three Part Test Item",
        description: "Worker integration item.",
        identifiedDescription: "Worker integration item.",
        comment: "Generated by the Item integration test.",
        parts: fields.map((field) => ({ field, value: 7 })),
        colors: {
          leather1Color: null,
          leather2Color: null,
          cloth1Color: null,
          cloth2Color: null,
          metal1Color: null,
          metal2Color: null,
        },
        cost: 25,
        addCost: 5,
        charges: 3,
        stackSize: 1,
        paletteId: 2,
        identified: true,
        stolen: false,
        cursed: false,
        plot: false,
      }),
      parts: fields.map((field, index) => ({
        field,
        variant: 7,
        sourceKind: "MESHY_GLB",
        modelResref: modelResrefs[index],
        iconResref: iconResrefs[index],
        textureResref: `m2ait4${index}`,
        sourceGlb: sources[index],
        sourceNode: null,
        textureEncoding: "DIRECT_COLOR",
        transformJson: JSON.stringify({
          translation: [index * 2, 0, 0],
          rotationXyzw: [0, 0, 0, 1],
          uniformScale: 1,
          pivot: [0, 0, 0],
        }),
      })),
    } satisfies Parameters<StudioWorkerClient["request"]>[0];
    const response = await client.request(itemRequest, [baseitemsTwoDa, ...sources]);

    expect(baseitemsTwoDa.byteLength).toBe(0);
    expect(sources.every((source) => source.byteLength === 0)).toBe(true);
    expect(response).toMatchObject({ ok: true, type: "ITEM_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "ITEM_PACKAGE_BUILT") {
      throw new Error("real Worker did not return an item package");
    }
    expect(JSON.parse(response.reportJson)).toMatchObject({
      status: "OFFLINE_ITEM_PACKAGE_PASSED",
      profile: "ITEM",
      baseItem: 4,
      partCount: 3,
      iconLayerCount: 3,
      iconLayerMode: "GEOMETRY_RASTER_TGA_V2",
      iconRuntimeParity: "offline_semantic_readback_only",
      seamValidation: {
        status: "PASSED",
        results: [
          {
            status: "TOUCHING",
            requiredRelation: "ADJACENT_TOUCH",
            algorithm: "TRIANGLE_SURFACE_BVH_CONTAINMENT_V1",
            measurementSha256: expect.any(String),
          },
          {
            status: "TOUCHING",
            requiredRelation: "NON_ADJACENT_NO_OVERLAP",
            algorithm: "TRIANGLE_SURFACE_BVH_CONTAINMENT_V1",
            measurementSha256: expect.any(String),
          },
          {
            status: "TOUCHING",
            requiredRelation: "ADJACENT_TOUCH",
            algorithm: "TRIANGLE_SURFACE_BVH_CONTAINMENT_V1",
            measurementSha256: expect.any(String),
          },
        ],
      },
      triangleBudget: { triangleBudget: 300000, warning: false },
      modelVisibility: "not_tested",
      proofCompleteness: "missing",
      readyForOwnerProof: false,
    });
    expect(JSON.parse(response.utiReportJson)).toMatchObject({
      baseItem: 4,
      modelType: 2,
      partCount: 3,
      colorFieldCount: 0,
      semanticReadbackStatus: "PASS",
    });
    expect(JSON.parse(response.partReadbacksJson).map(
      (part: { modelResref: string }) => part.modelResref,
    )).toEqual(modelResrefs);
    expect(response.artifacts.filter(({ kind }) => kind === "MODEL").map(
      ({ fileName }) => fileName,
    )).toEqual(modelResrefs.map((resref) => `${resref}.mdl`));
    expect(response.artifacts.filter(({ kind }) => kind === "TEXTURE")).toHaveLength(6);
    const iconLayers = response.artifacts.filter(
      ({ artifactId }) => artifactId.startsWith("item-part-") && artifactId.endsWith("-icon"),
    );
    expect(iconLayers).toHaveLength(3);
    for (const layer of iconLayers) {
      const bytes = new Uint8Array(layer.bytes);
      expect(bytes[2]).toBe(2);
      expect(bytes[16]).toBe(32);
      const width = new DataView(layer.bytes).getUint16(12, true);
      const height = new DataView(layer.bytes).getUint16(14, true);
      const alpha = Array.from(
        { length: width * height },
        (_, index) => bytes[18 + index * 4 + 3],
      );
      expect(alpha.some((value) => value === 0)).toBe(true);
      expect(alpha.some((value) => value > 0)).toBe(true);
    }
    expect(response.artifacts).toEqual(expect.arrayContaining([
      expect.objectContaining({ kind: "HAK", fileName: "m2aitemhak4.hak" }),
      expect.objectContaining({ kind: "MODULE", fileName: "m2aitemmod4.mod" }),
      expect.objectContaining({ kind: "ITEM_BLUEPRINT", fileName: "m2aitemuti4.uti" }),
    ]));
    const hak = response.artifacts.find(({ kind }) => kind === "HAK");
    const module = response.artifacts.find(({ kind }) => kind === "MODULE");
    const uti = response.artifacts.find(({ kind }) => kind === "ITEM_BLUEPRINT");
    const report = response.artifacts.find(({ fileName }) => fileName === "item-build-report.json");
    expect(new TextDecoder().decode(hak!.bytes.slice(0, 8))).toBe("HAK V1.0");
    expect(new TextDecoder().decode(module!.bytes.slice(0, 8))).toBe("MOD V1.0");
    expect(new TextDecoder().decode(uti!.bytes.slice(0, 8))).toBe("UTI V3.2");
    expect(hak!.sha256).toBe(await sha256(hak!.bytes));
    expect(uti!.sha256).toBe(await sha256(uti!.bytes));
    expect(new TextDecoder().decode(report!.bytes)).toBe(response.reportJson);

    const collisionBaseitems = await fetchBytes(baseitemsUrl);
    const collisionSources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    await expect(client.request({
      ...itemRequest,
      requestId: "item-occupied-collision",
      baseitemsTwoDa: collisionBaseitems,
      occupiedResourceKeys: ["2025:m2aitemuti4"],
      parts: itemRequest.parts.map((part, index) => ({
        ...part,
        sourceGlb: collisionSources[index],
      })),
    }, [collisionBaseitems, ...collisionSources])).rejects.toThrow(
      "ITEM-RESOURCE-COLLISION",
    );

    const occupiedAreaBaseitems = await fetchBytes(baseitemsUrl);
    const occupiedAreaSources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    await expect(client.request({
      ...itemRequest,
      requestId: "item-occupied-area-collision",
      baseitemsTwoDa: occupiedAreaBaseitems,
      occupiedResourceKeys: ["2012:m2aitemarea4"],
      parts: itemRequest.parts.map((part, index) => ({
        ...part,
        sourceGlb: occupiedAreaSources[index],
      })),
    }, [occupiedAreaBaseitems, ...occupiedAreaSources])).rejects.toThrow(
      "ITEM-RESOURCE-COLLISION",
    );

    const overlapBaseitems = await fetchBytes(baseitemsUrl);
    const overlapSources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    await expect(client.request({
      ...itemRequest,
      requestId: "item-non-adjacent-overlap",
      baseitemsTwoDa: overlapBaseitems,
      parts: itemRequest.parts.map((part, index) => ({
        ...part,
        sourceGlb: overlapSources[index],
        transformJson: JSON.stringify({
          translation: [index === 1 ? 2 : 0, 0, 0],
          rotationXyzw: [0, 0, 0, 1],
          uniformScale: 1,
          pivot: [0, 0, 0],
        }),
      })),
    }, [overlapBaseitems, ...overlapSources])).rejects.toThrow(
      "ITEM-SEAM-VALIDATION-FAILED",
    );
  }, 60_000);

  it("builds CAPART armor and cloak as retail reference composers without fake GLBs", async () => {
    const client = new StudioWorkerClient();
    clients.push(client);
    const armorFields = [
      "ArmorPart_RFoot", "ArmorPart_LFoot", "ArmorPart_RShin", "ArmorPart_LShin",
      "ArmorPart_LThigh", "ArmorPart_RThigh", "ArmorPart_Pelvis", "ArmorPart_Torso",
      "ArmorPart_Belt", "ArmorPart_Neck", "ArmorPart_RFArm", "ArmorPart_LFArm",
      "ArmorPart_RBicep", "ArmorPart_LBicep", "ArmorPart_RShoul", "ArmorPart_LShoul",
      "ArmorPart_RHand", "ArmorPart_LHand", "ArmorPart_Robe",
    ];
    const baseitems = new TextEncoder().encode([
      "2DA V2.0",
      "",
      "Label ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight",
      "16 Armor armor 3 0 it_bag iit_chest 1 2 3",
      "80 Cloak cloak 1 1 it_bag icloak 1 2 2",
      "",
    ].join("\n")).buffer;
    const capart = capartFixture();
    const partTables = [
      ...capartPartTables.map((tableName) => ({
        tableName,
        fileName: `${tableName.toLowerCase()}.2da`,
        bytes: genericPartsFixture(),
      })),
      { tableName: "PARTS_ROBE", fileName: "parts_robe.2da", bytes: robePartsFixture() },
    ];
    const referenceSource = asStaticItemPart(await fetchBytes(sourceUrl));
    const mdlTemplate = await referenceMdlTemplate(referenceSource);
    const capartResources = capartRows.flatMap(([, mdlName]) => {
      const resref = `pmh0_${mdlName.toLowerCase()}001`;
      return [
        {
          resourceType: 2002 as const,
          resref,
          fileName: `${resref}.mdl`,
          bytes: renameBinaryMdl(mdlTemplate, resref),
        },
        {
          resourceType: 6 as const,
          resref,
          fileName: `${resref}.plt`,
          bytes: onePixelRetailPlt(),
        },
      ];
    });
    const appearance = appearanceFixture();
    const capartManifest = await retailManifest(capartResources);
    const colors = {
      leather1Color: 0,
      leather2Color: 0,
      cloth1Color: 0,
      cloth2Color: 0,
      metal1Color: 0,
      metal2Color: 0,
    };
    const commonPart = {
      modelResref: "",
      iconResref: "",
      textureResref: "",
      sourceNode: null,
      textureEncoding: "PLT_LEATHER1" as const,
      transformJson: JSON.stringify({
        translation: [0, 0, 0],
        rotationXyzw: [0, 0, 0, 1],
        uniformScale: 1,
        pivot: [0, 0, 0],
      }),
    };
    const armorResponse = await client.request({
      requestId: "item-armor-build",
      type: "BUILD_ITEM_PACKAGE",
      baseitemsTwoDa: baseitems.slice(0),
      baseItem: 16,
      hakResref: "m2aarmorhak",
      hakFileName: "m2aarmorhak.hak",
      moduleResref: "m2aarmormod",
      moduleFileName: "m2aarmormod.mod",
      moduleName: "Meshy2Aurora CAPART candidate",
      areaResref: "m2aarmorarea",
      areaName: "CAPART Item Assembly Proof",
      blueprintResref: "m2aarmoruti",
      occupiedResourceKeys: [],
      seamValidation: { tolerance: 0.01 },
      blueprintJson: JSON.stringify({
        schemaVersion: 1,
        templateResref: "m2aarmoruti",
        tag: "M2AARMORUTI",
        localizedName: "CAPART armor",
        description: "CAPART armor",
        identifiedDescription: "CAPART armor",
        comment: "Reference composer integration test.",
        parts: armorFields.map((field) => ({ field, value: 1 })),
        properties: [],
        colors,
        cost: 0,
        addCost: 0,
        charges: 0,
        stackSize: 1,
        paletteId: 0,
        identified: true,
        stolen: false,
        cursed: false,
        plot: false,
      }),
      referenceTables: [
        { tableName: "CAPART", fileName: "capart.2da", bytes: capart },
        ...partTables,
        { tableName: "APPEARANCE", fileName: "appearance.2da", bytes: appearance },
      ],
      referenceResources: capartResources,
      referenceResourceManifest: {
        fileName: "nwn-base-item-resources.json",
        bytes: capartManifest,
      },
      capartContext: {
        schemaVersion: 1,
        modelPrefix: "pmh0",
        genderCode: null,
      },
      equippedProofContext: {
        creatureResref: "m2aarmornpc",
        appearanceRow: 6,
        race: 6,
        gender: 0,
        phenotype: 0,
      },
      parts: armorFields.map((field) => ({
        field,
        variant: 1,
        sourceKind: "CAPART_SELECTION" as const,
        ...commonPart,
      })),
    }, [
      baseitems,
      capart,
      ...partTables.map((table) => table.bytes),
      appearance,
      capartManifest,
      ...capartResources.map((resource) => resource.bytes),
    ]);
    expect(armorResponse).toMatchObject({ ok: true, type: "ITEM_PACKAGE_BUILT" });
    if (!armorResponse.ok || armorResponse.type !== "ITEM_PACKAGE_BUILT") {
      throw new Error("CAPART reference composer build failed");
    }
    expect(JSON.parse(armorResponse.reportJson)).toMatchObject({
      partCount: 19,
      meshyPartCount: 0,
      referenceSelectorCount: 19,
      iconLayerCount: 0,
      iconLayerMode: "RETAIL_CAPART_COMPOSITION_PLAN_V1",
      triangleBudget: {
        triangleCount: 216,
        triangleBudget: 300000,
      },
      specialResolution: expect.arrayContaining([
        expect.objectContaining({
          field: "ArmorPart_RFoot",
          partsTable: "PARTS_FOOT",
          availablePartCount: 2,
        }),
        expect.objectContaining({
          field: "ArmorPart_Robe",
          partsTable: "PARTS_ROBE",
          robeHiddenMdlNames: ["FOOTR"],
        }),
      ]),
      proofModule: {
        fixtureProfile: "EQUIPPED_CAPART_ARMOR_V2",
        equipmentSlot: 2,
        creatureResref: "m2aarmornpc",
        appearanceRow: 6,
        race: 6,
        gender: 0,
        phenotype: 0,
        semanticReadbackStatus: "PASS",
      },
    });
    expect(armorResponse.artifacts.filter(({ kind }) => kind === "MODEL")).toHaveLength(0);
    expect(armorResponse.artifacts.filter(({ kind }) => kind === "TEXTURE")).toHaveLength(0);

    const cloakBaseitems = new TextEncoder().encode([
      "2DA V2.0",
      "",
      "Label ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight",
      "80 Cloak cloak 1 1 it_bag icloak 1 2 2",
      "",
    ].join("\n")).buffer;
    const cloakModel = new TextEncoder().encode(
      "2DA V2.0\n\nMODEL TEXTURE ICON\n0 0 0 0\n1 1 1 1\n",
    ).buffer;
    const cloakMdl = renameBinaryMdl(mdlTemplate, "pmh0_cloak_001");
    const cloakTexture = onePixelRetailPlt();
    const cloakIcon = onePixelRetailPlt();
    const cloakAppearance = appearanceFixture();
    const cloakResources = [
      {
        resourceType: 2002 as const,
        resref: "pmh0_cloak_001",
        fileName: "pmh0_cloak_001.mdl",
        bytes: cloakMdl,
      },
      {
        resourceType: 6 as const,
        resref: "cloak_001",
        fileName: "cloak_001.plt",
        bytes: cloakTexture,
      },
      {
        resourceType: 6 as const,
        resref: "icloak_m_001",
        fileName: "icloak_m_001.plt",
        bytes: cloakIcon,
      },
    ];
    const cloakManifest = await retailManifest(cloakResources);
    const cloakRequest = {
      requestId: "item-cloak-build",
      type: "BUILD_ITEM_PACKAGE",
      baseitemsTwoDa: cloakBaseitems,
      baseItem: 80,
      hakResref: "m2acloakhak",
      hakFileName: "m2acloakhak.hak",
      moduleResref: "m2acloakmod",
      moduleFileName: "m2acloakmod.mod",
      moduleName: "Meshy2Aurora cloak candidate",
      areaResref: "m2acloakarea",
      areaName: "Cloak Item Assembly Proof",
      blueprintResref: "m2acloakuti",
      occupiedResourceKeys: [],
      seamValidation: { tolerance: 0.01 },
      blueprintJson: JSON.stringify({
        schemaVersion: 1,
        templateResref: "m2acloakuti",
        tag: "M2ACLOAKUTI",
        localizedName: "Cloak",
        description: "Cloak",
        identifiedDescription: "Cloak",
        comment: "Reference composer integration test.",
        parts: [{ field: "ModelPart1", value: 1 }],
        properties: [],
        colors,
        cost: 0,
        addCost: 0,
        charges: 0,
        stackSize: 1,
        paletteId: 0,
        identified: true,
        stolen: false,
        cursed: false,
        plot: false,
      }),
      referenceTables: [
        {
          tableName: "CLOAKMODEL",
          fileName: "cloakmodel.2da",
          bytes: cloakModel,
        },
        {
          tableName: "APPEARANCE",
          fileName: "appearance.2da",
          bytes: cloakAppearance,
        },
      ],
      referenceResources: cloakResources,
      referenceResourceManifest: {
        fileName: "nwn-base-cloak-resources.json",
        bytes: cloakManifest,
      },
      capartContext: null,
      equippedProofContext: {
        creatureResref: "m2acloaknpc",
        appearanceRow: 6,
        race: 6,
        gender: 0,
        phenotype: 0,
      },
      parts: [{
        field: "ModelPart1",
        variant: 1,
        sourceKind: "CLOAK_MODEL_SELECTION",
        ...commonPart,
      }],
    } satisfies Parameters<StudioWorkerClient["request"]>[0];
    await expect(client.request({
      ...cloakRequest,
      requestId: "item-cloak-appearance-mismatch",
      baseitemsTwoDa: cloakBaseitems.slice(0),
      referenceTables: cloakRequest.referenceTables.map((table) => ({
        ...table,
        bytes: table.bytes.slice(0),
      })),
      referenceResources: cloakResources.map((resource) => ({
        ...resource,
        bytes: resource.bytes.slice(0),
      })),
      referenceResourceManifest: {
        ...cloakRequest.referenceResourceManifest,
        bytes: cloakManifest.slice(0),
      },
      equippedProofContext: {
        ...cloakRequest.equippedProofContext!,
        race: 5,
      },
    })).rejects.toThrow("ITEM-EQUIPPED-APPEARANCE-RACIALTYPE-MISMATCH");
    const tamperedManifestDocument = JSON.parse(
      new TextDecoder().decode(cloakManifest),
    ) as {
      resources: Array<{ sha256: string }>;
    };
    tamperedManifestDocument.resources[0].sha256 = "0".repeat(64);
    const tamperedManifest = new TextEncoder().encode(
      JSON.stringify(tamperedManifestDocument),
    ).buffer;
    await expect(client.request({
      ...cloakRequest,
      requestId: "item-cloak-payload-hash-mismatch",
      baseitemsTwoDa: cloakBaseitems.slice(0),
      referenceTables: cloakRequest.referenceTables.map((table) => ({
        ...table,
        bytes: table.bytes.slice(0),
      })),
      referenceResources: cloakResources.map((resource) => ({
        ...resource,
        bytes: resource.bytes.slice(0),
      })),
      referenceResourceManifest: {
        ...cloakRequest.referenceResourceManifest,
        bytes: tamperedManifest,
      },
    })).rejects.toThrow("ITEM-RESOURCE-PAYLOAD-HASH-MISMATCH");
    const cloakResponse = await client.request(cloakRequest, [
      cloakBaseitems,
      cloakModel,
      cloakAppearance,
      cloakManifest,
      cloakMdl,
      cloakTexture,
      cloakIcon,
    ]);
    expect(cloakResponse).toMatchObject({ ok: true, type: "ITEM_PACKAGE_BUILT" });
    if (!cloakResponse.ok || cloakResponse.type !== "ITEM_PACKAGE_BUILT") {
      throw new Error("cloak reference composer build failed");
    }
    expect(JSON.parse(cloakResponse.reportJson)).toMatchObject({
      meshyPartCount: 0,
      referenceSelectorCount: 1,
      iconLayerMode: "RETAIL_CLOAKMODEL_ICON",
      triangleBudget: {
        triangleCount: 12,
        triangleBudget: 300000,
      },
      specialResolution: {
        modelResref: "pmh0_cloak_001",
        textureResref: "cloak_001",
        iconResref: "icloak_m_001",
        resourceVerification: "PINNED_MANIFEST_PAYLOAD_VALIDATED",
      },
      proofModule: {
        fixtureProfile: "EQUIPPED_CLOAK_V2",
        equipmentSlot: 8192,
        creatureResref: "m2acloaknpc",
        appearanceRow: 6,
        race: 6,
        gender: 0,
        phenotype: 0,
        semanticReadbackStatus: "PASS",
      },
    });
  }, 60_000);
});
