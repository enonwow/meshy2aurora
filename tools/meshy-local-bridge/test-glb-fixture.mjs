export function createTestGlb({
  marker = 0,
  withSkin = true,
  bindSkin = withSkin,
  withAnimation = true,
  animateJoint = true,
  renderInScene = true,
  corruptIndex = false,
  corruptJoint = false,
} = {}) {
  const positions = new Float32Array([
    marker / 1_000, 0, 0,
    1, 0, 0,
    0, 1, 0,
  ]);
  const indices = new Uint16Array([0, 1, corruptIndex ? 99 : 2]);
  const times = new Float32Array([0, 1]);
  const translations = new Float32Array([
    0, 0, 0,
    0.25, 0, 0,
  ]);
  const inverseBindMatrices = new Float32Array([
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1,
    1, 0, 0, 0,
    0, 1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1,
  ]);
  const joints = new Uint16Array([
    0, 1, 0, 0,
    corruptJoint ? 99 : 0, 1, 0, 0,
    0, 1, 0, 0,
  ]);
  const weights = new Float32Array([
    1, 0, 0, 0,
    0.5, 0.5, 0, 0,
    0, 1, 0, 0,
  ]);

  const binary = new Uint8Array(280);
  binary.set(new Uint8Array(positions.buffer), 0);
  binary.set(new Uint8Array(indices.buffer), 36);
  binary.set(new Uint8Array(times.buffer), 44);
  binary.set(new Uint8Array(translations.buffer), 52);
  binary.set(new Uint8Array(inverseBindMatrices.buffer), 76);
  binary.set(new Uint8Array(joints.buffer), 204);
  binary.set(new Uint8Array(weights.buffer), 228);
  binary[276] = marker;

  const document = {
    asset: { version: "2.0" },
    buffers: [{ byteLength: binary.byteLength }],
    bufferViews: [
      { buffer: 0, byteOffset: 0, byteLength: 36 },
      { buffer: 0, byteOffset: 36, byteLength: 6 },
      { buffer: 0, byteOffset: 44, byteLength: 8 },
      { buffer: 0, byteOffset: 52, byteLength: 24 },
      { buffer: 0, byteOffset: 76, byteLength: 128 },
      { buffer: 0, byteOffset: 204, byteLength: 24 },
      { buffer: 0, byteOffset: 228, byteLength: 48 },
    ],
    accessors: [
      { bufferView: 0, componentType: 5126, count: 3, type: "VEC3" },
      { bufferView: 1, componentType: 5123, count: 3, type: "SCALAR" },
      { bufferView: 2, componentType: 5126, count: 2, type: "SCALAR" },
      { bufferView: 3, componentType: 5126, count: 2, type: "VEC3" },
      { bufferView: 4, componentType: 5126, count: 2, type: "MAT4" },
      { bufferView: 5, componentType: 5123, count: 3, type: "VEC4" },
      { bufferView: 6, componentType: 5126, count: 3, type: "VEC4" },
    ],
    meshes: [{
      primitives: [{
        attributes: {
          POSITION: 0,
          ...(bindSkin ? { JOINTS_0: 5, WEIGHTS_0: 6 } : {}),
        },
        indices: 1,
        mode: 4,
      }],
    }],
    nodes: [
      {
        name: "root",
        mesh: 0,
        ...(bindSkin ? { skin: 0 } : {}),
        children: [1],
      },
      { name: "spine" },
      { name: "non-joint-animation-target" },
    ],
    scenes: [{ nodes: renderInScene ? [0, 2] : [2] }],
    scene: 0,
    ...(withSkin ? {
      skins: [{
        joints: [0, 1],
        skeleton: 0,
        inverseBindMatrices: 4,
      }],
    } : {}),
    ...(withAnimation ? {
      animations: [{
        name: "attack",
        samplers: [{ input: 2, output: 3, interpolation: "LINEAR" }],
        channels: [{
          sampler: 0,
          target: {
            node: animateJoint ? 0 : 2,
            path: "translation",
          },
        }],
      }],
    } : {}),
  };
  const encoded = new TextEncoder().encode(JSON.stringify(document));
  const jsonLength = Math.ceil(encoded.byteLength / 4) * 4;
  const totalLength = 12 + 8 + jsonLength + 8 + binary.byteLength;
  const bytes = new Uint8Array(totalLength);
  const data = new DataView(bytes.buffer);
  data.setUint32(0, 0x46546c67, true);
  data.setUint32(4, 2, true);
  data.setUint32(8, totalLength, true);
  data.setUint32(12, jsonLength, true);
  data.setUint32(16, 0x4e4f534a, true);
  bytes.fill(0x20, 20, 20 + jsonLength);
  bytes.set(encoded, 20);
  const binaryHeader = 20 + jsonLength;
  data.setUint32(binaryHeader, binary.byteLength, true);
  data.setUint32(binaryHeader + 4, 0x004e4942, true);
  bytes.set(binary, binaryHeader + 8);
  return bytes;
}
