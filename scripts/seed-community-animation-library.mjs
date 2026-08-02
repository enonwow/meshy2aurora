import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const libraryRoot = path.join(repositoryRoot, "animation-library");
const presetRoot = path.join(libraryRoot, "presets");
const profile = JSON.parse(readFileSync(
  path.join(libraryRoot, "rig-profiles", "m2a-humanoid-strict-v1.json"),
  "utf8",
));
const nodes = new Map(profile.nodes.map((node) => [node.name, node]));

const specs = [
  {
    id: "m2a_right_cross",
    outputName: "m2a_rightcross",
    label: "Right cross",
    summary: "Compact boxing guard, straight right, impact extension and recoil.",
    tags: ["attack", "boxing", "humanoid", "one-shot", "right-hand", "unarmed", "upper-body"],
    playback: "ONE_SHOT",
    duration: 0.85,
    times: [0, 0.12, 0.25, 0.38, 0.58, 0.85],
    rotations: {
      Spine02: [[0,0,0],[0,0,6],[0,0,15],[0,0,-18],[0,0,-6],[0,0,0]],
      Spine: [[0,0,0],[0,0,10],[0,0,22],[0,0,-28],[0,0,-10],[0,0,0]],
      RightShoulder: [[20,0,-20],[12,0,-10],[-20,5,30],[-35,0,55],[-10,0,15],[20,0,-20]],
      RightArm: [[-25,35,10],[-40,50,20],[20,15,60],[75,-10,90],[0,20,35],[-25,35,10]],
      RightForeArm: [[-100,30,-55],[-125,25,-65],[-50,10,-20],[0,0,0],[-70,20,-35],[-100,30,-55]],
      LeftArm: [[-5,-20,-20],[-5,-20,-20],[-10,-18,-25],[-8,-18,-22],[-5,-20,-20],[-5,-20,-20]],
      LeftForeArm: [[105,-65,-100],[105,-65,-100],[110,-68,-98],[108,-66,-100],[105,-65,-100],[105,-65,-100]],
    },
    translations: { Hips: [[0,0,0],[-0.01,-0.01,-0.015],[0.01,0.04,0],[0.015,0.08,0.01],[0,0.02,0],[0,0,0]] },
  },
  {
    id: "m2a_left_jab",
    outputName: "m2a_leftjab",
    label: "Left jab",
    summary: "Fast lead-hand jab from a guarded stance with a short snap-back.",
    tags: ["attack", "boxing", "humanoid", "left-hand", "one-shot", "unarmed", "upper-body"],
    playback: "ONE_SHOT",
    duration: 0.62,
    times: [0, 0.1, 0.2, 0.31, 0.44, 0.62],
    rotations: {
      Spine02: [[0,0,0],[0,0,-4],[0,0,-9],[0,0,10],[0,0,3],[0,0,0]],
      Spine: [[0,0,0],[0,0,-7],[0,0,-14],[0,0,16],[0,0,5],[0,0,0]],
      LeftShoulder: [[20,0,20],[12,0,10],[-18,-5,-25],[-30,0,-48],[-8,0,-12],[20,0,20]],
      LeftArm: [[-25,-35,-10],[-38,-48,-18],[15,-12,-50],[66,8,-82],[-2,-18,-28],[-25,-35,-10]],
      LeftForeArm: [[100,-30,55],[120,-24,62],[45,-8,18],[0,0,0],[65,-18,30],[100,-30,55]],
      RightArm: [[-5,20,20],[-5,20,20],[-8,18,22],[-7,18,20],[-5,20,20],[-5,20,20]],
      RightForeArm: [[-105,65,100],[-105,65,100],[-108,67,98],[-107,66,99],[-105,65,100],[-105,65,100]],
    },
    translations: { Hips: [[0,0,0],[0.005,-0.005,-0.01],[-0.005,0.025,0],[-0.01,0.045,0.005],[0,0.01,0],[0,0,0]] },
  },
  {
    id: "m2a_right_hook",
    outputName: "m2a_righthook",
    label: "Right hook",
    summary: "Loaded right hook driven by hip and torso rotation, followed by recovery.",
    tags: ["attack", "boxing", "full-body", "humanoid", "one-shot", "right-hand", "unarmed"],
    playback: "ONE_SHOT",
    duration: 0.92,
    times: [0, 0.16, 0.32, 0.46, 0.68, 0.92],
    rotations: {
      Hips: [[0,0,0],[0,0,12],[0,0,25],[0,0,-35],[0,0,-12],[0,0,0]],
      Spine02: [[0,0,0],[0,0,10],[0,0,22],[0,0,-30],[0,0,-10],[0,0,0]],
      Spine: [[0,0,0],[0,0,14],[0,0,30],[0,0,-42],[0,0,-14],[0,0,0]],
      RightShoulder: [[18,0,-18],[5,0,15],[-30,5,55],[-20,-5,105],[2,0,25],[18,0,-18]],
      RightArm: [[-20,35,10],[-35,20,45],[-10,-10,88],[35,-25,120],[-5,15,50],[-20,35,10]],
      RightForeArm: [[-105,30,-55],[-115,20,-45],[-105,10,-20],[-95,-5,25],[-110,20,-35],[-105,30,-55]],
      LeftArm: [[-5,-20,-20],[-8,-22,-24],[-10,-25,-28],[-8,-20,-22],[-5,-20,-20],[-5,-20,-20]],
    },
    translations: { Hips: [[0,0,0],[-0.01,-0.02,-0.02],[-0.015,-0.03,-0.025],[0.02,0.03,0],[0.005,0.01,0],[0,0,0]] },
  },
  {
    id: "m2a_uppercut",
    outputName: "m2a_uppercut",
    label: "Right uppercut",
    summary: "Low compression, rising hip drive and a compact right uppercut.",
    tags: ["attack", "boxing", "full-body", "humanoid", "one-shot", "right-hand", "unarmed"],
    playback: "ONE_SHOT",
    duration: 1.0,
    times: [0, 0.18, 0.36, 0.52, 0.72, 1.0],
    rotations: {
      Hips: [[0,0,0],[8,0,8],[12,0,14],[-8,0,-18],[-3,0,-6],[0,0,0]],
      Spine02: [[0,0,0],[10,0,6],[16,0,12],[-14,0,-16],[-4,0,-5],[0,0,0]],
      Spine: [[0,0,0],[12,0,8],[20,0,16],[-22,0,-24],[-6,0,-8],[0,0,0]],
      RightShoulder: [[18,0,-18],[35,0,0],[55,5,25],[-35,-5,45],[-5,0,12],[18,0,-18]],
      RightArm: [[-20,35,10],[-55,25,25],[-80,10,45],[35,-15,80],[0,15,35],[-20,35,10]],
      RightForeArm: [[-105,30,-55],[-130,20,-40],[-145,10,-20],[-70,-5,15],[-95,20,-35],[-105,30,-55]],
      LeftArm: [[-5,-20,-20],[-8,-22,-25],[-12,-25,-30],[-8,-18,-22],[-5,-20,-20],[-5,-20,-20]],
    },
    translations: { Hips: [[0,0,0],[0,-0.02,-0.08],[0,-0.03,-0.14],[0.01,0.05,0.08],[0,0.02,0.02],[0,0,0]] },
  },
  {
    id: "m2a_combat_guard",
    outputName: "m2a_guard",
    label: "Combat guard",
    summary: "Looping boxing guard with subtle breathing and balanced footwork.",
    tags: ["boxing", "combat", "defense", "full-body", "guard", "humanoid", "loop", "unarmed"],
    playback: "LOOP",
    duration: 1.6,
    times: [0, 0.4, 0.8, 1.2, 1.6],
    rotations: {
      Spine02: [[2,0,2],[4,0,3],[1,0,1],[-1,0,0],[2,0,2]],
      Spine: [[4,0,3],[7,0,4],[3,0,2],[1,0,1],[4,0,3]],
      RightArm: [[-25,35,10],[-27,36,12],[-24,34,9],[-23,35,8],[-25,35,10]],
      RightForeArm: [[-105,30,-55],[-108,31,-56],[-103,29,-54],[-104,30,-55],[-105,30,-55]],
      LeftArm: [[-25,-35,-10],[-27,-36,-12],[-24,-34,-9],[-23,-35,-8],[-25,-35,-10]],
      LeftForeArm: [[105,-30,55],[108,-31,56],[103,-29,54],[104,-30,55],[105,-30,55]],
      Head: [[0,0,0],[-1,0,-1],[0,0,0],[1,0,1],[0,0,0]],
    },
    translations: { Hips: [[0,0,0],[0,0,-0.01],[0,0,-0.02],[0,0,-0.01],[0,0,0]] },
  },
  {
    id: "m2a_dodge_left",
    outputName: "m2a_dodgeleft",
    label: "Dodge left",
    summary: "Quick full-body slip to the left with guarded hands and centered recovery.",
    tags: ["combat", "defense", "dodge", "full-body", "humanoid", "one-shot", "unarmed"],
    playback: "ONE_SHOT",
    duration: 0.78,
    times: [0, 0.12, 0.26, 0.42, 0.58, 0.78],
    rotations: {
      Hips: [[0,0,0],[0,0,8],[0,0,15],[0,0,10],[0,0,3],[0,0,0]],
      Spine02: [[0,0,0],[0,0,-12],[0,0,-24],[0,0,-16],[0,0,-5],[0,0,0]],
      Spine: [[0,0,0],[0,0,-18],[0,0,-34],[0,0,-22],[0,0,-7],[0,0,0]],
      RightArm: [[-25,35,10],[-28,38,12],[-32,42,15],[-30,40,13],[-26,36,11],[-25,35,10]],
      LeftArm: [[-25,-35,-10],[-22,-32,-8],[-18,-28,-5],[-20,-30,-6],[-24,-34,-9],[-25,-35,-10]],
      LeftUpLeg: [[0,0,0],[5,0,-8],[10,0,-15],[6,0,-10],[2,0,-3],[0,0,0]],
      RightUpLeg: [[0,0,0],[-4,0,6],[-8,0,12],[-5,0,8],[-1,0,2],[0,0,0]],
    },
    translations: { Hips: [[0,0,0],[0.05,0,-0.02],[0.16,0,-0.05],[0.2,0.01,-0.04],[0.08,0,0],[0,0,0]] },
  },
  {
    id: "m2a_dodge_right",
    outputName: "m2a_dodgeright",
    label: "Dodge right",
    summary: "Quick full-body slip to the right with guarded hands and centered recovery.",
    tags: ["combat", "defense", "dodge", "full-body", "humanoid", "one-shot", "unarmed"],
    playback: "ONE_SHOT",
    duration: 0.78,
    times: [0, 0.12, 0.26, 0.42, 0.58, 0.78],
    rotations: {
      Hips: [[0,0,0],[0,0,-8],[0,0,-15],[0,0,-10],[0,0,-3],[0,0,0]],
      Spine02: [[0,0,0],[0,0,12],[0,0,24],[0,0,16],[0,0,5],[0,0,0]],
      Spine: [[0,0,0],[0,0,18],[0,0,34],[0,0,22],[0,0,7],[0,0,0]],
      LeftArm: [[-25,-35,-10],[-28,-38,-12],[-32,-42,-15],[-30,-40,-13],[-26,-36,-11],[-25,-35,-10]],
      RightArm: [[-25,35,10],[-22,32,8],[-18,28,5],[-20,30,6],[-24,34,9],[-25,35,10]],
      RightUpLeg: [[0,0,0],[5,0,8],[10,0,15],[6,0,10],[2,0,3],[0,0,0]],
      LeftUpLeg: [[0,0,0],[-4,0,-6],[-8,0,-12],[-5,0,-8],[-1,0,-2],[0,0,0]],
    },
    translations: { Hips: [[0,0,0],[-0.05,0,-0.02],[-0.16,0,-0.05],[-0.2,0.01,-0.04],[-0.08,0,0],[0,0,0]] },
  },
];

if (existsSync(presetRoot)) {
  throw new Error(`Refusing to overwrite existing starter preset root: ${presetRoot}`);
}
mkdirSync(presetRoot, { recursive: true });

for (const spec of specs) {
  const directory = path.join(presetRoot, spec.id);
  mkdirSync(directory);
  const animation = {
    schemaVersion: 1,
    durationSeconds: spec.duration,
    animationRootBoneName: profile.animationRootBoneName,
    tracks: [
      ...Object.entries(spec.rotations).map(([bone, eulers]) => rotationTrack(bone, spec.times, eulers)),
      ...Object.entries(spec.translations).map(([bone, deltas]) => translationTrack(bone, spec.times, deltas)),
    ].sort((left, right) => (
      left.targetBoneName.localeCompare(right.targetBoneName)
      || left.path.localeCompare(right.path)
    )),
    events: [],
  };
  const animationBytes = JSON.stringify(animation);
  writeFileSync(path.join(directory, "animation.json"), animationBytes);
  const manifest = {
    schemaVersion: 1,
    presetId: spec.id,
    presetVersion: 1,
    outputName: spec.outputName,
    label: spec.label,
    summary: spec.summary,
    source: "BUILT_IN",
    authors: [{ name: "Meshy2Aurora contributors" }],
    license: "LicenseRef-Meshy2Aurora-Project-Generated",
    tags: [...spec.tags].sort(),
    playback: spec.playback,
    durationSeconds: spec.duration,
    rigProfile: "M2A_HUMANOID_STRICT_V1",
    rigSignatureSha256: profile.signatureSha256,
    requiredBones: [],
    animationPath: "animation.json",
    animationByteLength: Buffer.byteLength(animationBytes),
    animationSha256: sha256(animationBytes),
    motionSha256: "0".repeat(64),
    previewPath: null,
    previewByteLength: null,
    previewSha256: null,
    validationStatus: "PIPELINE_VERIFIED",
  };
  writeFileSync(path.join(directory, "manifest.json"), JSON.stringify(manifest));
  writeFileSync(
    path.join(directory, "README.md"),
    `# ${spec.label}\n\n${spec.summary}\n\nGenerated by Meshy2Aurora from project-owned procedural motion.\n`,
  );
}

runCore([
  "hydrate-manifests",
  "--root",
  libraryRoot,
  "--rig-profile",
  path.join(libraryRoot, "rig-profiles", "m2a-humanoid-strict-v1.json"),
]);
runCore([
  "catalog",
  "--root",
  libraryRoot,
  "--output",
  path.join(repositoryRoot, "contracts", "community-animation-catalog-v1.json"),
  "--write",
]);

function rotationTrack(bone, times, eulers) {
  const node = requiredNode(bone);
  return {
    targetBoneName: bone,
    path: "ROTATION",
    interpolation: "LINEAR",
    keyframes: times.map((timeSeconds, index) => ({
      timeSeconds,
      value: canonicalQuaternion(multiplyQuaternion(
        node.rotation,
        quaternionFromEulerDegrees(eulers[index]),
      )),
    })),
  };
}

function translationTrack(bone, times, deltas) {
  const node = requiredNode(bone);
  return {
    targetBoneName: bone,
    path: "TRANSLATION",
    interpolation: "LINEAR",
    keyframes: times.map((timeSeconds, index) => ({
      timeSeconds,
      value: node.translation.map((component, axis) => component + deltas[index][axis]),
    })),
  };
}

function requiredNode(name) {
  const node = nodes.get(name);
  if (!node) throw new Error(`Rig profile does not contain ${name}`);
  return node;
}

function quaternionFromEulerDegrees([x, y, z]) {
  const qx = axisAngle([1, 0, 0], x * Math.PI / 180);
  const qy = axisAngle([0, 1, 0], y * Math.PI / 180);
  const qz = axisAngle([0, 0, 1], z * Math.PI / 180);
  return multiplyQuaternion(multiplyQuaternion(qz, qy), qx);
}

function axisAngle(axis, radians) {
  const half = radians / 2;
  const sine = Math.sin(half);
  return [axis[0] * sine, axis[1] * sine, axis[2] * sine, Math.cos(half)];
}

function multiplyQuaternion(left, right) {
  const [lx, ly, lz, lw] = left;
  const [rx, ry, rz, rw] = right;
  return [
    lw * rx + lx * rw + ly * rz - lz * ry,
    lw * ry - lx * rz + ly * rw + lz * rx,
    lw * rz + lx * ry - ly * rx + lz * rw,
    lw * rw - lx * rx - ly * ry - lz * rz,
  ];
}

function canonicalQuaternion(value) {
  const length = Math.hypot(...value);
  let result = value.map((component) => component / length);
  const first = [result[3], result[2], result[1], result[0]].find((component) => Math.abs(component) > 1e-7) ?? 1;
  if (first < 0) result = result.map((component) => -component);
  return result.map((component) => Object.is(component, -0) ? 0 : component);
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function runCore(command) {
  const result = spawnSync("cargo", [
    "run",
    "--quiet",
    "-p",
    "m2a-core",
    "--example",
    "community_animation_library",
    "--",
    ...command,
  ], {
    cwd: repositoryRoot,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Core library command failed: ${command.join(" ")}`);
}
