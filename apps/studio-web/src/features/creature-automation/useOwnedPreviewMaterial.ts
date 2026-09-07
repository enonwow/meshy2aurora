import { useEffect, useState } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
import type { AuroraReadbackMaterialResolver } from "../preview/AuroraReadbackViewport";
import type { SourcePreviewInput } from "../preview/types";

/** The Creature product boundary currently accepts one owned material.
 * Show its diffuse texture on the actual MDL readback, never substitute GLB
 * geometry for the generated skin. This display is not native material proof.
 */
export function useOwnedPreviewMaterial(source?: SourcePreviewInput) {
  const [resolver, setResolver] = useState<AuroraReadbackMaterialResolver>();
  const [error, setError] = useState<string>();
  useEffect(() => {
    setResolver(undefined); setError(undefined);
    if (!source) return;
    let cancelled = false;
    const textures = new Set<THREE.Texture>();
    const materials = new Set<THREE.Material>();
    const cleanup = () => { textures.forEach(t => t.dispose()); materials.forEach(m => m.dispose()); };
    const manager = new THREE.LoadingManager();
    manager.setURLModifier(url => {
      if (url.startsWith("blob:") || url.startsWith("data:")) return url;
      throw new Error("Owned Creature preview forbids external GLB resources");
    });
    void source.file.arrayBuffer().then(bytes => new GLTFLoader(manager).parseAsync(bytes, "")).then(gltf => {
      const diffuse = new Set<THREE.MeshStandardMaterial>();
      gltf.scene.traverse(object => {
        if (!(object instanceof THREE.Mesh)) return;
        object.geometry.dispose();
        for (const material of Array.isArray(object.material) ? object.material : [object.material]) {
          materials.add(material);
          for (const value of Object.values(material)) if (value instanceof THREE.Texture) textures.add(value);
          if (material instanceof THREE.MeshStandardMaterial) diffuse.add(material);
        }
      });
      if (cancelled) { cleanup(); return; }
      if (diffuse.size !== 1) throw new Error("Podgląd tekstury Creature wymaga jednego materiału źródłowego.");
      const owned = [...diffuse][0];
      setResolver(() => ((_mesh, _node, selected) => {
        const map = owned.map?.clone() ?? null;
        if (map) {
          // Creature writes Aurora UVs with V = 1 - glTF V. Apply the inverse
          // in texture coordinates; ImageBitmap ignores Texture.flipY.
          owned.map!.updateMatrix();
          map.matrixAutoUpdate = false;
          map.matrix.copy(owned.map!.matrix).multiply(new THREE.Matrix3().set(1, 0, 0, 0, -1, 1, 0, 0, 1));
          map.needsUpdate = true; textures.add(map);
        }
        return new THREE.MeshStandardMaterial({
          map, color: selected ? 0xffdcaa : owned.color,
          roughness: 1, metalness: 0, side: THREE.DoubleSide,
        });
      }) satisfies AuroraReadbackMaterialResolver);
    }).catch(reason => { if (!cancelled) setError(String(reason)); cleanup(); });
    return () => { cancelled = true; cleanup(); };
  }, [source?.file, source?.sourceSha256]);
  return { resolver, error };
}
