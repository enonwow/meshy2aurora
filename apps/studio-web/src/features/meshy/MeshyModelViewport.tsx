import { useEffect, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer.js";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass.js";
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass.js";
import type { MeshyArtifactProvenance, MeshyRunArtifact } from "./bridge";

interface MeshyModelViewportProps {
  readonly artifact: MeshyRunArtifact;
  readonly label: string;
  readonly onError?: (message: string) => void;
  readonly onClose?: () => void;
}

interface MeshyViewportStatistics {
  readonly topology: "Triangles" | "Mixed";
  readonly faces: number;
  readonly vertices: number;
}

type MeshyMaterialChannel = "baseColor" | "roughness" | "metallic" | "normal";
type MeshyRenderMode = "solid" | "unlit" | "lit";
type MeshyOriginalMaterials = Map<THREE.Mesh, THREE.Material | THREE.Material[]>;
type MeshyMaterialMatching = { readonly target: "metallic" | "roughness"; readonly intensity: number; readonly contrast: number };

const MATERIAL_CHANNELS: readonly { readonly id: MeshyMaterialChannel; readonly label: string }[] = [
  { id: "baseColor", label: "Base color" },
  { id: "roughness", label: "Roughness" },
  { id: "metallic", label: "Metallic" },
  { id: "normal", label: "Normal" },
];

function textureForChannel(material: THREE.Material, channel: MeshyMaterialChannel) {
  if (!(material instanceof THREE.MeshStandardMaterial)) return undefined;
  if (channel === "baseColor") return material.map ?? undefined;
  if (channel === "roughness") return material.roughnessMap ?? undefined;
  if (channel === "metallic") return material.metalnessMap ?? undefined;
  return material.normalMap ?? undefined;
}

export function collectMeshyMaterialChannels(root: THREE.Object3D): readonly MeshyMaterialChannel[] {
  const available = new Set<MeshyMaterialChannel>();
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    (Array.isArray(object.material) ? object.material : [object.material]).forEach((material) => {
      MATERIAL_CHANNELS.forEach(({ id }) => { if (textureForChannel(material, id)) available.add(id); });
    });
  });
  return MATERIAL_CHANNELS.map(({ id }) => id).filter((id) => available.has(id));
}

function captureOriginalMaterials(root: THREE.Object3D): MeshyOriginalMaterials {
  const originals: MeshyOriginalMaterials = new Map();
  root.traverse((object) => { if (object instanceof THREE.Mesh) originals.set(object, object.material); });
  return originals;
}

function disposeInspectorMaterials(root: THREE.Object3D, originals: MeshyOriginalMaterials) {
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const original = originals.get(object);
    if (!original || object.material === original) return;
    (Array.isArray(object.material) ? object.material : [object.material]).forEach((material) => material.dispose());
    object.material = original;
  });
}

function channelMaterial(map: THREE.Texture, channel: MeshyMaterialChannel) {
  if (channel === "baseColor" || channel === "normal") return new THREE.MeshBasicMaterial({ map, toneMapped: false });
  const component = channel === "roughness" ? "g" : "b";
  return new THREE.ShaderMaterial({
    uniforms: { map: { value: map } },
    vertexShader: "varying vec2 vUv; void main() { vUv = uv; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }",
    fragmentShader: `uniform sampler2D map; varying vec2 vUv; void main() { float value = texture2D(map, vUv).${component}; gl_FragColor = vec4(vec3(value), 1.0); }`,
  });
}

/** Meshy's glTF metallic-roughness maps are packed: roughness=G, metalness=B. */
export function materialMapComponent(channel: MeshyMaterialChannel): "rgb" | "g" | "b" {
  if (channel === "roughness") return "g";
  if (channel === "metallic") return "b";
  return "rgb";
}

/** Applies a local viewport material only; it never mutates a Meshy task. */
export function applyMeshyViewportMaterial(root: THREE.Object3D, originals: MeshyOriginalMaterials, renderMode: MeshyRenderMode, channel: MeshyMaterialChannel | undefined, matching?: MeshyMaterialMatching) {
  disposeInspectorMaterials(root, originals);
  originals.forEach((original, mesh) => {
    const makeInspectorMaterial = (material: THREE.Material) => {
      if (renderMode === "lit" && !channel) {
        if (!matching || !(material instanceof THREE.MeshStandardMaterial)) return material;
        const adjusted = material.clone();
        const signedIntensity = (matching.intensity - .5) * 2;
        if (matching.target === "metallic") adjusted.metalness = THREE.MathUtils.clamp(material.metalness + signedIntensity * .5, 0, 1);
        else adjusted.roughness = THREE.MathUtils.clamp(material.roughness - signedIntensity * .5, 0, 1);
        adjusted.color.setRGB(
          THREE.MathUtils.clamp(.5 + (material.color.r - .5) * matching.contrast, 0, 1),
          THREE.MathUtils.clamp(.5 + (material.color.g - .5) * matching.contrast, 0, 1),
          THREE.MathUtils.clamp(.5 + (material.color.b - .5) * matching.contrast, 0, 1),
        );
        return adjusted;
      }
      if (renderMode === "solid") return new THREE.MeshStandardMaterial({ color: 0x8d918a, roughness: .72, metalness: 0 });
      const map = channel ? textureForChannel(material, channel) : undefined;
      if (channel && map) return channelMaterial(map, channel);
      // Meshy's unlit renderer keeps the authored base-color texture while
      // removing scene illumination.  A flat grey fallback made the control
      // look like a different feature rather than the same model view.
      if (renderMode === "unlit") {
        const baseColor = textureForChannel(material, "baseColor");
        return new THREE.MeshBasicMaterial({
          color: material instanceof THREE.MeshStandardMaterial ? material.color : 0xa7aaa4,
          map: baseColor,
          transparent: material.transparent,
          opacity: material.opacity,
          side: material.side,
        });
      }
      return material;
    };
    mesh.material = Array.isArray(original) ? original.map(makeInspectorMaterial) : makeInspectorMaterial(original);
  });
}

function dispose(root: THREE.Object3D) {
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    object.geometry.dispose();
    (Array.isArray(object.material) ? object.material : [object.material]).forEach((material) => {
      Object.values(material).forEach((value) => { if (value instanceof THREE.Texture) value.dispose(); });
      material.dispose();
    });
  });
}

function frameModel(camera: THREE.PerspectiveCamera, controls: OrbitControls, root: THREE.Object3D) {
  const bounds = new THREE.Box3().setFromObject(root);
  const center = bounds.getCenter(new THREE.Vector3());
  const size = bounds.getSize(new THREE.Vector3());
  const largestDimension = Math.max(size.x, size.y, size.z, 0.75);
  const verticalFov = THREE.MathUtils.degToRad(camera.fov);
  // Leave the same breathing room around an asset that the Meshy workspace does.
  // The previous distance filled almost the entire canvas for tall recovered GLBs.
  const distance = largestDimension / (2 * Math.tan(verticalFov / 2)) * 1.4;
  camera.position.copy(center).addScaledVector(new THREE.Vector3(1, 0.68, 1).normalize(), distance);
  camera.near = Math.max(largestDimension / 1000, 0.001);
  camera.far = distance * 100;
  camera.updateProjectionMatrix();
  controls.target.copy(center);
  controls.update();
}

export function collectMeshyViewportStatistics(root: THREE.Object3D): MeshyViewportStatistics {
  let faces = 0;
  let vertices = 0;
  let triangleOnly = true;
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const geometry = object.geometry;
    vertices += geometry.getAttribute("position")?.count ?? 0;
    const indexCount = geometry.index?.count;
    const drawCount = indexCount ?? geometry.getAttribute("position")?.count ?? 0;
    faces += Math.floor(drawCount / 3);
    // BufferGeometry omits drawMode when it uses Three's default: triangles.
    triangleOnly &&= (geometry.drawMode ?? THREE.TrianglesDrawMode) === THREE.TrianglesDrawMode;
  });
  return { topology: triangleOnly ? "Triangles" : "Mixed", faces, vertices };
}

export function applyMeshyViewportWireframe(root: THREE.Object3D, enabled: boolean) {
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    (Array.isArray(object.material) ? object.material : [object.material]).forEach((material) => {
      const wireframeMaterial = material as THREE.Material & { wireframe?: boolean };
      if (wireframeMaterial.wireframe === undefined) return;
      wireframeMaterial.wireframe = enabled;
      wireframeMaterial.needsUpdate = true;
    });
  });
}

/** ReTexture accepts a generated model task, never an arbitrary browser file or signed URL. */
export function retextureInputTaskIdForArtifact(provenance: Pick<MeshyArtifactProvenance, "taskIds">) {
  return provenance.taskIds.REFINE ?? provenance.taskIds.PREVIEW;
}

function drawUvLayout(canvas: HTMLCanvasElement, root: THREE.Object3D) {
  const context = canvas.getContext("2d");
  if (!context) return;
  const size = 512;
  canvas.width = size;
  canvas.height = size;
  context.fillStyle = "#171a17";
  context.fillRect(0, 0, size, size);
  context.strokeStyle = "#cfff47";
  context.globalAlpha = .7;
  context.lineWidth = 1;
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const uv = object.geometry.getAttribute("uv");
    if (!uv) return;
    const index = object.geometry.index;
    const triangleCount = Math.floor((index?.count ?? uv.count) / 3);
    for (let triangle = 0; triangle < triangleCount; triangle += 1) {
      const vertex = (offset: number) => index ? index.getX(triangle * 3 + offset) : triangle * 3 + offset;
      const point = (offset: number) => [uv.getX(vertex(offset)) * size, (1 - uv.getY(vertex(offset))) * size] as const;
      const [x0, y0] = point(0); const [x1, y1] = point(1); const [x2, y2] = point(2);
      context.beginPath(); context.moveTo(x0, y0); context.lineTo(x1, y1); context.lineTo(x2, y2); context.closePath(); context.stroke();
    }
  });
  context.globalAlpha = 1;
}

function showScalarMapChannel(context: CanvasRenderingContext2D, width: number, height: number, channel: MeshyMaterialChannel) {
  if (channel !== "roughness" && channel !== "metallic") return;
  const component = channel === "roughness" ? 1 : 2;
  try {
    const pixels = context.getImageData(0, 0, width, height);
    for (let offset = 0; offset < pixels.data.length; offset += 4) {
      const value = pixels.data[offset + component];
      pixels.data[offset] = value;
      pixels.data[offset + 1] = value;
      pixels.data[offset + 2] = value;
    }
    context.putImageData(pixels, 0, 0);
  } catch {
    // A non-readable texture still remains usable by the 3D shader. Do not
    // replace it with a misleading synthetic thumbnail.
  }
}

function drawUvTexturePreview(canvas: HTMLCanvasElement, root: THREE.Object3D, originals: MeshyOriginalMaterials, channel: MeshyMaterialChannel) {
  const context = canvas.getContext("2d");
  if (!context) return;
  let texture: THREE.Texture | undefined;
  originals.forEach((original) => {
    if (texture) return;
    for (const material of (Array.isArray(original) ? original : [original])) texture ??= textureForChannel(material, channel);
  });
  const image = texture?.image as CanvasImageSource | undefined;
  const source = image as (CanvasImageSource & { width?: number; height?: number }) | undefined;
  const sourceWidth = source?.width;
  const sourceHeight = source?.height;
  if (!image || !sourceWidth || !sourceHeight) {
    drawUvLayout(canvas, root);
    return;
  }
  const size = 512;
  canvas.width = size;
  canvas.height = size;
  context.fillStyle = "#171a17";
  context.fillRect(0, 0, size, size);
  const scale = Math.min(size / sourceWidth, size / sourceHeight);
  const width = sourceWidth * scale;
  const height = sourceHeight * scale;
  context.drawImage(image, (size - width) / 2, (size - height) / 2, width, height);
  showScalarMapChannel(context, size, size, channel);
}

function drawMaterialMapThumbnail(canvas: HTMLCanvasElement, originals: MeshyOriginalMaterials, channel: MeshyMaterialChannel) {
  let texture: THREE.Texture | undefined;
  originals.forEach((original) => {
    if (texture) return;
    for (const material of (Array.isArray(original) ? original : [original])) texture ??= textureForChannel(material, channel);
  });
  const image = texture?.image as CanvasImageSource | undefined;
  const source = image as (CanvasImageSource & { width?: number; height?: number }) | undefined;
  const sourceWidth = source?.width;
  const sourceHeight = source?.height;
  const size = 34;
  canvas.width = size;
  canvas.height = size;
  const context = canvas.getContext("2d");
  if (!context) return;
  context.fillStyle = "#202420";
  context.fillRect(0, 0, size, size);
  if (!image || !sourceWidth || !sourceHeight) return;
  const scale = Math.max(size / sourceWidth, size / sourceHeight);
  const width = sourceWidth * scale;
  const height = sourceHeight * scale;
  context.drawImage(image, (size - width) / 2, (size - height) / 2, width, height);
  showScalarMapChannel(context, size, size, channel);
}

function ViewportIcon({ kind }: { readonly kind: "display" | "wireframe" | "solid" | "unlit" | "lit" | "materialMatching" | "glow" | "environment" | "uv" }) {
  const common = { viewBox: "0 0 24 24", fill: "none", stroke: "currentColor", strokeWidth: 1.6, strokeLinecap: "round" as const, strokeLinejoin: "round" as const, "aria-hidden": true };
  if (kind === "display") return <svg {...common}><rect x="4" y="5" width="16" height="11" rx="1" /><path d="M9 20h6M12 16v4M7 8h10" /></svg>;
  if (kind === "wireframe") return <svg {...common}><circle cx="12" cy="12" r="8.5" opacity=".4" /><path d="M3.5 12h17M12 3.5c2.7 2.1 4 4.9 4 8.5s-1.3 6.4-4 8.5M12 3.5c-2.7 2.1-4 4.9-4 8.5s1.3 6.4 4 8.5" opacity=".75" /></svg>;
  if (kind === "solid") return <svg {...common}><circle cx="12" cy="12" r="8.5" fill="currentColor" opacity=".35" /><circle cx="12" cy="12" r="8.5" /></svg>;
  if (kind === "unlit") return <svg {...common}><circle cx="12" cy="12" r="8.5" /><path d="M7.2 8.4A6.7 6.7 0 0 1 12 6M5.8 12h.1M7.2 15.6A6.7 6.7 0 0 0 12 18" /></svg>;
  if (kind === "lit") return <svg {...common}><circle cx="12" cy="12" r="8.5" /><path d="M12 5.4c1.2 2 2.1 3.1 4.6 4.2-2.5 1.1-3.4 2.2-4.6 4.2-1.2-2-2.1-3.1-4.6-4.2 2.5-1.1 3.4-2.2 4.6-4.2Z" fill="currentColor" opacity=".45" /></svg>;
  if (kind === "materialMatching") return <svg {...common}><path d="m12 7 4 5-4 5-4-5 4-5Z" /><circle cx="12" cy="2" r=".9" fill="currentColor" stroke="none" /><circle cx="22" cy="12" r=".9" fill="currentColor" stroke="none" /><circle cx="12" cy="22" r=".9" fill="currentColor" stroke="none" /><circle cx="2" cy="12" r=".9" fill="currentColor" stroke="none" /><circle cx="19.1" cy="4.9" r=".9" fill="currentColor" stroke="none" /><circle cx="4.9" cy="19.1" r=".9" fill="currentColor" stroke="none" /><circle cx="19.1" cy="19.1" r=".9" fill="currentColor" stroke="none" /><circle cx="4.9" cy="4.9" r=".9" fill="currentColor" stroke="none" /></svg>;
  if (kind === "glow") return <svg {...common}><path d="M12 3v5M12 16v5M3 12h5M16 12h5M5.6 5.6l3.5 3.5M14.9 14.9l3.5 3.5M18.4 5.6l-3.5 3.5M9.1 14.9l-3.5 3.5" /><path d="m12 8 1.5 2.5L16 12l-2.5 1.5L12 16l-1.5-2.5L8 12l2.5-1.5L12 8Z" /></svg>;
  if (kind === "environment") return <svg {...common}><circle cx="12" cy="12" r="4" /><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9 7 7M17 17l2.1 2.1M19.1 4.9 17 7M7 17l-2.1 2.1" /></svg>;
  return <svg {...common}><rect x="4" y="4" width="16" height="16" rx="2" /><path d="m7 16 3-5 2.5 3 2-4 2.5 6M7 8h.01" /></svg>;
}

export function MeshyModelViewport({ artifact, label, onError, onClose }: MeshyModelViewportProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sceneRef = useRef<THREE.Scene | undefined>(undefined);
  const controlsRef = useRef<OrbitControls | undefined>(undefined);
  const cameraRef = useRef<THREE.PerspectiveCamera | undefined>(undefined);
  const glowPassRef = useRef<UnrealBloomPass | undefined>(undefined);
  const environmentLightsRef = useRef<{ key: THREE.DirectionalLight; rim: THREE.DirectionalLight } | undefined>(undefined);
  const rootRef = useRef<THREE.Object3D | undefined>(undefined);
  const gridRef = useRef<THREE.GridHelper | undefined>(undefined);
  const originalMaterialsRef = useRef<MeshyOriginalMaterials | undefined>(undefined);
  const frameModelRef = useRef<(() => void) | undefined>(undefined);
  const [statistics, setStatistics] = useState<MeshyViewportStatistics>();
  const [materialChannels, setMaterialChannels] = useState<readonly MeshyMaterialChannel[]>([]);
  const [materialChannel, setMaterialChannel] = useState<MeshyMaterialChannel | undefined>();
  const [renderMode, setRenderMode] = useState<MeshyRenderMode>("lit");
  const [showSettings, setShowSettings] = useState(false);
  const [showGrid, setShowGrid] = useState(false);
  const [showStatistics, setShowStatistics] = useState(true);
  const [verticalFov, setVerticalFov] = useState(38);
  const [showUvPreview, setShowUvPreview] = useState(false);
  const [uvPreviewChannel, setUvPreviewChannel] = useState<MeshyMaterialChannel>("baseColor");
  const [showGlowSettings, setShowGlowSettings] = useState(false);
  const [glowEnabled, setGlowEnabled] = useState(true);
  // Meshy's default is deliberately subtle.  Keep the local bloom pass at the
  // same low starting point, while the slider still controls the real pass.
  const [glowIntensity, setGlowIntensity] = useState(.1);
  const [showEnvironmentSettings, setShowEnvironmentSettings] = useState(false);
  const [environmentIntensity, setEnvironmentIntensity] = useState(.8);
  const [environmentRotation, setEnvironmentRotation] = useState(0);
  const [environmentBackground, setEnvironmentBackground] = useState(true);
  const [environmentProfile, setEnvironmentProfile] = useState<"indoor" | "outdoor" | "studio" | "soft">("indoor");
  const [autoRotate, setAutoRotate] = useState(true);
  const [wireframe, setWireframe] = useState(false);
  const [showMaterialMatching, setShowMaterialMatching] = useState(false);
  const [materialMatching, setMaterialMatching] = useState<MeshyMaterialMatching>({ target: "metallic", intensity: .5, contrast: 1 });

  useEffect(() => { if (controlsRef.current) controlsRef.current.autoRotate = autoRotate; }, [autoRotate]);
  useEffect(() => { if (gridRef.current) gridRef.current.visible = showGrid; }, [showGrid]);
  useEffect(() => {
    if (!cameraRef.current) return;
    cameraRef.current.fov = verticalFov;
    cameraRef.current.updateProjectionMatrix();
    frameModelRef.current?.();
  }, [verticalFov]);
  useEffect(() => {
    if (glowPassRef.current) {
      glowPassRef.current.enabled = glowEnabled;
      glowPassRef.current.strength = glowIntensity;
    }
  }, [glowEnabled, glowIntensity]);
  useEffect(() => {
    const scene = sceneRef.current;
    const lights = environmentLightsRef.current;
    const profile = environmentProfile === "outdoor"
      ? { background: 0x182127, key: 0xc7e7ff, rim: 0x83b7d8 }
      : environmentProfile === "studio"
        ? { background: 0x242326, key: 0xf7f0e1, rim: 0xd1c5ad }
        : environmentProfile === "soft"
          ? { background: 0x24231f, key: 0xffead1, rim: 0xd9cfa2 }
          : { background: 0x1a1c1a, key: 0xfff5df, rim: 0xd8ff91 };
    if (scene) scene.background = environmentBackground ? new THREE.Color(profile.background) : null;
    if (!lights) return;
    const rotation = THREE.MathUtils.degToRad(environmentRotation);
    lights.key.position.set(4 * Math.cos(rotation), 6, 4 * Math.sin(rotation));
    lights.rim.position.set(-4 * Math.cos(rotation), 2, -4 * Math.sin(rotation));
    lights.key.intensity = 2.4 * environmentIntensity;
    lights.rim.intensity = 1.15 * environmentIntensity;
    lights.key.color.setHex(profile.key);
    lights.rim.color.setHex(profile.rim);
  }, [environmentBackground, environmentIntensity, environmentProfile, environmentRotation]);
  useEffect(() => { if (rootRef.current) applyMeshyViewportWireframe(rootRef.current, wireframe); }, [wireframe]);
  useEffect(() => {
    if (!rootRef.current || !originalMaterialsRef.current) return;
    applyMeshyViewportMaterial(rootRef.current, originalMaterialsRef.current, renderMode, materialChannel, showMaterialMatching ? materialMatching : undefined);
    applyMeshyViewportWireframe(rootRef.current, wireframe);
  }, [materialChannel, materialMatching, renderMode, showMaterialMatching, wireframe]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let disposed = false;
    let frame = 0;
    let root: THREE.Object3D | undefined;
    setStatistics(undefined);
    setMaterialChannels([]);
    setMaterialChannel(undefined);
    setRenderMode("lit");
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a1c1a);
    sceneRef.current = scene;
    const camera = new THREE.PerspectiveCamera(38, 1, 0.01, 10_000);
    cameraRef.current = camera;
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.setClearAlpha(0);
    const composer = new EffectComposer(renderer);
    const bloomPass = new UnrealBloomPass(new THREE.Vector2(1, 1), glowIntensity, .25, .78);
    bloomPass.enabled = glowEnabled;
    composer.addPass(new RenderPass(scene, camera));
    composer.addPass(bloomPass);
    glowPassRef.current = bloomPass;
    const controls = new OrbitControls(camera, canvas);
    controlsRef.current = controls;
    controls.autoRotate = autoRotate;
    controls.enableDamping = true;
    controls.enablePan = false;
    controls.minDistance = 0.15;
    scene.add(new THREE.HemisphereLight(0xffffff, 0x2c302b, 2.2));
    const key = new THREE.DirectionalLight(0xfff5df, 2.4);
    key.position.set(4, 6, 4);
    scene.add(key);
    const rim = new THREE.DirectionalLight(0xd8ff91, 1.15);
    rim.position.set(-4, 2, -4);
    scene.add(rim);
    environmentLightsRef.current = { key, rim };

    const resize = () => {
      const width = Math.max(canvas.clientWidth, 1);
      const height = Math.max(canvas.clientHeight, 1);
      renderer.setSize(width, height, false);
      composer.setSize(width, height);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      if (root) frameModel(camera, controls, root);
    };
    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    resize();

    const animate = () => {
      if (disposed) return;
      controls.update();
      composer.render();
      frame = requestAnimationFrame(animate);
    };
    void artifact.file.arrayBuffer().then((bytes) => new Promise<THREE.Object3D>((resolve, reject) => {
      const manager = new THREE.LoadingManager();
      manager.setURLModifier((url) => {
        if (url.startsWith("blob:") || url.startsWith("data:")) return url;
        throw new Error("Meshy viewport forbids external GLB resource URLs.");
      });
      new GLTFLoader(manager).parse(bytes, "", ({ scene: loaded }) => resolve(loaded), reject);
    })).then((loaded) => {
      if (disposed) return dispose(loaded);
      root = loaded;
      rootRef.current = root;
      const originals = captureOriginalMaterials(root);
      originalMaterialsRef.current = originals;
      scene.add(root);
      const bounds = new THREE.Box3().setFromObject(root);
      const size = bounds.getSize(new THREE.Vector3());
      const grid = new THREE.GridHelper(Math.max(size.x, size.z, 1) * 1.8, 18, 0x67715f, 0x303630);
      grid.position.y = bounds.min.y;
      grid.visible = showGrid;
      scene.add(grid);
      gridRef.current = grid;
      applyMeshyViewportWireframe(root, wireframe);
      frameModel(camera, controls, root);
      frameModelRef.current = () => frameModel(camera, controls, root!);
      setStatistics(collectMeshyViewportStatistics(root));
      const channels = collectMeshyMaterialChannels(root);
      setMaterialChannels(channels);
      setMaterialChannel(channels.includes("baseColor") ? "baseColor" : undefined);
    }).catch((error: unknown) => onError?.(error instanceof Error ? error.message : `Could not load ${label}.`));
    animate();

    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
      observer.disconnect();
      controls.dispose();
      if (controlsRef.current === controls) controlsRef.current = undefined;
      if (rootRef.current === root) rootRef.current = undefined;
      if (gridRef.current) { gridRef.current.geometry.dispose(); (gridRef.current.material as THREE.Material).dispose(); gridRef.current = undefined; }
      if (cameraRef.current === camera) cameraRef.current = undefined;
      if (glowPassRef.current === bloomPass) glowPassRef.current = undefined;
      if (environmentLightsRef.current?.key === key) environmentLightsRef.current = undefined;
      if (root && originalMaterialsRef.current) disposeInspectorMaterials(root, originalMaterialsRef.current);
      if (root && originalMaterialsRef.current) originalMaterialsRef.current = undefined;
      frameModelRef.current = undefined;
      if (sceneRef.current === scene) sceneRef.current = undefined;
      if (root) dispose(root);
      renderer.dispose();
      composer.dispose();
    };
  }, [artifact.file, label, onError]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => { if (event.key === "Escape") { setShowSettings(false); setShowGlowSettings(false); setShowEnvironmentSettings(false); setShowMaterialMatching(false); setShowUvPreview(false); } };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  return <div className="meshy-model-viewport" aria-label={`Meshy model viewport: ${label}`}>
    <canvas ref={canvasRef} />
    <div className="meshy-model-viewport__toolbar" role="toolbar" aria-label="Meshy viewport controls">
      <div className="meshy-model-viewport__toolbar-group" aria-label="Display controls">
        <button type="button" aria-label="Open display settings" aria-expanded={showSettings} onClick={() => { setShowSettings((current) => !current); setShowGlowSettings(false); setShowEnvironmentSettings(false); setShowMaterialMatching(false); }}><ViewportIcon kind="display" /></button>
        <button type="button" className="meshy-model-viewport__wireframe" aria-label="Toggle model wireframe" aria-pressed={wireframe} onClick={() => setWireframe((current) => !current)}><ViewportIcon kind="wireframe" /></button>
        {(["solid", "unlit", "lit"] as const).map((mode) => <button key={mode} type="button" className={`meshy-model-viewport__render-mode meshy-model-viewport__render-mode--${mode}`} aria-label={`Use ${mode} render mode`} aria-pressed={renderMode === mode} onClick={() => { setRenderMode(mode); setMaterialChannel(undefined); setShowMaterialMatching(false); }}><ViewportIcon kind={mode} /></button>)}
        <span className="meshy-model-viewport__toolbar-divider" aria-hidden="true" />
        <button type="button" className="meshy-model-viewport__material-matching" aria-label="Open material matching" aria-expanded={showMaterialMatching} onClick={() => { setShowMaterialMatching((current) => !current); setRenderMode("lit"); setMaterialChannel(undefined); setShowSettings(false); setShowGlowSettings(false); setShowEnvironmentSettings(false); }}><ViewportIcon kind="materialMatching" /></button>
        <button type="button" className="meshy-model-viewport__glow" aria-label="Open glow settings" aria-expanded={showGlowSettings} onClick={() => { setShowGlowSettings((current) => !current); setShowSettings(false); setShowEnvironmentSettings(false); setShowMaterialMatching(false); }}><ViewportIcon kind="glow" /></button>
        <button type="button" className="meshy-model-viewport__environment" aria-label="Open environment settings" aria-expanded={showEnvironmentSettings} onClick={() => { setShowEnvironmentSettings((current) => !current); setShowSettings(false); setShowGlowSettings(false); setShowMaterialMatching(false); }}><ViewportIcon kind="environment" /></button>
        {showSettings ? <div className="meshy-model-viewport__settings" role="dialog" aria-label="Display settings"><strong>Ustawienia wyświetlania</strong><label>Siatka<input type="checkbox" checked={showGrid} onChange={(event) => setShowGrid(event.target.checked)} /></label><label>Statystyka<input type="checkbox" checked={showStatistics} onChange={(event) => setShowStatistics(event.target.checked)} /></label><label>Automatyczne obracanie<input type="checkbox" checked={autoRotate} onChange={(event) => setAutoRotate(event.target.checked)} /></label><label>Pionowe FOV<input type="range" min="20" max="65" value={verticalFov} onChange={(event) => setVerticalFov(Number(event.target.value))} /></label><button type="button" aria-label="Reset vertical FOV" disabled={verticalFov === 38} onClick={() => { setVerticalFov(38); frameModelRef.current?.(); }}>↻</button></div> : null}
        {showMaterialMatching ? <div className="meshy-model-viewport__settings meshy-model-viewport__settings--material-matching" role="dialog" aria-label="Material matching"><strong>Dopasowanie materiału</strong><div className="meshy-model-viewport__matching-target" role="radiogroup" aria-label="Material matching target"><label><input type="radio" name="meshy-material-matching" checked={materialMatching.target === "metallic"} onChange={() => setMaterialMatching((current) => ({ ...current, target: "metallic" }))} />Metaliczność</label><label><input type="radio" name="meshy-material-matching" checked={materialMatching.target === "roughness"} onChange={() => setMaterialMatching((current) => ({ ...current, target: "roughness" }))} />Szorstkość</label></div><label>Intensywność<input type="range" min="0" max="1" step=".01" value={materialMatching.intensity} onChange={(event) => setMaterialMatching((current) => ({ ...current, intensity: Number(event.target.value) }))} /></label><label>Kontrast<input type="range" min="0" max="2" step=".01" value={materialMatching.contrast} onChange={(event) => setMaterialMatching((current) => ({ ...current, contrast: Number(event.target.value) }))} /></label></div> : null}
        {showGlowSettings ? <div className="meshy-model-viewport__settings meshy-model-viewport__settings--glow" role="dialog" aria-label="Glow settings"><strong>Ustawienia poświaty</strong><label>Włącz bloom<input type="checkbox" checked={glowEnabled} onChange={(event) => setGlowEnabled(event.target.checked)} /></label><label>Natężenie<input type="range" min="0" max="2" step=".05" value={glowIntensity} onChange={(event) => setGlowIntensity(Number(event.target.value))} /></label></div> : null}
        {showEnvironmentSettings ? <div className="meshy-model-viewport__settings meshy-model-viewport__settings--environment" role="dialog" aria-label="Environment settings"><strong>Ustawienia środowiska</strong><label>HDRI<select aria-label="HDRI" value={environmentProfile} onChange={(event) => setEnvironmentProfile(event.target.value as typeof environmentProfile)}><option value="indoor">Oświetlenie wewnętrzne</option><option value="outdoor">Oświetlenie zewnętrzne</option><option value="studio">Oświetlenie studyjne</option><option value="soft">Miękkie oświetlenie</option></select></label><label>Wytrzymałość<input type="range" min=".2" max="2" step=".1" value={environmentIntensity} onChange={(event) => setEnvironmentIntensity(Number(event.target.value))} /></label><label>Obrót<input type="range" min="0" max="360" value={environmentRotation} onChange={(event) => setEnvironmentRotation(Number(event.target.value))} disabled={autoRotate} /></label><small>{autoRotate ? "Wyłącz automatyczne obracanie, aby dostosować obrót." : "Lokalny preset światła — nie pobiera zewnętrznego HDRI."}</small><label>Automatyczne obracanie<input type="checkbox" checked={autoRotate} onChange={(event) => setAutoRotate(event.target.checked)} /></label><label>Tło<input type="checkbox" checked={environmentBackground} onChange={(event) => setEnvironmentBackground(event.target.checked)} /></label></div> : null}
      </div>
      <div className="meshy-model-viewport__toolbar-group meshy-model-viewport__toolbar-group--maps" aria-label="Local material maps">
        {MATERIAL_CHANNELS.map(({ id, label: channelLabel }) => <button key={id} type="button" className={`meshy-model-viewport__material meshy-model-viewport__material--${id}`} aria-label={`View ${channelLabel.toLowerCase()} map`} aria-pressed={materialChannel === id} onClick={() => { setRenderMode("lit"); setMaterialChannel(id); setShowMaterialMatching(false); }} disabled={!materialChannels.includes(id)}>{originalMaterialsRef.current ? <canvas aria-hidden="true" ref={(canvas) => { if (canvas) drawMaterialMapThumbnail(canvas, originalMaterialsRef.current!, id); }} /> : <i aria-hidden="true" />}</button>)}
        <button type="button" aria-label="Open UV texture preview" onClick={() => { setUvPreviewChannel(materialChannel ?? "baseColor"); setShowUvPreview(true); }}><ViewportIcon kind="uv" /></button>
      </div>
    </div>
    {showStatistics && statistics ? <dl className="meshy-model-viewport__statistics"><div><dt>Topologia</dt><dd>{statistics.topology === "Triangles" ? "Trójkąty" : "Mieszana"}</dd></div><div><dt>Twarze</dt><dd>{statistics.faces.toLocaleString()}</dd></div><div><dt>Wierzchołki</dt><dd>{statistics.vertices.toLocaleString()}</dd></div></dl> : null}
    {showUvPreview ? <div className="meshy-model-viewport__uv-backdrop"><div className="meshy-model-viewport__uv-preview" role="dialog" aria-label="UV texture preview"><div className="meshy-model-viewport__uv-header"><span aria-hidden="true">⠿</span><nav aria-label="UV texture channels">{MATERIAL_CHANNELS.filter(({ id }) => materialChannels.includes(id)).map(({ id, label: channelLabel }) => <button key={id} type="button" aria-label={`Preview ${channelLabel} UV texture`} aria-pressed={uvPreviewChannel === id} onClick={() => setUvPreviewChannel(id)}>{originalMaterialsRef.current ? <canvas aria-hidden="true" ref={(canvas) => { if (canvas) drawMaterialMapThumbnail(canvas, originalMaterialsRef.current!, id); }} /> : <i aria-hidden="true" />}</button>)}</nav><button type="button" aria-label="Close UV texture preview" onClick={() => setShowUvPreview(false)}>×</button></div><canvas ref={(canvas) => { if (canvas && rootRef.current && originalMaterialsRef.current) drawUvTexturePreview(canvas, rootRef.current, originalMaterialsRef.current, uvPreviewChannel); }} /></div></div> : null}
    <p>Drag to orbit · scroll to zoom</p>
  </div>;
}
