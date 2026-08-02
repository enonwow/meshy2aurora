import * as THREE from "three";

export interface ItemAuthoredTransform {
  readonly translation: readonly [number, number, number];
  readonly rotationDegrees: readonly [number, number, number];
  readonly uniformScale: number;
  readonly pivot: readonly [number, number, number];
}

export interface ItemPreviewGroundingBounds {
  readonly min: THREE.Vector3;
  readonly max: THREE.Vector3;
}

export function itemPreviewGroundingMatrix(bounds: ItemPreviewGroundingBounds) {
  return new THREE.Matrix4().makeTranslation(
    -(bounds.min.x + bounds.max.x) / 2,
    -bounds.min.y,
    -(bounds.min.z + bounds.max.z) / 2,
  );
}

export function authoredItemMatrix(transform: ItemAuthoredTransform) {
  const translation = new THREE.Matrix4().makeTranslation(...transform.translation);
  const rotation = new THREE.Matrix4().makeRotationFromEuler(new THREE.Euler(
    ...transform.rotationDegrees.map((value) => value * Math.PI / 180) as [number, number, number],
    "XYZ",
  ));
  const toPivot = new THREE.Matrix4().makeTranslation(...transform.pivot);
  const scale = new THREE.Matrix4().makeScale(
    transform.uniformScale,
    transform.uniformScale,
    transform.uniformScale,
  );
  const fromPivot = new THREE.Matrix4().makeTranslation(
    -transform.pivot[0],
    -transform.pivot[1],
    -transform.pivot[2],
  );
  // Core bakes P*S*P^-1 into geometry and then applies T*P*R*P^-1
  // to the root controller. Uniform scale commutes with rotation, yielding
  // this exact authored Aurora transform: T*P*R*S*P^-1.
  const aurora = translation
    .multiply(toPivot)
    .multiply(rotation)
    .multiply(scale)
    .multiply(fromPivot);
  // Product conversion emits Aurora Z-up coordinates with P(x,y,z)=(x,z,y).
  // The browser displays the source GLB in Three.js Y-up coordinates, so the
  // authored Aurora controller is conjugated by the same self-inverse P.
  const basis = new THREE.Matrix4().set(
    1, 0, 0, 0,
    0, 0, 1, 0,
    0, 1, 0, 0,
    0, 0, 0, 1,
  );
  return basis.clone().multiply(aurora).multiply(basis);
}
