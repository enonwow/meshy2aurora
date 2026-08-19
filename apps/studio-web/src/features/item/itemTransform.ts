import * as THREE from "three";

export interface ItemAuthoredTransform {
  readonly translation: readonly [number, number, number];
  readonly rotationXyzw: readonly [number, number, number, number];
  readonly uniformScale: number;
  readonly pivot: readonly [number, number, number];
  readonly targetSpaceScaleXyz?: readonly [number, number, number];
}

export interface ItemPreviewGroundingBounds {
  readonly min: THREE.Vector3;
  readonly max: THREE.Vector3;
}

export interface ItemPropertiesCameraFrame {
  readonly position: readonly [number, number, number];
  readonly target: readonly [number, number, number];
  readonly up: readonly [number, number, number];
  readonly halfHeight: number;
  readonly near: number;
  readonly far: number;
}

export function itemPreviewGroundingMatrix(bounds: ItemPreviewGroundingBounds) {
  return new THREE.Matrix4().makeTranslation(
    -(bounds.min.x + bounds.max.x) / 2,
    -bounds.min.y,
    -(bounds.min.z + bounds.max.z) / 2,
  );
}

export function itemPropertiesCameraFrame(
  bounds: ItemPreviewGroundingBounds,
  aspect: number,
): ItemPropertiesCameraFrame {
  const safeAspect = Number.isFinite(aspect) && aspect > 0 ? aspect : 1;
  const center = bounds.min.clone().add(bounds.max).multiplyScalar(0.5);
  const size = bounds.max.clone().sub(bounds.min);

  // After P(x,y,z)=(x,z,y), Aurora Item coordinates become:
  // Three X = model depth, Three Y = model width, Three Z = axial length.
  // Aurora Item Properties looks straight down the depth axis and presents
  // axial length as screen-up. The orthographic frame therefore fits Z
  // vertically and Y horizontally without changing any authored transform.
  const axialHalfHeight = size.z / 2;
  const widthHalfHeight = size.y / (2 * safeAspect);
  const halfHeight = Math.max(axialHalfHeight, widthHalfHeight, 0.125) * 1.18;
  const depth = Math.max(size.x, halfHeight * 2, 0.25);
  const distance = Math.max(depth * 3, 1);

  return {
    position: [center.x + distance, center.y, center.z],
    target: center.toArray() as [number, number, number],
    up: [0, 0, 1],
    halfHeight,
    near: Math.max(distance - depth * 2, 0.001),
    far: Math.max(distance + depth * 2, 100),
  };
}

export function itemHorizontalBroadsideCameraFrame(
  bounds: ItemPreviewGroundingBounds,
  aspect: number,
): ItemPropertiesCameraFrame {
  const safeAspect = Number.isFinite(aspect) && aspect > 0 ? aspect : 1;
  const center = bounds.min.clone().add(bounds.max).multiplyScalar(0.5);
  const size = bounds.max.clone().sub(bounds.min);

  // After P(x,y,z)=(x,z,y), Three Z is Aurora axial +Y and Three Y is
  // Aurora width +Z. Keep the same broadside depth view as Item Properties,
  // but roll the proof camera clockwise so Bottom -> Middle -> Top reads
  // left-to-right without changing any authored or emitted model transform.
  const widthHalfHeight = size.y / 2;
  const axialHalfHeight = size.z / (2 * safeAspect);
  const halfHeight = Math.max(widthHalfHeight, axialHalfHeight, 0.125) * 1.18;
  const depth = Math.max(size.x, halfHeight * 2, 0.25);
  const distance = Math.max(depth * 3, 1);

  return {
    position: [center.x + distance, center.y, center.z],
    target: center.toArray() as [number, number, number],
    up: [0, -1, 0],
    halfHeight,
    near: Math.max(distance - depth * 2, 0.001),
    far: Math.max(distance + depth * 2, 100),
  };
}

export function authoredItemMatrix(transform: ItemAuthoredTransform) {
  const translation = new THREE.Matrix4().makeTranslation(...transform.translation);
  const rotation = new THREE.Matrix4().makeRotationFromQuaternion(new THREE.Quaternion(
    ...transform.rotationXyzw,
  ));
  const toPivot = new THREE.Matrix4().makeTranslation(...transform.pivot);
  const uniformScale = new THREE.Matrix4().makeScale(
    transform.uniformScale,
    transform.uniformScale,
    transform.uniformScale,
  );
  const targetSpaceScale = new THREE.Matrix4().makeScale(
    ...(transform.targetSpaceScaleXyz ?? [1, 1, 1]),
  );
  const fromPivot = new THREE.Matrix4().makeTranslation(
    -transform.pivot[0],
    -transform.pivot[1],
    -transform.pivot[2],
  );
  // Core bakes P*S*P^-1, then rotates geometry, applies the target-space
  // transverse scale and finally keeps only the rigid controller translation.
  const bakedUniformScale = toPivot.clone().multiply(uniformScale).multiply(fromPivot);
  const rigidController = translation.clone()
    .multiply(toPivot)
    .multiply(rotation)
    .multiply(fromPivot);
  const rigidTranslation = new THREE.Vector3().setFromMatrixPosition(rigidController);
  const rigidRotation = new THREE.Matrix4().extractRotation(rigidController);
  const aurora = new THREE.Matrix4()
    .makeTranslation(...rigidTranslation.toArray())
    .multiply(targetSpaceScale)
    .multiply(rigidRotation)
    .multiply(bakedUniformScale);
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
