import { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { ModelPartRef } from "./types";

interface SceneViewportProps {
  provenance: "SOURCE" | "AURORA IR" | "READBACK";
  detail: string;
  buildRoot: () => Promise<THREE.Object3D>;
  dependency: unknown;
  onSelectPart?: (part?: ModelPartRef) => void;
  onError?: (message: string) => void;
}

function disposeMaterial(material: THREE.Material) {
  Object.values(material).forEach((value) => {
    if (value instanceof THREE.Texture) value.dispose();
  });
  material.dispose();
}

export function disposeObjectResources(root: THREE.Object3D) {
  root.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    object.geometry.dispose();
    const materials = Array.isArray(object.material) ? object.material : [object.material];
    materials.forEach(disposeMaterial);
    if (object instanceof THREE.SkinnedMesh) object.skeleton.dispose();
  });
}

function modelPart(object?: THREE.Object3D | null): ModelPartRef | undefined {
  let candidate = object;
  while (candidate) {
    const part = candidate.userData.modelPart as ModelPartRef | undefined;
    if (part) return part;
    candidate = candidate.parent;
  }
  return undefined;
}

function fitCamera(camera: THREE.PerspectiveCamera, controls: OrbitControls, root: THREE.Object3D) {
  const bounds = new THREE.Box3().setFromObject(root);
  const center = bounds.isEmpty() ? new THREE.Vector3() : bounds.getCenter(new THREE.Vector3());
  const size = bounds.isEmpty() ? new THREE.Vector3(1, 1, 1) : bounds.getSize(new THREE.Vector3());
  const distance = Math.max(size.length(), 1);
  camera.position.copy(center).add(new THREE.Vector3(distance, distance * 0.7, distance));
  camera.near = Math.max(distance / 1000, 0.001);
  camera.far = Math.max(distance * 100, 100);
  camera.updateProjectionMatrix();
  controls.target.copy(center);
  controls.update();
}

export function SceneViewport({ provenance, detail, buildRoot, dependency, onSelectPart, onError }: SceneViewportProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let stopped = false;
    let root: THREE.Object3D | undefined;
    let frame = 0;
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0a121a);
    const camera = new THREE.PerspectiveCamera(45, 1, 0.01, 10_000);
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    const controls = new OrbitControls(camera, canvas);
    controls.enableDamping = true;
    scene.add(new THREE.HemisphereLight(0xffffff, 0x203044, 2));
    const key = new THREE.DirectionalLight(0xffffff, 2);
    key.position.set(4, 6, 3);
    scene.add(key, new THREE.GridHelper(10, 20, 0x38536b, 0x1e3040));

    const resize = () => {
      const width = Math.max(canvas.clientWidth, 1);
      const height = Math.max(canvas.clientHeight, 1);
      renderer.setSize(width, height, false);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    resize();

    const raycaster = new THREE.Raycaster();
    const pointer = new THREE.Vector2();
    const select = (event: PointerEvent) => {
      const rect = canvas.getBoundingClientRect();
      pointer.set(
        ((event.clientX - rect.left) / rect.width) * 2 - 1,
        -((event.clientY - rect.top) / rect.height) * 2 + 1,
      );
      raycaster.setFromCamera(pointer, camera);
      onSelectPart?.(modelPart(raycaster.intersectObjects(scene.children, true)[0]?.object));
    };
    canvas.addEventListener("pointerdown", select);

    void buildRoot()
      .then((value) => {
        if (stopped) return disposeObjectResources(value);
        root = value;
        scene.add(value);
        fitCamera(camera, controls, value);
      })
      .catch((error: unknown) => {
        if (!stopped) onError?.(error instanceof Error ? error.message : String(error));
      });

    const render = () => {
      controls.update();
      renderer.render(scene, camera);
      frame = requestAnimationFrame(render);
    };
    render();

    return () => {
      stopped = true;
      cancelAnimationFrame(frame);
      canvas.removeEventListener("pointerdown", select);
      observer.disconnect();
      controls.dispose();
      if (root) disposeObjectResources(root);
      renderer.dispose();
    };
  }, [buildRoot, dependency, onSelectPart, onError]);

  return (
    <section className="viewport" aria-label={`${provenance} model viewport`}>
      <header><strong>{provenance}</strong><span>{detail}</span></header>
      <canvas ref={canvasRef} />
    </section>
  );
}
