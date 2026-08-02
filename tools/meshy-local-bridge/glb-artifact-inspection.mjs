import { createHash } from "node:crypto";

const GLB_MAGIC = 0x46546c67;
const GLB_VERSION = 2;
const GLB_JSON_CHUNK = 0x4e4f534a;
const GLB_BIN_CHUNK = 0x004e4942;
const TRIANGLES_MODE = 4;

const COMPONENT_BYTES = new Map([
  [5120, 1],
  [5121, 1],
  [5122, 2],
  [5123, 2],
  [5125, 4],
  [5126, 4],
]);
const TYPE_COMPONENTS = new Map([
  ["SCALAR", 1],
  ["VEC2", 2],
  ["VEC3", 3],
  ["VEC4", 4],
  ["MAT2", 4],
  ["MAT3", 9],
  ["MAT4", 16],
]);

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}

function readU32(bytes, offset) {
  invariant(
    Number.isSafeInteger(offset) && offset >= 0 && offset + 4 <= bytes.byteLength,
    "GLB integer field is truncated.",
  );
  return new DataView(
    bytes.buffer,
    bytes.byteOffset + offset,
    4,
  ).getUint32(0, true);
}

function requireArray(value, path, { allowEmpty = true } = {}) {
  invariant(Array.isArray(value), `${path} must be an array.`);
  invariant(allowEmpty || value.length > 0, `${path} must not be empty.`);
  return value;
}

function optionalArray(value, path) {
  return value === undefined ? [] : requireArray(value, path);
}

function safeIndex(value, length, path) {
  invariant(
    Number.isSafeInteger(value) && value >= 0 && value < length,
    `${path} is not a valid array index.`,
  );
  return value;
}

function parseGlb(bytes) {
  invariant(bytes.byteLength >= 20, "GLB header or JSON chunk is truncated.");
  invariant(readU32(bytes, 0) === GLB_MAGIC, "GLB magic is invalid.");
  invariant(readU32(bytes, 4) === GLB_VERSION, "Only binary glTF 2.0 is supported.");
  invariant(
    readU32(bytes, 8) === bytes.byteLength,
    "GLB declared length does not match the downloaded artifact.",
  );

  let offset = 12;
  let jsonBytes;
  let binaryBytes;
  let chunkIndex = 0;
  while (offset < bytes.byteLength) {
    invariant(offset + 8 <= bytes.byteLength, "GLB chunk header is truncated.");
    const chunkLength = readU32(bytes, offset);
    const chunkType = readU32(bytes, offset + 4);
    invariant(chunkLength % 4 === 0, "GLB chunk length is not four-byte aligned.");
    const chunkStart = offset + 8;
    const chunkEnd = chunkStart + chunkLength;
    invariant(chunkEnd <= bytes.byteLength, "GLB chunk payload is truncated.");
    const chunk = bytes.subarray(chunkStart, chunkEnd);
    if (chunkIndex === 0) {
      invariant(chunkType === GLB_JSON_CHUNK, "The first GLB chunk must be JSON.");
      jsonBytes = chunk;
    } else if (chunkType === GLB_BIN_CHUNK) {
      invariant(binaryBytes === undefined, "GLB contains more than one BIN chunk.");
      binaryBytes = chunk;
    }
    offset = chunkEnd;
    chunkIndex += 1;
  }
  invariant(offset === bytes.byteLength, "GLB chunks do not consume the file.");
  invariant(jsonBytes !== undefined, "GLB JSON chunk is missing.");

  let document;
  try {
    const jsonText = new TextDecoder("utf-8", { fatal: true })
      .decode(jsonBytes)
      .replace(/[\u0000\u0020]+$/u, "");
    document = JSON.parse(jsonText);
  } catch (error) {
    throw new Error(
      `GLB JSON chunk is invalid: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
  invariant(
    typeof document === "object"
      && document !== null
      && !Array.isArray(document),
    "GLB JSON root must be an object.",
  );
  invariant(document.asset?.version === "2.0", "GLB asset.version must be 2.0.");
  return { document, binaryBytes: binaryBytes ?? new Uint8Array() };
}

function validateBuffers(document, binaryBytes) {
  const buffers = requireArray(document.buffers, "buffers", { allowEmpty: false });
  invariant(buffers.length === 1, "Local Bridge GLB supports exactly one BIN buffer.");
  invariant(
    buffers[0]?.uri === undefined,
    "Binary GLB buffers must not reference an external URI.",
  );
  const declaredLength = buffers[0]?.byteLength;
  invariant(
    Number.isSafeInteger(declaredLength) && declaredLength >= 0,
    "buffers[0].byteLength must be a non-negative integer.",
  );
  invariant(
    declaredLength <= binaryBytes.byteLength
      && binaryBytes.byteLength - declaredLength <= 3,
    "GLB BIN chunk does not match buffers[0].byteLength.",
  );

  const bufferViews = optionalArray(document.bufferViews, "bufferViews");
  for (const [index, view] of bufferViews.entries()) {
    invariant(
      typeof view === "object" && view !== null && !Array.isArray(view),
      `bufferViews[${index}] must be an object.`,
    );
    invariant(view.buffer === 0, `bufferViews[${index}].buffer must be 0.`);
    const byteOffset = view.byteOffset ?? 0;
    invariant(
      Number.isSafeInteger(byteOffset) && byteOffset >= 0,
      `bufferViews[${index}].byteOffset is invalid.`,
    );
    invariant(
      Number.isSafeInteger(view.byteLength) && view.byteLength >= 0,
      `bufferViews[${index}].byteLength is invalid.`,
    );
    invariant(
      byteOffset + view.byteLength <= declaredLength,
      `bufferViews[${index}] exceeds the declared BIN buffer.`,
    );
    if (view.byteStride !== undefined) {
      invariant(
        Number.isSafeInteger(view.byteStride)
          && view.byteStride >= 4
          && view.byteStride <= 252
          && view.byteStride % 4 === 0,
        `bufferViews[${index}].byteStride is invalid.`,
      );
    }
  }
  return { declaredLength, bufferViews };
}

function accessorLayout(document, bufferViews, binaryBytes, accessorIndex) {
  const accessors = requireArray(document.accessors, "accessors", {
    allowEmpty: false,
  });
  const index = safeIndex(accessorIndex, accessors.length, "accessor index");
  const accessor = accessors[index];
  invariant(
    typeof accessor === "object" && accessor !== null && !Array.isArray(accessor),
    `accessors[${index}] must be an object.`,
  );
  invariant(
    accessor.sparse === undefined,
    `accessors[${index}] uses sparse storage, which is not accepted by the deterministic Bridge readback.`,
  );
  const viewIndex = safeIndex(
    accessor.bufferView,
    bufferViews.length,
    `accessors[${index}].bufferView`,
  );
  const view = bufferViews[viewIndex];
  const componentBytes = COMPONENT_BYTES.get(accessor.componentType);
  const components = TYPE_COMPONENTS.get(accessor.type);
  invariant(componentBytes !== undefined, `accessors[${index}].componentType is unsupported.`);
  invariant(components !== undefined, `accessors[${index}].type is unsupported.`);
  invariant(
    Number.isSafeInteger(accessor.count) && accessor.count >= 0,
    `accessors[${index}].count is invalid.`,
  );
  const elementBytes = componentBytes * components;
  const stride = view.byteStride ?? elementBytes;
  invariant(
    stride >= elementBytes,
    `accessors[${index}] element does not fit its bufferView stride.`,
  );
  const accessorOffset = accessor.byteOffset ?? 0;
  invariant(
    Number.isSafeInteger(accessorOffset) && accessorOffset >= 0,
    `accessors[${index}].byteOffset is invalid.`,
  );
  const viewOffset = view.byteOffset ?? 0;
  const start = viewOffset + accessorOffset;
  const consumed = accessor.count === 0
    ? 0
    : (accessor.count - 1) * stride + elementBytes;
  invariant(
    accessorOffset + consumed <= view.byteLength
      && start + consumed <= binaryBytes.byteLength,
    `accessors[${index}] exceeds its bufferView or BIN chunk.`,
  );
  return {
    accessor,
    index,
    start,
    stride,
    count: accessor.count,
    componentBytes,
    componentType: accessor.componentType,
    components,
    elementBytes,
    type: accessor.type,
  };
}

function readComponent(data, offset, componentType) {
  if (componentType === 5120) return data.getInt8(offset);
  if (componentType === 5121) return data.getUint8(offset);
  if (componentType === 5122) return data.getInt16(offset, true);
  if (componentType === 5123) return data.getUint16(offset, true);
  if (componentType === 5125) return data.getUint32(offset, true);
  if (componentType === 5126) return data.getFloat32(offset, true);
  throw new Error("Unsupported accessor component type.");
}

function readAccessor(document, bufferViews, binaryBytes, accessorIndex) {
  const layout = accessorLayout(
    document,
    bufferViews,
    binaryBytes,
    accessorIndex,
  );
  const data = new DataView(
    binaryBytes.buffer,
    binaryBytes.byteOffset,
    binaryBytes.byteLength,
  );
  const values = [];
  for (let item = 0; item < layout.count; item += 1) {
    const itemOffset = layout.start + item * layout.stride;
    const value = [];
    for (let component = 0; component < layout.components; component += 1) {
      const componentOffset = itemOffset + component * layout.componentBytes;
      const number = readComponent(data, componentOffset, layout.componentType);
      invariant(
        Number.isFinite(number),
        `accessors[${layout.index}] contains a non-finite value.`,
      );
      value.push(number);
    }
    values.push(value);
  }
  return { layout, values };
}

function validateAccessors(document, bufferViews, binaryBytes) {
  const accessors = requireArray(document.accessors, "accessors", {
    allowEmpty: false,
  });
  for (let index = 0; index < accessors.length; index += 1) {
    accessorLayout(document, bufferViews, binaryBytes, index);
  }
}

function validateNodesAndScenes(document) {
  const nodes = requireArray(document.nodes, "nodes", { allowEmpty: false });
  const meshes = requireArray(document.meshes, "meshes", { allowEmpty: false });
  const skins = optionalArray(document.skins, "skins");
  const parentByNode = new Map();
  for (const [nodeIndex, node] of nodes.entries()) {
    invariant(
      typeof node === "object" && node !== null && !Array.isArray(node),
      `nodes[${nodeIndex}] must be an object.`,
    );
    if (node.mesh !== undefined) {
      safeIndex(node.mesh, meshes.length, `nodes[${nodeIndex}].mesh`);
    }
    if (node.skin !== undefined) {
      safeIndex(node.skin, skins.length, `nodes[${nodeIndex}].skin`);
      invariant(
        node.mesh !== undefined,
        `nodes[${nodeIndex}] has a skin but no mesh.`,
      );
    }
    for (const [childIndex, child] of optionalArray(
      node.children,
      `nodes[${nodeIndex}].children`,
    ).entries()) {
      safeIndex(child, nodes.length, `nodes[${nodeIndex}].children[${childIndex}]`);
      invariant(child !== nodeIndex, `nodes[${nodeIndex}] cannot be its own child.`);
      invariant(
        !parentByNode.has(child),
        `nodes[${child}] has more than one parent.`,
      );
      parentByNode.set(child, nodeIndex);
    }
  }
  for (let nodeIndex = 0; nodeIndex < nodes.length; nodeIndex += 1) {
    const visited = new Set([nodeIndex]);
    let current = parentByNode.get(nodeIndex);
    while (current !== undefined) {
      invariant(!visited.has(current), "GLB node hierarchy contains a cycle.");
      visited.add(current);
      current = parentByNode.get(current);
    }
  }
  invariant(
    nodes.some((node) => node.mesh !== undefined),
    "GLB has no node that instantiates render geometry.",
  );

  const scenes = requireArray(document.scenes, "scenes", { allowEmpty: false });
  for (const [sceneIndex, scene] of scenes.entries()) {
    const roots = requireArray(scene?.nodes, `scenes[${sceneIndex}].nodes`, {
      allowEmpty: false,
    });
    roots.forEach((node, rootIndex) => {
      safeIndex(node, nodes.length, `scenes[${sceneIndex}].nodes[${rootIndex}]`);
      invariant(
        !parentByNode.has(node),
        `scenes[${sceneIndex}].nodes[${rootIndex}] is not a root node.`,
      );
    });
  }
  const activeSceneIndex = document.scene === undefined
    ? 0
    : safeIndex(document.scene, scenes.length, "scene");
  const activeNodeIds = new Set();
  const pending = [...scenes[activeSceneIndex].nodes];
  while (pending.length > 0) {
    const nodeIndex = pending.pop();
    if (activeNodeIds.has(nodeIndex)) continue;
    activeNodeIds.add(nodeIndex);
    pending.push(...optionalArray(
      nodes[nodeIndex].children,
      `nodes[${nodeIndex}].children`,
    ));
  }
  invariant(
    [...activeNodeIds].some((nodeIndex) => nodes[nodeIndex].mesh !== undefined),
    "The active GLB scene contains no render geometry.",
  );
  return { nodes, skins, parentByNode, activeNodeIds };
}

function validateGeometry(document, bufferViews, binaryBytes) {
  const meshes = requireArray(document.meshes, "meshes", { allowEmpty: false });
  let primitiveCount = 0;
  let triangleCount = 0;
  for (const [meshIndex, mesh] of meshes.entries()) {
    const primitives = requireArray(
      mesh?.primitives,
      `meshes[${meshIndex}].primitives`,
      { allowEmpty: false },
    );
    for (const [primitiveIndex, primitive] of primitives.entries()) {
      const path = `meshes[${meshIndex}].primitives[${primitiveIndex}]`;
      invariant(
        typeof primitive === "object"
          && primitive !== null
          && !Array.isArray(primitive),
        `${path} must be an object.`,
      );
      invariant(
        (primitive.mode ?? TRIANGLES_MODE) === TRIANGLES_MODE,
        `${path}.mode must be TRIANGLES.`,
      );
      invariant(
        typeof primitive.attributes === "object"
          && primitive.attributes !== null
          && !Array.isArray(primitive.attributes),
        `${path}.attributes must be an object.`,
      );
      invariant(
        Number.isSafeInteger(primitive.attributes.POSITION),
        `${path}.attributes.POSITION is required.`,
      );
      const position = accessorLayout(
        document,
        bufferViews,
        binaryBytes,
        primitive.attributes.POSITION,
      );
      invariant(
        position.type === "VEC3" && position.componentType === 5126,
        `${path}.attributes.POSITION must use FLOAT VEC3.`,
      );
      invariant(position.count > 0, `${path} has no vertices.`);
      for (const [semantic, accessorIndex] of Object.entries(
        primitive.attributes,
      )) {
        const attribute = accessorLayout(
          document,
          bufferViews,
          binaryBytes,
          accessorIndex,
        );
        invariant(
          attribute.count === position.count,
          `${path}.attributes.${semantic} has a different vertex count.`,
        );
      }

      let elementCount = position.count;
      if (primitive.indices !== undefined) {
        const indices = readAccessor(
          document,
          bufferViews,
          binaryBytes,
          primitive.indices,
        );
        invariant(
          indices.layout.type === "SCALAR"
            && [5121, 5123, 5125].includes(indices.layout.componentType),
          `${path}.indices must use an unsigned integer SCALAR accessor.`,
        );
        invariant(
          indices.values.every(([index]) => index < position.count),
          `${path}.indices references a vertex outside POSITION.`,
        );
        elementCount = indices.layout.count;
      }
      invariant(
        elementCount >= 3 && elementCount % 3 === 0,
        `${path} triangle element count is invalid.`,
      );
      if (primitive.material !== undefined) {
        safeIndex(
          primitive.material,
          optionalArray(document.materials, "materials").length,
          `${path}.material`,
        );
      }
      primitiveCount += 1;
      triangleCount += elementCount / 3;
    }
  }
  return { meshCount: meshes.length, primitiveCount, triangleCount };
}

function validateSkins(
  document,
  bufferViews,
  binaryBytes,
  nodes,
  skins,
  parentByNode,
) {
  const jointIds = new Set();
  const rootNodeIds = new Set();
  const inverseBindMatrices = [];
  for (const [skinIndex, skin] of skins.entries()) {
    const joints = requireArray(skin?.joints, `skins[${skinIndex}].joints`, {
      allowEmpty: false,
    });
    invariant(
      new Set(joints).size === joints.length,
      `skins[${skinIndex}].joints contains duplicates.`,
    );
    for (const [jointIndex, joint] of joints.entries()) {
      safeIndex(joint, nodes.length, `skins[${skinIndex}].joints[${jointIndex}]`);
      jointIds.add(joint);
    }
    if (skin.skeleton !== undefined) {
      safeIndex(skin.skeleton, nodes.length, `skins[${skinIndex}].skeleton`);
      rootNodeIds.add(skin.skeleton);
    }
    if (skin.inverseBindMatrices !== undefined) {
      const matrices = readAccessor(
        document,
        bufferViews,
        binaryBytes,
        skin.inverseBindMatrices,
      );
      invariant(
        matrices.layout.type === "MAT4"
          && matrices.layout.componentType === 5126
          && matrices.layout.count === joints.length,
        `skins[${skinIndex}].inverseBindMatrices must be FLOAT MAT4 with one matrix per joint.`,
      );
      inverseBindMatrices.push(...matrices.values);
    }
  }
  if (rootNodeIds.size === 0) {
    for (const joint of jointIds) {
      if (!jointIds.has(parentByNode.get(joint))) rootNodeIds.add(joint);
    }
  }
  const skeletonIdentity = [...jointIds]
    .sort((left, right) => left - right)
    .map((jointId) => ({
      nodeId: jointId,
      name: typeof nodes[jointId]?.name === "string"
        ? nodes[jointId].name
        : `node-${jointId}`,
      parentNodeId: parentByNode.get(jointId) ?? null,
    }));
  return {
    jointIds,
    rootNodeIds,
    skeleton: {
      jointCount: skeletonIdentity.length,
      signatureSha256: sha256(JSON.stringify({
        joints: skeletonIdentity,
        inverseBindMatrices,
      })),
      joints: skeletonIdentity,
    },
  };
}

function validateSkinBindings(
  document,
  bufferViews,
  binaryBytes,
  nodes,
  skins,
  activeNodeIds,
) {
  const meshes = requireArray(document.meshes, "meshes", { allowEmpty: false });
  let skinnedMeshNodeCount = 0;
  let skinnedPrimitiveCount = 0;
  let weightedVertexCount = 0;
  for (const [nodeIndex, node] of nodes.entries()) {
    if (node.skin === undefined) continue;
    const skinIndex = safeIndex(
      node.skin,
      skins.length,
      `nodes[${nodeIndex}].skin`,
    );
    const meshIndex = safeIndex(
      node.mesh,
      meshes.length,
      `nodes[${nodeIndex}].mesh`,
    );
    const jointsInSkin = requireArray(
      skins[skinIndex]?.joints,
      `skins[${skinIndex}].joints`,
      { allowEmpty: false },
    );
    const primitives = requireArray(
      meshes[meshIndex]?.primitives,
      `meshes[${meshIndex}].primitives`,
      { allowEmpty: false },
    );
    if (activeNodeIds.has(nodeIndex)) skinnedMeshNodeCount += 1;
    for (const [primitiveIndex, primitive] of primitives.entries()) {
      const path = `meshes[${meshIndex}].primitives[${primitiveIndex}]`;
      const jointAccessorIndex = primitive?.attributes?.JOINTS_0;
      const weightAccessorIndex = primitive?.attributes?.WEIGHTS_0;
      invariant(
        Number.isSafeInteger(jointAccessorIndex)
          && Number.isSafeInteger(weightAccessorIndex),
        `${path} is instantiated with a skin but has no JOINTS_0/WEIGHTS_0 binding.`,
      );
      const jointAccessor = readAccessor(
        document,
        bufferViews,
        binaryBytes,
        jointAccessorIndex,
      );
      const weightAccessor = readAccessor(
        document,
        bufferViews,
        binaryBytes,
        weightAccessorIndex,
      );
      invariant(
        jointAccessor.layout.type === "VEC4"
          && [5121, 5123].includes(jointAccessor.layout.componentType)
          && jointAccessor.layout.accessor.normalized !== true,
        `${path}.attributes.JOINTS_0 must be an unnormalized UNSIGNED_BYTE or UNSIGNED_SHORT VEC4 accessor.`,
      );
      invariant(
        weightAccessor.layout.type === "VEC4"
          && (
            weightAccessor.layout.componentType === 5126
            || (
              [5121, 5123].includes(weightAccessor.layout.componentType)
              && weightAccessor.layout.accessor.normalized === true
            )
          ),
        `${path}.attributes.WEIGHTS_0 must be FLOAT VEC4 or a normalized unsigned VEC4 accessor.`,
      );
      invariant(
        jointAccessor.layout.count === weightAccessor.layout.count,
        `${path} JOINTS_0 and WEIGHTS_0 vertex counts differ.`,
      );
      for (
        let vertexIndex = 0;
        vertexIndex < jointAccessor.values.length;
        vertexIndex += 1
      ) {
        const jointValues = jointAccessor.values[vertexIndex];
        const weightValues = weightAccessor.values[vertexIndex];
        invariant(
          jointValues.every((joint) => joint < jointsInSkin.length),
          `${path}.attributes.JOINTS_0 references a joint outside skins[${skinIndex}].joints.`,
        );
        invariant(
          weightValues.every((weight) => weight >= 0)
            && weightValues.some((weight) => weight > 0),
          `${path}.attributes.WEIGHTS_0 contains an unweighted vertex.`,
        );
      }
      if (activeNodeIds.has(nodeIndex)) {
        skinnedPrimitiveCount += 1;
        weightedVertexCount += jointAccessor.layout.count;
      }
    }
  }
  return {
    skinnedMeshNodeCount,
    skinnedPrimitiveCount,
    weightedVertexCount,
  };
}

function animationInventory(
  document,
  bufferViews,
  binaryBytes,
  nodes,
  rootNodeIds,
  jointIds,
) {
  return optionalArray(document.animations, "animations").map(
    (animation, animationIndex) => {
      const animationPath = `animations[${animationIndex}]`;
      const samplers = requireArray(
        animation?.samplers,
        `${animationPath}.samplers`,
        { allowEmpty: false },
      );
      const channels = requireArray(
        animation?.channels,
        `${animationPath}.channels`,
        { allowEmpty: false },
      );
      const samplerData = samplers.map((sampler, samplerIndex) => {
        const path = `${animationPath}.samplers[${samplerIndex}]`;
        const input = readAccessor(
          document,
          bufferViews,
          binaryBytes,
          sampler?.input,
        );
        const output = readAccessor(
          document,
          bufferViews,
          binaryBytes,
          sampler?.output,
        );
        invariant(
          input.layout.type === "SCALAR"
            && input.layout.componentType === 5126
            && input.layout.count > 0,
          `${path}.input must be a non-empty FLOAT SCALAR accessor.`,
        );
        const inputTimes = input.values.map(([value]) => value);
        invariant(
          inputTimes.every((value, index) => (
            value >= 0 && (index === 0 || value > inputTimes[index - 1])
          )),
          `${path}.input times must be finite, non-negative, and strictly increasing.`,
        );
        const interpolation = sampler.interpolation ?? "LINEAR";
        invariant(
          ["LINEAR", "STEP", "CUBICSPLINE"].includes(interpolation),
          `${path}.interpolation is unsupported.`,
        );
        return { input, inputTimes, interpolation, output, path };
      });

      let durationSeconds = 0;
      const paths = new Map();
      const channelKeys = new Set();
      let jointChannelCount = 0;
      let rootTranslationChannelCount = 0;
      let maximumRootTranslationDelta = 0;
      for (const [channelIndex, channel] of channels.entries()) {
        const path = `${animationPath}.channels[${channelIndex}]`;
        const samplerIndex = safeIndex(
          channel?.sampler,
          samplerData.length,
          `${path}.sampler`,
        );
        const sampler = samplerData[samplerIndex];
        const node = safeIndex(
          channel?.target?.node,
          nodes.length,
          `${path}.target.node`,
        );
        const targetPath = channel?.target?.path;
        invariant(
          ["translation", "rotation", "scale", "weights"].includes(targetPath),
          `${path}.target.path is unsupported.`,
        );
        invariant(
          !channelKeys.has(`${node}:${targetPath}`),
          `${animationPath} contains duplicate channels for node ${node} ${targetPath}.`,
        );
        channelKeys.add(`${node}:${targetPath}`);
        const expectedType = targetPath === "rotation"
          ? "VEC4"
          : targetPath === "weights"
            ? undefined
            : "VEC3";
        invariant(
          sampler.output.layout.componentType === 5126
            && (
              expectedType === undefined
              || sampler.output.layout.type === expectedType
            ),
          `${sampler.path}.output has the wrong type for ${targetPath}.`,
        );
        const outputFactor = sampler.interpolation === "CUBICSPLINE" ? 3 : 1;
        invariant(
          sampler.output.layout.count >= sampler.input.layout.count * outputFactor
            && (
              targetPath === "weights"
              || sampler.output.layout.count
                === sampler.input.layout.count * outputFactor
            ),
          `${sampler.path}.output count does not match its input key count.`,
        );
        durationSeconds = Math.max(
          durationSeconds,
          sampler.inputTimes.at(-1) ?? 0,
        );
        paths.set(targetPath, (paths.get(targetPath) ?? 0) + 1);
        if (
          jointIds.has(node)
          && ["translation", "rotation", "scale"].includes(targetPath)
        ) {
          jointChannelCount += 1;
        }

        if (targetPath === "translation" && rootNodeIds.has(node)) {
          rootTranslationChannelCount += 1;
          const values = sampler.output.values;
          const valueOffset = sampler.interpolation === "CUBICSPLINE" ? 1 : 0;
          const valueStep = sampler.interpolation === "CUBICSPLINE" ? 3 : 1;
          const poses = sampler.inputTimes.map(
            (_, index) => values[valueOffset + index * valueStep],
          );
          const origin = poses[0];
          maximumRootTranslationDelta = Math.max(
            maximumRootTranslationDelta,
            ...poses.map((value) => Math.hypot(
              value[0] - origin[0],
              value[1] - origin[1],
              value[2] - origin[2],
            )),
          );
        }
      }
      return {
        name: typeof animation.name === "string" && animation.name.trim()
          ? animation.name
          : `animation-${animationIndex}`,
        durationSeconds,
        channelCount: channels.length,
        jointChannelCount,
        samplerCount: samplers.length,
        channelPaths: Object.fromEntries([...paths].sort(
          ([left], [right]) => left.localeCompare(right, "en-US"),
        )),
        rootMotion: {
          assessed: true,
          rootTranslationChannelCount,
          maximumTranslationDelta: maximumRootTranslationDelta,
          classification: maximumRootTranslationDelta > 1e-4
            ? "ROOT_MOTION"
            : "IN_PLACE",
        },
      };
    },
  );
}

export function inspectGlbArtifactV1(bytes) {
  if (!(bytes instanceof Uint8Array)) {
    throw new TypeError("GLB inspection requires Uint8Array bytes.");
  }
  const { document, binaryBytes } = parseGlb(bytes);
  const { bufferViews } = validateBuffers(document, binaryBytes);
  validateAccessors(document, bufferViews, binaryBytes);
  const {
    nodes,
    skins,
    parentByNode,
    activeNodeIds,
  } = validateNodesAndScenes(document);
  const geometry = validateGeometry(document, bufferViews, binaryBytes);
  const { jointIds, rootNodeIds, skeleton } = validateSkins(
    document,
    bufferViews,
    binaryBytes,
    nodes,
    skins,
    parentByNode,
  );
  const skinBindings = validateSkinBindings(
    document,
    bufferViews,
    binaryBytes,
    nodes,
    skins,
    activeNodeIds,
  );
  const animations = animationInventory(
    document,
    bufferViews,
    binaryBytes,
    nodes,
    rootNodeIds,
    jointIds,
  );
  return {
    schemaVersion: 1,
    status: "PARSED",
    geometry,
    skeleton: {
      ...skeleton,
      ...skinBindings,
    },
    clipInventory: {
      clipCount: animations.length,
      clips: animations,
    },
  };
}
