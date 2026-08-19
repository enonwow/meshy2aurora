import { afterEach, describe, expect, it } from "vitest";
import sourceUrl from "../.generated/owned-package/generated/source.glb?url";
import baseitemsUrl from "../fixtures/baseitems.2da?url";
import { StudioWorkerClient } from "../../src/worker/client";
import initWasm, {
  appendItemCustomWeaponBaseitemV2,
  appendItemCustomWeaponBaseitemV2ReportJson,
  buildMeshyItemPartWithOptionsV2,
  inspectItemBaseitemsV1Json,
} from "@m2a-wasm";

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
  root.nodes = [{
    name: "item-part-source-root",
    mesh: 0,
    // Weapon-shaped integration witness: a long axial source with a broad
    // face and materially thinner depth. The old humanoid-width fixture could
    // not exercise a LONG_VERTICAL_PART_ORDER_V2 inventory layout honestly.
    scale: [0.25, 1, 0.25],
  }];
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

function baseitemsFixtureWithRowCount(rowCount: number) {
  const rows = Array.from({ length: rowCount }, (_, index) => index === 6
    ? "6 heavycrossbow 173 WBwXh 2 0 it_bag iwbwxh 0x00030 2 4 10 100 6 1 25"
    : `${index} filler_${index} **** Ring 0 0 it_bag iring 8 1 1 **** **** **** **** ****`);
  return twoDa([
    "2DA V2.0",
    "",
    "Label Name ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight MinRange MaxRange WeaponWield WeaponType RangedWeapon",
    ...rows,
    "",
  ].join("\n"));
}

function rangedAmmunitionBaseitemsFixture() {
  return twoDa([
    "2DA V2.0",
    "",
    "Label Name ItemClass ModelType GenderSpecific DefaultModel DefaultIcon EquipableSlots InvSlotWidth InvSlotHeight MinRange MaxRange WeaponWield WeaponType RangedWeapon AmmunitionType",
    "1 testgun **** M2aGun 0 0 it_bag im2agun 0x00030 2 2 10 100 6 1 27 3",
    "20 arrow **** WAmAr 0 0 it_bag iwamar 0 1 1 **** **** **** **** **** ****",
    "25 bolt **** WAmBo 0 0 it_bag iwambo 0 1 1 **** **** **** **** **** ****",
    "27 bullet **** WAmBu 0 0 it_bag iwambu 0 1 1 **** **** **** **** **** ****",
    "",
  ].join("\n"));
}

function rangedAmmunitiontypesFixture() {
  const kinds = [
    ["arrow", "wamar_001", "cb_ht_arrow1"],
    ["bolt", "wambo_001", "cb_ht_arrow1"],
    ["bullet", "wambu_001", "cb_ht_bullet1"],
    ["dart", "wthdt_001", "cb_ht_dart1"],
    ["shuriken", "wthsh_001", "cb_ht_dart1"],
    ["throwingaxe", "wthax_001", "cb_ht_throwaxe1"],
  ] as const;
  return twoDa([
    "2DA V2.0",
    "",
    "label Model ShotSound ImpactSound AmmunitionType DamageRangedProjectile",
    ...Array.from({ length: 6 }, (_, damage) => kinds.map((kind, offset) => (
      `${damage * 6 + offset} ${kind[0]}_${damage} ${kind[1]} **** ${kind[2]} ${offset + 1} ${damage}`
    ))).flat(),
    "",
  ].join("\n"));
}

function rangedDamageTypesFixture() {
  return twoDa([
    "2DA V2.0",
    "",
    "Label CharsheetStrref DamageTypeGroup DamageRangedProjectile",
    "0 Bludgeoning 58345 0 0",
    "6 Divine 58305 4 0",
    "",
  ].join("\n"));
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
  it("keeps legacy donor cloning available for unrelated output 114 across the public WASM boundary", async () => {
    wasmReady ??= initWasm();
    await wasmReady;
    const source = baseitemsFixtureWithRowCount(114);
    const requestJson = JSON.stringify({
      schemaVersion: 2,
      donorBaseItem: 6,
      outputBaseItem: 114,
      label: "legacy_arc_cannon",
      itemClass: "WArc",
      nameStrref: null,
      invSlotWidth: 2,
      invSlotHeight: 4,
    });

    const report = JSON.parse(appendItemCustomWeaponBaseitemV2ReportJson(
      new Uint8Array(source),
      requestJson,
    ));
    const output = appendItemCustomWeaponBaseitemV2(new Uint8Array(source), requestJson);
    const catalog = JSON.parse(inspectItemBaseitemsV1Json(output));

    expect(report).toMatchObject({
      status: "APPENDED_EXACT",
      donorBaseItem: 6,
      outputBaseItem: 114,
      runtimeRoute: { baseItem: 6, runtimeClip: "xbowshot" },
    });
    expect(catalog.rows.find((row: { baseItem: number }) => row.baseItem === 114)).toMatchObject({
      label: "legacy_arc_cannon",
      itemClass: "WArc",
      weaponWield: 6,
      weaponType: 1,
      rangedWeapon: 25,
    });
    expect(catalog.rows.find((row: { baseItem: number }) => row.baseItem === 6)).toMatchObject({
      label: "heavycrossbow",
      itemClass: "WBwXh",
    });
  });

  it("rejects the reserved standalone 113 identity through the legacy donor-clone WASM boundary", async () => {
    wasmReady ??= initWasm();
    await wasmReady;
    const source = baseitemsFixtureWithRowCount(113);
    const requestJson = JSON.stringify({
      schemaVersion: 2,
      donorBaseItem: 6,
      outputBaseItem: 113,
      label: "hextech_shotgun",
      itemClass: "WHxSh",
      nameStrref: null,
      invSlotWidth: 2,
      invSlotHeight: 4,
    });

    let reservedError: unknown;
    try {
      appendItemCustomWeaponBaseitemV2(new Uint8Array(source), requestJson);
    } catch (error) {
      reservedError = error;
    }

    expect(JSON.parse(String(reservedError))).toMatchObject({
      code: "ITEM-CUSTOM-BASEITEM-STANDALONE-RESERVED",
    });
  });

  it("packages a custom Bullet stack, explicit EE damage route and +Y projectile in one worker transaction", async () => {
    const client = new StudioWorkerClient();
    clients.push(client);
    const baseitemsTwoDa = rangedAmmunitionBaseitemsFixture();
    const weaponSourceGlb = asStaticItemPart(await fetchBytes(sourceUrl));
    const projectileSourceGlb = weaponSourceGlb.slice(0);
    const ammunitiontypesTwoDa = rangedAmmunitiontypesFixture();
    const damageTypesTwoDa = rangedDamageTypesFixture();
    const colors = {
      leather1Color: null,
      leather2Color: null,
      cloth1Color: null,
      cloth2Color: null,
      metal1Color: null,
      metal2Color: null,
    };
    const transform = {
      translation: [0, 0, 0],
      rotationXyzw: [0, 0, 0, 1],
      uniformScale: 0.2,
      pivot: [0, 0, 0],
    };
    const ammoProperty = {
      propertyName: 16,
      subtype: 8,
      costTable: 4,
      costValue: 1,
      param1: 255,
      param1Value: 0,
      chanceAppear: 100,
    };
    const response = await client.request({
      requestId: "item-ranged-ammunition-build",
      type: "BUILD_ITEM_PACKAGE",
      baseitemsTwoDa,
      baseItem: 1,
      rangedAmmunition: {
        ammunitiontypesTwoDa,
        damageTypesTwoDa,
        ammunitionChannel: "BULLET",
        damageRangedProjectile: 6,
        damageTypeRow: 6,
        damageTypeLabel: "Divine",
        damagePropertySubtype: 8,
        wielderClip: "XBOWSHOT",
        projectileSourceGlb,
        projectileModelResref: "m2ahxshell",
        projectileTextureResref: "m2ahxshtex",
        projectileOptionsJson: JSON.stringify({
          schemaVersion: 1,
          sourceForwardAxis: "POSITIVE_Y",
          transform,
          sourceNode: null,
        }),
        shotSoundResref: "cb_sh_prjtlelec",
        impactSoundResref: "scm_elec",
        ammunitionBlueprintResref: "m2ahxammo",
        ammunitionBlueprintJson: JSON.stringify({
          schemaVersion: 1,
          templateResref: "m2ahxammo",
          tag: "M2AHXAMMO",
          localizedName: "Hextech Shells",
          description: "Worker integration ammunition.",
          identifiedDescription: "Worker integration ammunition.",
          comment: "Ranged ammunition integration test.",
          parts: [{ field: "ModelPart1", value: 99 }],
          properties: [ammoProperty],
          colors,
          cost: 50,
          addCost: 0,
          charges: 0,
          stackSize: 99,
          paletteId: 4,
          identified: true,
          stolen: false,
          cursed: false,
          plot: false,
        }),
        ammunitionModelResref: "wambu_099",
        ammunitionTextureResref: "m2ahxamtex",
        ammunitionIconResref: "iwambu_099",
        ammunitionVariant: 99,
        ammunitionOptionsJson: JSON.stringify({
          schemaVersion: 1,
          transform,
          sourceNode: null,
          textureEncoding: "DIRECT_COLOR",
          iconSize: [32, 32],
          iconProjectionBounds: null,
          weaponColor: null,
          targetSpaceScaleXyz: [1, 1, 1],
        }),
      },
      hakResref: "m2arnghak",
      hakFileName: "m2arnghak.hak",
      moduleResref: "m2arngmod",
      moduleFileName: "m2arngmod.mod",
      moduleName: "Meshy2Aurora ranged ammunition candidate",
      areaResref: "m2arngarea",
      areaName: "Meshy2Aurora Ranged Ammunition Proof",
      blueprintResref: "m2arngweapon",
      blueprintJson: JSON.stringify({
        schemaVersion: 1,
        templateResref: "m2arngweapon",
        tag: "M2ARNGWEAPON",
        localizedName: "Test gun",
        description: "Worker integration ranged weapon.",
        identifiedDescription: "Worker integration ranged weapon.",
        comment: "Ranged ammunition integration test.",
        parts: [{ field: "ModelPart1", value: 1 }],
        properties: [],
        colors,
        cost: 100,
        addCost: 0,
        charges: 0,
        stackSize: 1,
        paletteId: 2,
        identified: true,
        stolen: false,
        cursed: false,
        plot: false,
      }),
      generationSessionJson: null,
      generationArtifactsJson: null,
      fitReportJson: null,
      attachmentProfileJson: null,
      occupiedResourceKeys: [],
      seamValidation: { tolerance: 0.01 },
      referenceTables: [],
      referenceResources: [],
      referenceResourceManifest: null,
      capartContext: null,
      equippedProofContext: null,
      parts: [{
        field: "ModelPart1",
        variant: 1,
        sourceKind: "MESHY_GLB",
        modelResref: "m2agun_001",
        iconResref: "im2agun_001",
        textureResref: "m2aguntx",
        sourceGlb: weaponSourceGlb,
        transformJson: JSON.stringify({
          ...transform,
          uniformScale: 1,
        }),
        targetSpaceScaleXyz: [1, 1, 1],
        sourceNode: null,
        textureEncoding: "DIRECT_COLOR",
      }],
    }, [
      baseitemsTwoDa,
      ammunitiontypesTwoDa,
      damageTypesTwoDa,
      projectileSourceGlb,
      weaponSourceGlb,
    ]);
    expect(response).toMatchObject({ ok: true, type: "ITEM_PACKAGE_BUILT" });
    if (!response.ok || response.type !== "ITEM_PACKAGE_BUILT") {
      throw new Error("ranged ammunition worker transaction failed");
    }
    expect(JSON.parse(response.reportJson)).toMatchObject({
      status: "OFFLINE_ITEM_PACKAGE_PASSED",
      rangedAmmunition: {
        status: "OFFLINE_RANGED_AMMUNITION_PASSED",
        binding: {
          weaponBaseItem: 1,
          ammoBaseItem: 27,
          ammunitionType: 3,
          damageRangedProjectile: 6,
          ammunitiontypesRow: 38,
          projectileModelResref: "m2ahxshell",
          runtimeClip: "xbowshot",
        },
        damageRoute: {
          status: "PATCHED_DAMAGE_RANGED_PROJECTILE",
          damageTypeRow: 6,
          expectedLabel: "Divine",
          semanticReadbackStatus: "PASS",
        },
        ammunitiontypes: {
          firstRow: 36,
          lastRow: 41,
          semanticReadbackStatus: "PASS",
        },
        ammunitionItem: {
          baseItem: 27,
          modelResref: "wambu_099",
          blueprintResref: "m2ahxammo",
        },
        runtimeValidation: {
          status: "OWNER_PROOF_REQUIRED",
          modelVisibility: "not_tested",
          proofCompleteness: "missing",
        },
      },
      proofModule: {
        fixtureProfile: "RANGED_WEAPON_AMMUNITION_AND_IMMOBILE_TARGET_V1",
        weaponBlueprintResref: "m2arngweapon",
        ammunitionBlueprintResref: "m2ahxammo",
        groundItemCount: 2,
        creatureCount: 1,
        targetBlueprintResref: "m2arngtarget",
        targetAppearanceRow: 102,
        targetPosition: [10, 18, 0],
        targetWalkRate: 0,
        targetScriptsEmpty: true,
        semanticReadbackStatus: "PASS",
      },
      triangleBudget: { triangleBudget: 300000, warning: false },
    });
    expect(response.artifacts.filter(({ kind }) => kind === "TWO_DA").map(({ fileName }) => fileName)).toEqual([
      "ammunitiontypes.2da",
      "damagetypes.2da",
    ]);
    expect(response.artifacts.filter(({ kind }) => kind === "ITEM_BLUEPRINT")).toHaveLength(2);
    expect(response.artifacts.filter(({ kind }) => kind === "MODEL")).toHaveLength(3);
  }, 60_000);

  it("builds ModelType 2 as three independent +Y MDLs in an item-only proof MOD", async () => {
    const baseitemsTwoDa = await fetchBytes(baseitemsUrl);
    const appearance = appearanceFixture();
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
    const modelResrefs = ["sw_b_251", "sw_m_251", "sw_t_251"];
    const iconResrefs = ["isw_b_251", "isw_m_251", "isw_t_251"];
    const sources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    const fitSources = sources.map((source) => source.slice(0));
    const fitted = await client.request({
      requestId: "item-fit",
      type: "FIT_ITEM_PARTS",
      tolerance: 10,
      parts: fields.map((field, index) => ({
        field,
        modelResref: modelResrefs[index],
        sourceGlb: fitSources[index],
        sourceNode: null,
      })),
    }, fitSources);
    expect(fitted).toMatchObject({ ok: true, type: "ITEM_PARTS_FITTED" });
    if (!fitted.ok || fitted.type !== "ITEM_PARTS_FITTED") {
      throw new Error("real Worker did not return an Item fit report");
    }
    let fitReport = JSON.parse(fitted.fitReportJson) as {
      schemaVersion: number;
      algorithm: string;
      status: string;
      solutionSha256: string;
      referenceProfileSha256?: string;
      parts: Array<{
        field: string;
        axialTargetAxis: number;
        transform: unknown;
        targetSpaceScaleXyz: [number, number, number];
        transformSha256: string;
      }>;
      adjacentConnectors: Array<{
        firstField: string;
        secondField: string;
        axialOverlap: number;
        status: string;
      }>;
      orientationFrame: {
        targetAxialAxis: number;
        targetWidthAxis: number;
        targetDepthAxis: number;
        widthToDepthRatio: number;
        handednessDeterminant: number;
        status: string;
      };
    };
    expect(fitReport).toMatchObject({
      schemaVersion: 3,
      algorithm: "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX",
      status: "PASSED",
      solutionSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
      parts: fields.map((field) => ({
        field,
        axialTargetAxis: 1,
        transformSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
      })),
      adjacentConnectors: [
        {
          firstField: "ModelPart1",
          secondField: "ModelPart2",
          axialOverlap: expect.any(Number),
          status: "OVERLAPPING",
        },
        {
          firstField: "ModelPart2",
          secondField: "ModelPart3",
          axialOverlap: expect.any(Number),
          status: "OVERLAPPING",
        },
      ],
      orientationFrame: {
        targetAxialAxis: 1,
        targetWidthAxis: 2,
        targetDepthAxis: 0,
        widthToDepthRatio: expect.any(Number),
        handednessDeterminant: 1,
        status: "PASSED",
      },
    });
    wasmReady ??= initWasm();
    await wasmReady;
    const referenceMdls = sources.map((source, index) => {
      const result = buildMeshyItemPartWithOptionsV2(
        new Uint8Array(source),
        modelResrefs[index],
        `reftex${index}`,
        JSON.stringify({
          schemaVersion: 1,
          transform: fitReport.parts[index].transform,
          sourceNode: null,
          textureEncoding: "DIRECT_COLOR",
          iconSize: null,
          iconProjectionBounds: null,
        }),
      );
      try {
        const mdl = result.takeMdlBytes().slice().buffer;
        result.takeTextureBytes();
        result.takeIconBytes();
        return mdl;
      } finally {
        result.free();
      }
    });
    const profileBaseitems = baseitemsTwoDa.slice(0);
    const profileResponse = await client.request({
      requestId: "item-attachment-profile",
      type: "BUILD_ITEM_ATTACHMENT_PROFILE",
      baseitemsTwoDa: profileBaseitems,
      baseItem: 4,
      referenceKind: "EXPLICIT_VARIANTS",
      referenceId: "worker-integration-reference",
      models: fields.map((field, index) => ({
        field,
        modelResref: modelResrefs[index],
        bytes: referenceMdls[index],
      })),
    }, [profileBaseitems, ...referenceMdls]);
    expect(profileResponse).toMatchObject({
      ok: true,
      type: "ITEM_ATTACHMENT_PROFILE_BUILT",
    });
    if (!profileResponse.ok || profileResponse.type !== "ITEM_ATTACHMENT_PROFILE_BUILT") {
      throw new Error("real Worker did not return an Item attachment profile");
    }
    const attachmentProfile = JSON.parse(profileResponse.attachmentProfileJson) as {
      profileSha256: string;
    };
    const profileFitSources = sources.map((source) => source.slice(0));
    const profileFitted = await client.request({
      requestId: "item-reference-fit",
      type: "FIT_ITEM_PARTS",
      tolerance: 10,
      attachmentProfileJson: profileResponse.attachmentProfileJson,
      parts: fields.map((field, index) => ({
        field,
        modelResref: modelResrefs[index],
        sourceGlb: profileFitSources[index],
        sourceNode: null,
      })),
    }, profileFitSources);
    expect(profileFitted).toMatchObject({ ok: true, type: "ITEM_PARTS_FITTED" });
    if (!profileFitted.ok || profileFitted.type !== "ITEM_PARTS_FITTED") {
      throw new Error("real Worker did not return a profile-bound Item fit report");
    }
    fitReport = JSON.parse(profileFitted.fitReportJson) as typeof fitReport;
    expect(fitReport).toMatchObject({
      schemaVersion: 4,
      algorithm: "ITEM_REFERENCE_SLOT_FRAME_FIT_V1",
      status: "PASSED",
      referenceProfileSha256: attachmentProfile.profileSha256,
      parts: fields.map((field) => ({
        field,
        sourceSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
        transformSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
      })),
    });
    const fittedParts = new Map(fitReport.parts.map((part) => [part.field, part]));
    const sourceHashes = await Promise.all(sources.map(sha256));
    const sourceLengths = sources.map((source) => source.byteLength);
    const generationSessionJson = JSON.stringify({
      schemaVersion: 1,
      sessionId: "item-worker-generation",
      createdAt: "2026-08-02T12:00:00.000Z",
      baseitemsSha256: await sha256(baseitemsTwoDa),
      baseItem: 4,
      itemClass: "sw",
      modelType: 2,
      status: "ARTIFACTS_VERIFIED",
      ownerCreditCap: 90,
      balanceAtReview: 120,
      maximumCredits: 90,
      slots: fields.map((field, index) => ({
        field,
        label: ["Bottom", "Middle", "Top"][index],
        token: ["b", "m", "t"][index],
        role: ["BOTTOM", "MIDDLE", "TOP"][index],
        profileId: "S1-static-prop/v1",
        targetPolycount: [5000, 8000, 3000][index],
        status: "ARTIFACT_VERIFIED",
        concept: {
          fileName: `${field}.png`,
          mimeType: "image/png",
          byteLength: 4,
          sha256: String(index + 1).repeat(64),
        },
        preview: null,
        run: {
          runId: `run-${index + 1}`,
          taskId: `task-${index + 1}`,
          createdAt: "2026-08-02T12:01:00.000Z",
        },
        artifact: {
          sha256: sourceHashes[index],
          byteLength: sources[index].byteLength,
          consumedCredits: 30,
          finishedAt: "2026-08-02T12:03:00.000Z",
        },
      })),
    });
    const generationArtifactsJson = JSON.stringify({
      schemaVersion: 1,
      artifacts: fields.map((field, index) => ({
        field,
        profileId: "S1-static-prop/v1",
        bridgeProtocolVersion: 1,
        sha256: sourceHashes[index],
        byteLength: sourceLengths[index],
        taskIds: { PREVIEW: `task-${index + 1}` },
        consumedCredits: 30,
        createdAt: "2026-08-02T12:01:00.000Z",
        finishedAt: "2026-08-02T12:03:00.000Z",
      })),
    });
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
      generationSessionJson,
      generationArtifactsJson,
      fitReportJson: profileFitted.fitReportJson,
      attachmentProfileJson: profileResponse.attachmentProfileJson,
      occupiedResourceKeys: [],
      seamValidation: {
        tolerance: 10,
      },
      referenceTables: [{
        tableName: "APPEARANCE",
        fileName: "appearance.2da",
        bytes: appearance,
      }],
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
        parts: fields.map((field) => ({ field, value: 251 })),
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
      parts: fields.map((field, index) => {
        const token = ["b", "m", "t"][index];
        const weaponColorways = ([1, 2, 3, 4] as const).map((color) => ({
          color,
          variant: 250 + color,
          modelResref: `sw_${token}_${250 + color}`,
          iconResref: `isw_${token}_${250 + color}`,
          textureResref: `m2ait4${index}c${color}`,
        }));
        return {
          field,
          variant: 251,
          sourceKind: "MESHY_GLB" as const,
          modelResref: modelResrefs[index],
          iconResref: iconResrefs[index],
          textureResref: `m2ait4${index}c1`,
          weaponColorways,
          sourceGlb: sources[index],
          sourceNode: null,
          textureEncoding: "DIRECT_COLOR" as const,
          transformJson: JSON.stringify(fittedParts.get(field)?.transform),
          targetSpaceScaleXyz: fittedParts.get(field)?.targetSpaceScaleXyz,
        };
      }),
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
      iconLayerCount: 12,
      weaponColorwayCoverage: {
        status: "COMPLETE",
        expectedResourceCount: 12,
        emittedResourceCount: 12,
        colors: [1, 2, 3, 4],
        geometryReuse: "ONE_MESHY_GLB_PER_PART",
      },
      baseitemsModelRange: {
        status: "PATCHED",
        sourceMinRange: 10,
        sourceMaxRange: 100,
        effectiveMaxRange: 250,
      },
      iconLayerMode: "AURORA_MODELTYPE2_ICON_LAYERS_V3",
      iconRuntimeParity: "offline_native_layer_composite_validated",
      seamValidation: {
        status: "PASSED",
        results: [
          {
            status: "OVERLAP",
            requiredRelation: "ADJACENT_CONNECTED",
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
            status: "OVERLAP",
            requiredRelation: "ADJACENT_CONNECTED",
            algorithm: "TRIANGLE_SURFACE_BVH_CONTAINMENT_V1",
            measurementSha256: expect.any(String),
          },
        ],
      },
      triangleBudget: { triangleBudget: 300000, warning: false },
      generation: {
        sessionId: "item-worker-generation",
        status: "ARTIFACTS_VERIFIED",
        maximumCredits: 90,
        actualConsumedCredits: 90,
        slots: fields.map((field, index) => ({
          field,
          role: ["BOTTOM", "MIDDLE", "TOP"][index],
          taskId: `task-${index + 1}`,
          glbSha256: sourceHashes[index],
          consumedCredits: 30,
        })),
      },
      fit: {
        status: "PASSED",
        algorithm: "ITEM_REFERENCE_SLOT_FRAME_FIT_V1",
        solutionSha256: fitReport.solutionSha256,
        referenceProfileSha256: attachmentProfile.profileSha256,
        parts: fitReport.parts.map((part) => ({
          field: part.field,
          transformSha256: part.transformSha256,
        })),
      },
      iconPresentationFit: {
        status: "PASSED",
        algorithm: "ITEM_MODELTYPE2_FULL_FRAME_CONNECTOR_FIT_V4_AURORA_YZX",
        solutionSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
        targetAxialLengths: [0.22, 0.08, 0.90],
        worldAttachmentUnaffected: true,
        parts: fields.map((field) => ({
          field,
          sourceSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
          transformSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
        })),
      },
      attachmentProfile: {
        profileSha256: attachmentProfile.profileSha256,
      },
      itemPropertiesModelConformance: {
        status: "PASSED",
        algorithm: "ITEM_MODELTYPE2_AURORA_APPEND_CONFORMANCE_V2",
        axialTargetAxis: 1,
        checkedMdlCount: 12,
        appendOrder: fields,
        colorways: [
          { color: 1, totalTriangleCount: expect.any(Number) },
          { color: 2, totalTriangleCount: expect.any(Number) },
          { color: 3, totalTriangleCount: expect.any(Number) },
          { color: 4, totalTriangleCount: expect.any(Number) },
        ],
      },
      itemIconConformance: {
        status: "PASSED",
        algorithm: "AURORA_MODELTYPE2_ICON_LAYER_COMPOSITE_V3",
        layoutProfile: "LONG_VERTICAL_PART_ORDER_V2",
        colorways: [
          { color: 1, axialFillRatio: expect.any(Number), partOrderStatus: "PASSED" },
          { color: 2, axialFillRatio: expect.any(Number), partOrderStatus: "PASSED" },
          { color: 3, axialFillRatio: expect.any(Number), partOrderStatus: "PASSED" },
          { color: 4, axialFillRatio: expect.any(Number), partOrderStatus: "PASSED" },
        ],
      },
      proofModule: {
        fixtureProfile: "ITEM_ONLY_GROUND_ITEM_V1",
        groundItemCount: 1,
        creatureCount: 0,
        semanticReadbackStatus: "PASS",
      },
      sourceBundle: {
        schema: "meshy2aurora.sample-3d/v1",
        assetId: "item-4-item-worker-generation",
        manifestFileName: "manifest.yaml",
        manifestSha256: expect.stringMatching(/^[a-f0-9]{64}$/),
        files: fields.map((field, index) => ({
          field,
          role: `source-modelpart-${["bottom", "middle", "top"][index]}`,
          fileName: `${["bottom", "middle", "top"][index]}.glb`,
          byteLength: sourceLengths[index],
          sha256: sourceHashes[index],
        })),
      },
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
    expect(response.artifacts.filter(({ kind }) => kind === "MODEL")).toHaveLength(12);
    expect(response.artifacts.filter(({ kind }) => kind === "MODEL").map(
      ({ fileName }) => fileName,
    )).toEqual(itemRequest.parts.flatMap((part) => (
      part.weaponColorways.map((colorway) => `${colorway.modelResref}.mdl`)
    )));
    expect(response.artifacts.filter(({ kind }) => kind === "TEXTURE")).toHaveLength(24);
    const concreteColorwayTextures = response.artifacts.filter(
      ({ artifactId }) => artifactId.startsWith("item-part-") && artifactId.endsWith("-texture"),
    );
    expect(concreteColorwayTextures).toHaveLength(12);
    for (let partIndex = 0; partIndex < 3; partIndex += 1) {
      expect(new Set(concreteColorwayTextures.slice(partIndex * 4, partIndex * 4 + 4).map(
        ({ sha256: textureSha256 }) => textureSha256,
      )).size).toBe(4);
    }
    const iconLayers = response.artifacts.filter(
      ({ artifactId }) => artifactId.startsWith("item-part-") && artifactId.endsWith("-icon"),
    );
    expect(iconLayers).toHaveLength(12);
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
      expect.objectContaining({ kind: "SOURCE_MANIFEST", fileName: "manifest.yaml" }),
    ]));
    const sourceModels = response.artifacts.filter(({ kind }) => kind === "SOURCE_MODEL");
    expect(sourceModels.map(({ fileName, byteLength, sha256 }) => ({ fileName, byteLength, sha256 })))
      .toEqual(["bottom", "middle", "top"].map((role, index) => ({
        fileName: `${role}.glb`,
        byteLength: sourceLengths[index],
        sha256: sourceHashes[index],
      })));
    const sourceManifest = response.artifacts.find(({ kind }) => kind === "SOURCE_MANIFEST");
    const sourceManifestText = new TextDecoder().decode(sourceManifest!.bytes);
    expect(sourceManifest!.sha256).toBe(await sha256(sourceManifest!.bytes));
    expect(sourceManifestText).toContain("schema: meshy2aurora.sample-3d/v1");
    expect(sourceManifestText).toContain("asset_id: item-4-item-worker-generation");
    expect(sourceManifestText).toContain(`sha256: ${sourceHashes[0]}`);
    expect(sourceManifestText).toContain(`solution_sha256: ${fitReport.solutionSha256}`);
    expect(sourceManifestText).not.toMatch(/https?:\/\/|authorization|api[_-]?key|signedUrl/i);
    const hak = response.artifacts.find(({ kind }) => kind === "HAK");
    const module = response.artifacts.find(({ kind }) => kind === "MODULE");
    const uti = response.artifacts.find(({ kind }) => kind === "ITEM_BLUEPRINT");
    const report = response.artifacts.find(({ fileName }) => fileName === "item-build-report.json");
    expect(new TextDecoder().decode(hak!.bytes.slice(0, 8))).toBe("HAK V1.0");
    expect(new TextDecoder().decode(module!.bytes.slice(0, 8))).toBe("MOD V1.0");
    expect(new TextDecoder().decode(module!.bytes)).not.toContain("m2aitemnpc4");
    expect(new TextDecoder().decode(uti!.bytes.slice(0, 8))).toBe("UTI V3.2");
    expect(hak!.sha256).toBe(await sha256(hak!.bytes));
    expect(uti!.sha256).toBe(await sha256(uti!.bytes));
    expect(new TextDecoder().decode(report!.bytes)).toBe(response.reportJson);

    const incompleteColorwayBaseitems = await fetchBytes(baseitemsUrl);
    const incompleteColorwaySources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    await expect(client.request({
      ...itemRequest,
      requestId: "item-colorway-incomplete",
      baseitemsTwoDa: incompleteColorwayBaseitems,
      parts: itemRequest.parts.map((part, index) => ({
        ...part,
        sourceGlb: incompleteColorwaySources[index],
        weaponColorways: index === 0 ? part.weaponColorways.slice(0, 3) : part.weaponColorways,
      })),
    }, [incompleteColorwayBaseitems, ...incompleteColorwaySources])).rejects.toThrow(
      "ITEM-WEAPON-COLORWAY-COVERAGE-INCOMPLETE",
    );

    const weaponPltBaseitems = await fetchBytes(baseitemsUrl);
    const weaponPltSources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    await expect(client.request({
      ...itemRequest,
      requestId: "item-modeltype2-plt",
      baseitemsTwoDa: weaponPltBaseitems,
      parts: itemRequest.parts.map((part, index) => ({
        ...part,
        sourceGlb: weaponPltSources[index],
        textureEncoding: index === 0 ? "PLT_METAL1" as const : part.textureEncoding,
      })),
    }, [weaponPltBaseitems, ...weaponPltSources])).rejects.toThrow(
      "ITEM-MODELTYPE2-PLT-UNSUPPORTED",
    );

    const taskTamperBaseitems = await fetchBytes(baseitemsUrl);
    const taskTamperSources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    const taskTamperSession = JSON.parse(generationSessionJson);
    taskTamperSession.slots[0].run.taskId = "task-substituted";
    await expect(client.request({
      ...itemRequest,
      requestId: "item-generation-task-tamper",
      baseitemsTwoDa: taskTamperBaseitems,
      generationSessionJson: JSON.stringify(taskTamperSession),
      parts: itemRequest.parts.map((part, index) => ({ ...part, sourceGlb: taskTamperSources[index] })),
    }, [taskTamperBaseitems, ...taskTamperSources])).rejects.toThrow(
      "ITEM-GENERATION-PROVENANCE-TASK-MISMATCH",
    );

    const tamperedBaseitems = await fetchBytes(baseitemsUrl);
    const tamperedSources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    const tamperedFirst = new Uint8Array(tamperedSources[0]);
    tamperedFirst[tamperedFirst.length - 1] ^= 1;
    await expect(client.request({
      ...itemRequest,
      requestId: "item-generation-source-tamper",
      baseitemsTwoDa: tamperedBaseitems,
      parts: itemRequest.parts.map((part, index) => ({ ...part, sourceGlb: tamperedSources[index] })),
    }, [tamperedBaseitems, ...tamperedSources])).rejects.toThrow(
      "ITEM-GENERATION-PROVENANCE-SLOT-MISMATCH",
    );

    const fitTamperBaseitems = await fetchBytes(baseitemsUrl);
    const fitTamperSources = await Promise.all([
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
      fetchBytes(sourceUrl).then(asStaticItemPart),
    ]);
    await expect(client.request({
      ...itemRequest,
      requestId: "item-fit-transform-tamper",
      baseitemsTwoDa: fitTamperBaseitems,
      generationSessionJson: null,
      generationArtifactsJson: null,
      parts: itemRequest.parts.map((part, index) => ({
        ...part,
        sourceGlb: fitTamperSources[index],
        transformJson: index === 0
          ? JSON.stringify({ ...JSON.parse(part.transformJson), translation: [99, 0, 0] })
          : part.transformJson,
      })),
    }, [fitTamperBaseitems, ...fitTamperSources])).rejects.toThrow(
      "ITEM-FIT-PROVENANCE-MISMATCH",
    );

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
      generationSessionJson: null,
      generationArtifactsJson: null,
      fitReportJson: null,
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
      "ITEM-MODELTYPE2-FIT-REQUIRED",
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
      "16 Armor armor 3 0 it_bag iit_chest 2 2 3",
      "80 Cloak cloak 1 1 it_bag icloak 8192 2 2",
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
      generationSessionJson: null,
      generationArtifactsJson: null,
      fitReportJson: null,
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
      "80 Cloak cloak 1 1 it_bag icloak 8192 2 2",
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
      generationSessionJson: null,
      generationArtifactsJson: null,
      fitReportJson: null,
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
