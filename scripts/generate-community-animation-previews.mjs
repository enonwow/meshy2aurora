import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const libraryRoot = join(repositoryRoot, "animation-library");
const profile = JSON.parse(readFileSync(
  join(libraryRoot, "rig-profiles", "m2a-humanoid-strict-v1.json"),
  "utf8",
));
const playwrightUrl = pathToFileURL(join(
  repositoryRoot,
  "apps",
  "studio-web",
  "node_modules",
  "playwright",
  "index.mjs",
)).href;
const { chromium } = await import(playwrightUrl);
const browser = await chromium.launch({ headless: true });

try {
  const page = await browser.newPage({ viewport: { width: 960, height: 360 } });
  await page.setContent('<canvas id="preview" width="960" height="360"></canvas>');
  const presetDirectories = readdirSync(join(libraryRoot, "presets"), {
    withFileTypes: true,
  }).filter((entry) => entry.isDirectory()).map(({ name }) => name).sort();
  for (const presetId of presetDirectories) {
    const directory = join(libraryRoot, "presets", presetId);
    const manifest = JSON.parse(readFileSync(join(directory, "manifest.json"), "utf8"));
    const animation = JSON.parse(readFileSync(join(directory, "animation.json"), "utf8"));
    const preview = buildPreviewProjection(profile, animation);
    const dataUrl = await page.evaluate(({ frames, edges, durationSeconds, loop }) => {
      const canvas = document.querySelector("#preview");
      if (!(canvas instanceof HTMLCanvasElement)) throw new Error("preview canvas missing");
      const context = canvas.getContext("2d");
      if (!context) throw new Error("preview canvas context missing");
      const width = canvas.width;
      const height = canvas.height;
      const panelWidth = width / frames.length;
      context.fillStyle = "#0b0f19";
      context.fillRect(0, 0, width, height);
      const allPoints = frames.flatMap(({ points }) => Object.values(points));
      const minX = Math.min(...allPoints.map(([x]) => x));
      const maxX = Math.max(...allPoints.map(([x]) => x));
      const minY = Math.min(...allPoints.map(([, y]) => y));
      const maxY = Math.max(...allPoints.map(([, y]) => y));
      const scale = Math.min(
        (panelWidth - 34) / Math.max(maxX - minX, 0.01),
        (height - 64) / Math.max(maxY - minY, 0.01),
      );
      const project = ([x, y], index) => [
        index * panelWidth + panelWidth / 2 + (x - (minX + maxX) / 2) * scale,
        height - 34 - (y - minY) * scale,
      ];
      frames.forEach((frame, index) => {
        context.fillStyle = index % 2 === 0 ? "#111827" : "#0f1725";
        context.fillRect(index * panelWidth, 0, panelWidth, height);
        context.strokeStyle = "#26354f";
        context.lineWidth = 1;
        context.beginPath();
        context.moveTo(index * panelWidth + 0.5, 0);
        context.lineTo(index * panelWidth + 0.5, height);
        context.stroke();
        context.lineCap = "round";
        for (const [parent, child] of edges) {
          const first = frame.points[parent];
          const second = frame.points[child];
          if (!first || !second) continue;
          const [x1, y1] = project(first, index);
          const [x2, y2] = project(second, index);
          const active = frame.activeBones.includes(child) || frame.activeBones.includes(parent);
          context.strokeStyle = active ? "#a78bfa" : "#8292ac";
          context.lineWidth = active ? 4.5 : 3;
          context.beginPath();
          context.moveTo(x1, y1);
          context.lineTo(x2, y2);
          context.stroke();
        }
        const head = frame.points.Head;
        if (head) {
          const [x, y] = project(head, index);
          context.fillStyle = "#c4b5fd";
          context.beginPath();
          context.arc(x, y, 7, 0, Math.PI * 2);
          context.fill();
        }
        context.fillStyle = "#6d28d9";
        context.fillRect(index * panelWidth + 15, 13, (panelWidth - 30) * frame.progress, 4);
        context.strokeStyle = "#334155";
        context.strokeRect(index * panelWidth + 15, 13, panelWidth - 30, 4);
      });
      context.fillStyle = loop ? "#22c55e" : "#f59e0b";
      context.beginPath();
      context.arc(width - 18, height - 17, 5, 0, Math.PI * 2);
      context.fill();
      context.fillStyle = "#7c3aed";
      context.fillRect(0, height - 5, width, 5);
      context.fillStyle = "#c4b5fd";
      context.fillRect(0, height - 5, width * Math.min(durationSeconds / 2, 1), 5);
      return canvas.toDataURL("image/webp", 0.88);
    }, {
      ...preview,
      durationSeconds: animation.durationSeconds,
      loop: manifest.playback === "LOOP",
    });
    const payload = Buffer.from(dataUrl.slice(dataUrl.indexOf(",") + 1), "base64");
    writeFileSync(join(directory, "preview.webp"), payload);
    process.stdout.write(`animation-preview-written: ${presetId} (${payload.byteLength} bytes)\n`);
  }
} finally {
  await browser.close();
}

function buildPreviewProjection(rigProfile, animation) {
  const hidden = new Set(["head_end", "headfront"]);
  const nodes = new Map(rigProfile.nodes.map((node) => [node.name, node]));
  const edges = rigProfile.nodes
    .filter(({ name, parentName }) => parentName && !hidden.has(name) && !hidden.has(parentName))
    .map(({ name, parentName }) => [parentName, name]);
  const times = [...new Set(animation.tracks.flatMap(({ keyframes }) => (
    keyframes.map(({ timeSeconds }) => timeSeconds)
  )))].sort((left, right) => left - right);
  const sampleTimes = selectFivePhases(times, animation.durationSeconds);
  const tracks = new Map(animation.tracks.map((track) => (
    [`${track.targetBoneName}:${track.path}`, track]
  )));
  return {
    edges,
    frames: sampleTimes.map((time) => {
      const world = new Map();
      const activeBones = [];
      const visit = (name) => {
        if (world.has(name)) return world.get(name);
        const node = nodes.get(name);
        if (!node) throw new Error(`unknown rig node ${name}`);
        const translationTrack = tracks.get(`${name}:TRANSLATION`);
        const rotationTrack = tracks.get(`${name}:ROTATION`);
        const translation = translationTrack
          ? sampleTrack(translationTrack, time)
          : node.translation;
        const rotation = rotationTrack
          ? sampleTrack(rotationTrack, time)
          : node.rotation;
        if (translationTrack || rotationTrack) activeBones.push(name);
        if (!node.parentName) {
          const result = { position: translation, rotation };
          world.set(name, result);
          return result;
        }
        const parent = visit(node.parentName);
        const result = {
          position: add(parent.position, rotate(parent.rotation, translation)),
          rotation: normalizeQuaternion(multiplyQuaternion(parent.rotation, rotation)),
        };
        world.set(name, result);
        return result;
      };
      const points = {};
      for (const name of nodes.keys()) {
        if (hidden.has(name)) continue;
        const { position } = visit(name);
        points[name] = [position[0], position[2]];
      }
      return {
        points,
        activeBones,
        progress: time / animation.durationSeconds,
      };
    }),
  };
}

function selectFivePhases(times, duration) {
  if (times.length <= 5) return [...times, ...Array(5 - times.length).fill(duration)].slice(0, 5);
  return [0, 0.25, 0.5, 0.75, 1].map((ratio) => {
    const target = duration * ratio;
    return times.reduce((best, time) => (
      Math.abs(time - target) < Math.abs(best - target) ? time : best
    ), times[0]);
  });
}

function sampleTrack(track, time) {
  const keys = track.keyframes;
  if (time <= keys[0].timeSeconds) return keys[0].value;
  if (time >= keys.at(-1).timeSeconds) return keys.at(-1).value;
  const right = keys.findIndex(({ timeSeconds }) => timeSeconds >= time);
  const first = keys[right - 1];
  const second = keys[right];
  const ratio = (time - first.timeSeconds) / (second.timeSeconds - first.timeSeconds);
  const value = first.value.map((component, index) => (
    component + (second.value[index] - component) * ratio
  ));
  if (track.path !== "ROTATION") return value;
  const dot = first.value.reduce((sum, component, index) => (
    sum + component * second.value[index]
  ), 0);
  if (dot < 0) {
    for (let index = 0; index < value.length; index += 1) {
      value[index] = first.value[index] + (-second.value[index] - first.value[index]) * ratio;
    }
  }
  return normalizeQuaternion(value);
}

function add(left, right) {
  return left.map((value, index) => value + right[index]);
}

function normalizeQuaternion(value) {
  const length = Math.hypot(...value);
  return value.map((component) => component / length);
}

function multiplyQuaternion([ax, ay, az, aw], [bx, by, bz, bw]) {
  return [
    aw * bx + ax * bw + ay * bz - az * by,
    aw * by - ax * bz + ay * bw + az * bx,
    aw * bz + ax * by - ay * bx + az * bw,
    aw * bw - ax * bx - ay * by - az * bz,
  ];
}

function rotate(rotation, vector) {
  const point = [...vector, 0];
  const inverse = [-rotation[0], -rotation[1], -rotation[2], rotation[3]];
  return multiplyQuaternion(multiplyQuaternion(rotation, point), inverse).slice(0, 3);
}
