from __future__ import annotations

import argparse
import json
import struct
from pathlib import Path

import numpy as np
from PIL import Image


def read_glb(path: Path) -> tuple[dict, bytes]:
    payload = path.read_bytes()
    magic, version, total = struct.unpack_from("<III", payload, 0)
    if magic != 0x46546C67 or version != 2 or total != len(payload):
        raise ValueError("expected an exact GLB 2.0 payload")
    offset = 12
    document = None
    binary = None
    while offset < len(payload):
        length, chunk_type = struct.unpack_from("<II", payload, offset)
        offset += 8
        chunk = payload[offset : offset + length]
        offset += length
        if chunk_type == 0x4E4F534A:
            document = json.loads(chunk.decode("utf-8").rstrip("\x00 \t\r\n"))
        elif chunk_type == 0x004E4942:
            binary = chunk
    if document is None or binary is None:
        raise ValueError("GLB has no JSON or BIN chunk")
    return document, binary


def quaternion_matrix(rotation: list[float]) -> np.ndarray:
    x, y, z, w = rotation
    length = np.linalg.norm([x, y, z, w])
    if length == 0:
        return np.eye(4)
    x, y, z, w = np.asarray([x, y, z, w], dtype=np.float64) / length
    matrix = np.eye(4)
    matrix[:3, :3] = [
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ]
    return matrix


def local_matrix(node: dict) -> np.ndarray:
    if "matrix" in node:
        return np.asarray(node["matrix"], dtype=np.float64).reshape((4, 4), order="F")
    translation = np.eye(4)
    translation[:3, 3] = node.get("translation", [0.0, 0.0, 0.0])
    scale = np.eye(4)
    scale[0, 0], scale[1, 1], scale[2, 2] = node.get("scale", [1.0, 1.0, 1.0])
    return translation @ quaternion_matrix(node.get("rotation", [0.0, 0.0, 0.0, 1.0])) @ scale


def accessor_positions(document: dict, binary: bytes, accessor_index: int) -> np.ndarray:
    accessor = document["accessors"][accessor_index]
    if accessor["componentType"] != 5126 or accessor["type"] != "VEC3":
        raise ValueError("POSITION must be FLOAT VEC3")
    view = document["bufferViews"][accessor["bufferView"]]
    start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    stride = view.get("byteStride", 12)
    count = accessor["count"]
    if stride == 12:
        return np.frombuffer(binary, dtype="<f4", count=count * 3, offset=start).reshape((-1, 3)).copy()
    return np.vstack(
        [np.frombuffer(binary, dtype="<f4", count=3, offset=start + index * stride) for index in range(count)]
    )


def collect_positions(document: dict, binary: bytes) -> np.ndarray:
    nodes = document.get("nodes", [])
    scene_index = document.get("scene", 0)
    roots = document.get("scenes", [{}])[scene_index].get("nodes", list(range(len(nodes))))
    result: list[np.ndarray] = []

    def visit(index: int, parent: np.ndarray) -> None:
        node = nodes[index]
        world = parent @ local_matrix(node)
        if "mesh" in node:
            for primitive in document["meshes"][node["mesh"]].get("primitives", []):
                position_accessor = primitive.get("attributes", {}).get("POSITION")
                if position_accessor is None:
                    continue
                positions = accessor_positions(document, binary, position_accessor)
                homogeneous = np.column_stack((positions, np.ones(len(positions))))
                result.append((homogeneous @ world.T)[:, :3])
        for child in node.get("children", []):
            visit(child, world)

    for root in roots:
        visit(root, np.eye(4))
    if not result:
        raise ValueError("GLB contains no POSITION vertices")
    return np.vstack(result)


def render_projection(points: np.ndarray, horizontal: int, vertical: int, depth: int, output: Path) -> None:
    width, height, margin = 1600, 1200, 70
    projected = points[:, [horizontal, vertical]]
    low = projected.min(axis=0)
    high = projected.max(axis=0)
    span = np.maximum(high - low, 1e-9)
    scale = min((width - 2 * margin) / span[0], (height - 2 * margin) / span[1])
    xy = (projected - low) * scale
    xy[:, 0] += (width - span[0] * scale) / 2
    xy[:, 1] += (height - span[1] * scale) / 2
    xy[:, 1] = height - 1 - xy[:, 1]
    xy = np.rint(xy).astype(np.int32)
    order = np.argsort(points[:, depth])
    depth_values = points[order, depth]
    normalized_depth = (depth_values - depth_values.min()) / max(np.ptp(depth_values), 1e-9)
    pixels = np.full((height, width, 3), 232, dtype=np.uint8)
    colors = np.column_stack(
        (
            214 + 25 * normalized_depth,
            184 + 35 * normalized_depth,
            130 + 55 * normalized_depth,
        )
    ).astype(np.uint8)
    ordered_xy = xy[order]
    for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
        x = np.clip(ordered_xy[:, 0] + dx, 0, width - 1)
        y = np.clip(ordered_xy[:, 1] + dy, 0, height - 1)
        pixels[y, x] = colors
    Image.fromarray(pixels, "RGB").save(output)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    document, binary = read_glb(args.source)
    points = collect_positions(document, binary)
    render_projection(points, 0, 2, 1, args.output / "xz.png")
    render_projection(points, 2, 1, 0, args.output / "zy.png")
    render_projection(points, 0, 1, 2, args.output / "xy.png")
    bounds = {"minimum": points.min(axis=0).tolist(), "maximum": points.max(axis=0).tolist(), "vertexCount": len(points)}
    (args.output / "bounds.json").write_text(json.dumps(bounds, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
