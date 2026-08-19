import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2 } from "../features/item/itemAuthoringRecipeV2";
import {
  authoredItemMatrix,
  itemHorizontalBroadsideCameraFrame,
  itemPreviewGroundingMatrix,
} from "../features/item/itemTransform";

function requireElement<T extends Element>(selector: string) {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`Hextech Shotgun proof is missing ${selector}.`);
  return element;
}

const canvas = requireElement<HTMLCanvasElement>("#viewport");
const loading = requireElement<HTMLElement>("#loading");
const status = requireElement<HTMLElement>("#status");
const partsHost = requireElement<HTMLElement>("#parts");

const partPresentation = {
  ModelPart1: { label: "Bottom · stock", fileName: "bottom.glb" },
  ModelPart2: { label: "Middle · receiver", fileName: "middle.glb" },
  ModelPart3: { label: "Top · twin barrels", fileName: "top.glb" },
} as const;

function parseGlb(loader: GLTFLoader, bytes: ArrayBuffer) {
  return new Promise<THREE.Group>((resolve, reject) => {
    loader.parse(bytes, "", (gltf) => resolve(gltf.scene), reject);
  });
}

async function sha256(bytes: ArrayBuffer) {
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((value) => value.toString(16).padStart(2, "0"))
    .join("");
}

function appendPartProof(
  label: string,
  hash: string,
  translation: readonly number[],
  rotationXyzw: readonly number[],
  uniformScale: number,
) {
  const article = document.createElement("article");
  const heading = document.createElement("strong");
  heading.textContent = label;
  const code = document.createElement("code");
  code.textContent = hash;
  const transform = document.createElement("small");
  transform.textContent = `T ${JSON.stringify(translation)} · Q ${JSON.stringify(rotationXyzw)} · S ${uniformScale}`;
  article.append(heading, code, transform);
  partsHost.append(article);
}

const scene = new THREE.Scene();
scene.background = new THREE.Color("#596164");
const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.001, 10_000);
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
const controls = new OrbitControls(camera, canvas);
controls.enableDamping = true;
scene.add(new THREE.HemisphereLight(0xc8f8ff, 0x293238, 3));
scene.add(new THREE.AmbientLight(0xffffff, 1.4));
const key = new THREE.DirectionalLight(0xffffff, 3.5);
scene.add(key, key.target);
const content = new THREE.Group();
scene.add(content);
const loader = new GLTFLoader();
let viewHalfHeight = 1;

function resize() {
  const width = Math.max(1, canvas.clientWidth);
  const height = Math.max(1, canvas.clientHeight);
  renderer.setSize(width, height, false);
  const aspect = width / height;
  camera.left = -viewHalfHeight * aspect;
  camera.right = viewHalfHeight * aspect;
  camera.top = viewHalfHeight;
  camera.bottom = -viewHalfHeight;
  camera.updateProjectionMatrix();
}

new ResizeObserver(resize).observe(canvas);
resize();

function render() {
  controls.update();
  renderer.render(scene, camera);
  requestAnimationFrame(render);
}
render();

void (async () => {
  try {
    const loaded = await Promise.all(
      HEXTECH_SHOTGUN_OWNER_DIRECTED_COMPOSITION_V2.parts.map(async (part) => {
        const presentation = partPresentation[part.field];
        const response = await fetch(
          `/__m2a-proof/hextech-shotgun/${presentation.fileName}`,
          { cache: "no-store" },
        );
        if (!response.ok) {
          throw new Error(`${presentation.fileName}: canonical source is unavailable (${response.status}).`);
        }
        const bytes = await response.arrayBuffer();
        const observedHash = await sha256(bytes);
        if (observedHash !== part.sourceSha256) {
          throw new Error(`${presentation.fileName}: SHA-256 mismatch.`);
        }
        const source = await parseGlb(loader, bytes);
        source.updateWorldMatrix(true, true);
        const sourceBounds = new THREE.Box3().setFromObject(source);
        if (sourceBounds.isEmpty()) {
          throw new Error(`${presentation.fileName}: source has no render geometry.`);
        }
        const grounded = new THREE.Group();
        grounded.matrixAutoUpdate = false;
        grounded.matrix.copy(itemPreviewGroundingMatrix(sourceBounds));
        grounded.add(source);
        const wrapper = new THREE.Group();
        wrapper.matrixAutoUpdate = false;
        wrapper.matrix.copy(authoredItemMatrix(part));
        wrapper.add(grounded);
        content.add(wrapper);
        return { wrapper, part, presentation, observedHash };
      }),
    );
    loaded.forEach(({ wrapper, part, presentation, observedHash }) => {
      wrapper.updateWorldMatrix(true, true);
      appendPartProof(
        presentation.label,
        observedHash,
        part.translation,
        part.rotationXyzw,
        part.uniformScale,
      );
    });
    const bounds = new THREE.Box3().setFromObject(content);
    if (bounds.isEmpty()) throw new Error("Assembled proof geometry is empty.");
    const aspect = Math.max(1, canvas.clientWidth) / Math.max(1, canvas.clientHeight);
    const frame = itemHorizontalBroadsideCameraFrame(
      { min: bounds.min, max: bounds.max },
      aspect,
    );
    viewHalfHeight = frame.halfHeight;
    controls.target.set(...frame.target);
    camera.up.set(...frame.up);
    camera.position.set(...frame.position);
    camera.zoom = 1.2;
    camera.near = frame.near;
    camera.far = frame.far;
    key.position.set(
      frame.position[0],
      frame.position[1] + frame.halfHeight * 0.45,
      frame.position[2] + frame.halfHeight * 0.45,
    );
    key.target.position.set(...frame.target);
    key.target.updateMatrixWorld();
    resize();
    controls.update();
    status.textContent = "VERIFIED";
    loading.dataset.ready = "true";
  } catch (reason) {
    status.textContent = "FAILED";
    loading.textContent = reason instanceof Error ? reason.message : String(reason);
    loading.dataset.ready = "false";
  }
})();
