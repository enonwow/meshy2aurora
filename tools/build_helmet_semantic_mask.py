"""Build a coherent Metal 1 mask for a single-texture helmet GLB.

The input texture can contain bronze-coloured highlights on cloth.  Classifying
individual texels therefore produces Metal 1 speckles.  This tool classifies
triangles, joins UV-seam duplicates by position, removes small metal islands,
and rasterizes the retained surface regions back into texture space.

Usage:
  python tools/build_helmet_semantic_mask.py input.glb output.png [min_faces]
"""

from __future__ import annotations

import argparse
import io
import json
import math
import struct
import sys
from collections import defaultdict, deque
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw


COMPONENT_DTYPES = {
    5121: np.uint8,
    5123: np.uint16,
    5125: np.uint32,
    5126: np.float32,
}
COMPONENT_COUNTS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}


def read_glb(path: Path) -> tuple[dict, bytes]:
    payload = path.read_bytes()
    if payload[:4] != b"glTF" or struct.unpack_from("<I", payload, 4)[0] != 2:
        raise ValueError("expected a GLB v2 file")
    offset = 12
    document = None
    binary = None
    while offset < len(payload):
        length, kind = struct.unpack_from("<II", payload, offset)
        offset += 8
        chunk = payload[offset : offset + length]
        offset += length
        if kind == 0x4E4F534A:
            document = json.loads(chunk.rstrip(b" \x00"))
        elif kind == 0x004E4942:
            binary = chunk
    if document is None or binary is None:
        raise ValueError("GLB is missing JSON or BIN chunk")
    return document, binary


def accessor(document: dict, binary: bytes, index: int) -> np.ndarray:
    spec = document["accessors"][index]
    view = document["bufferViews"][spec["bufferView"]]
    dtype = np.dtype(COMPONENT_DTYPES[spec["componentType"]]).newbyteorder("<")
    count = COMPONENT_COUNTS[spec["type"]]
    start = view.get("byteOffset", 0) + spec.get("byteOffset", 0)
    stride = view.get("byteStride")
    if stride is None:
        values = np.frombuffer(binary, dtype=dtype, count=spec["count"] * count, offset=start)
        return values.reshape(spec["count"], count)
    rows = np.empty((spec["count"], count), dtype=dtype)
    row_bytes = dtype.itemsize * count
    for row in range(spec["count"]):
        rows[row] = np.frombuffer(
            binary,
            dtype=dtype,
            count=count,
            offset=start + row * stride,
        )
    return rows


def embedded_image(document: dict, binary: bytes) -> Image.Image:
    image = document["images"][0]
    view = document["bufferViews"][image["bufferView"]]
    start = view.get("byteOffset", 0)
    end = start + view["byteLength"]
    return Image.open(io.BytesIO(binary[start:end])).convert("RGB")


def texture_sample(texture: np.ndarray, uv: np.ndarray) -> np.ndarray:
    height, width, _ = texture.shape
    x = np.clip(np.rint((uv[:, 0] % 1.0) * (width - 1)), 0, width - 1).astype(np.int64)
    # glTF texture coordinates address the stored image from its upper-left
    # origin.  Pillow exposes the same row order, so V is used directly.
    y = np.clip(np.rint((uv[:, 1] % 1.0) * (height - 1)), 0, height - 1).astype(np.int64)
    return texture[y, x].astype(np.float32)


def classify_faces(texture: np.ndarray, uv: np.ndarray, faces: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    weights = np.asarray(
        [
            (1 / 3, 1 / 3, 1 / 3),
            (0.70, 0.15, 0.15),
            (0.15, 0.70, 0.15),
            (0.15, 0.15, 0.70),
            (0.50, 0.40, 0.10),
            (0.10, 0.50, 0.40),
            (0.40, 0.10, 0.50),
        ],
        dtype=np.float32,
    )
    tri_uv = uv[faces]
    samples = np.concatenate(
        [texture_sample(texture, np.einsum("j,ijk->ik", weight, tri_uv)) for weight in weights],
        axis=1,
    ).reshape(len(faces), len(weights), 3)
    warm = (samples[:, :, 0] - samples[:, :, 2] >= 10.0) & (
        samples[:, :, 1] - samples[:, :, 2] >= 2.0
    )
    fraction = warm.mean(axis=1)
    return fraction >= 0.50, fraction


def welded_adjacency(positions: np.ndarray, faces: np.ndarray) -> list[list[int]]:
    keys = np.rint(positions.astype(np.float64) * 100_000.0).astype(np.int64)
    welded: dict[tuple[int, int, int], int] = {}
    ids = np.empty(len(keys), dtype=np.int64)
    for index, key in enumerate(map(tuple, keys)):
        ids[index] = welded.setdefault(key, len(welded))
    edges: dict[tuple[int, int], list[int]] = defaultdict(list)
    for face_index, face in enumerate(faces):
        a, b, c = (int(ids[int(index)]) for index in face)
        for left, right in ((a, b), (b, c), (c, a)):
            if left != right:
                edges[(min(left, right), max(left, right))].append(face_index)
    adjacency = [set() for _ in range(len(faces))]
    for incident in edges.values():
        for face in incident:
            adjacency[face].update(other for other in incident if other != face)
    return [sorted(neighbors) for neighbors in adjacency]


def connected_components(selected: np.ndarray, adjacency: list[list[int]]) -> list[list[int]]:
    seen = np.zeros(len(selected), dtype=bool)
    components: list[list[int]] = []
    for root in np.flatnonzero(selected):
        if seen[root]:
            continue
        seen[root] = True
        queue = [int(root)]
        component: list[int] = []
        while queue:
            face = queue.pop()
            component.append(face)
            for neighbor in adjacency[face]:
                if selected[neighbor] and not seen[neighbor]:
                    seen[neighbor] = True
                    queue.append(neighbor)
        components.append(component)
    return components


def rasterize_mask(size: tuple[int, int], uv: np.ndarray, faces: np.ndarray, selected: np.ndarray) -> Image.Image:
    width, height = size
    result = Image.new("L", size, 0)
    draw = ImageDraw.Draw(result)
    for face in faces[selected]:
        points = []
        for texcoord in uv[face]:
            points.append(
                (
                    float(texcoord[0] % 1.0) * (width - 1),
                    float(texcoord[1] % 1.0) * (height - 1),
                )
            )
        draw.polygon(points, fill=255)
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--mode", choices=("components", "face-shell"), default="components")
    parser.add_argument("--min-faces", type=int, default=80)
    parser.add_argument("--min-height", type=float, default=0.10)
    parser.add_argument("--max-height", type=float, default=0.58)
    parser.add_argument("--min-depth", type=float, default=0.04)
    parser.add_argument("--preview", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    source = args.input.resolve()
    output = args.output.resolve()
    minimum_faces = args.min_faces
    for path in (output, args.preview, args.report):
        if path is not None and path.exists():
            raise SystemExit(f"refusing to overwrite: {path}")

    document, binary = read_glb(source)
    primitive = document["meshes"][0]["primitives"][0]
    positions = accessor(document, binary, primitive["attributes"]["POSITION"]).astype(np.float32)
    uv = accessor(document, binary, primitive["attributes"]["TEXCOORD_0"]).astype(np.float32)
    faces = accessor(document, binary, primitive["indices"]).reshape(-1, 3).astype(np.int64)
    source_texture = embedded_image(document, binary)
    texture = np.asarray(source_texture)

    initially_metal, confidence = classify_faces(texture, uv, faces)
    components: list[list[int]] = []
    if args.mode == "components":
        adjacency = welded_adjacency(positions, faces)
        components = connected_components(initially_metal, adjacency)
        components.sort(key=len, reverse=True)
        retained = np.zeros(len(faces), dtype=bool)
        for component in components:
            if len(component) >= minimum_faces:
                retained[component] = True
    else:
        centroids = positions[faces].mean(axis=1)
        retained = (
            (centroids[:, 1] >= args.min_height)
            & (centroids[:, 1] <= args.max_height)
            & (centroids[:, 2] >= args.min_depth)
        )

    output.parent.mkdir(parents=True, exist_ok=True)
    mask = rasterize_mask(source_texture.size, uv, faces, retained)
    mask.save(output)
    if args.preview is not None:
        args.preview.parent.mkdir(parents=True, exist_ok=True)
        preview = Image.new("RGB", mask.size, (110, 0, 25))
        preview.paste((230, 215, 40), mask=mask)
        preview.save(args.preview)

    report = {
                "source": str(source),
                "output": str(output),
                "minimumFaces": minimum_faces,
                "mode": args.mode,
                "faceShell": {
                    "minimumHeight": args.min_height,
                    "maximumHeight": args.max_height,
                    "minimumDepth": args.min_depth,
                },
                "triangles": len(faces),
                "initialMetalTriangles": int(initially_metal.sum()),
                "retainedMetalTriangles": int(retained.sum()),
                "metalComponents": len(components),
                "retainedComponents": sum(len(component) >= minimum_faces for component in components),
                "largestComponents": [
                    {
                        "faces": len(component),
                        "meanWarmFraction": float(confidence[component].mean()),
                        "centroid": positions[faces[component]].mean(axis=(0, 1)).tolist(),
                    }
                    for component in components[:20]
                ],
            }
    report_json = json.dumps(report, indent=2) + "\n"
    if args.report is not None:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(report_json, encoding="utf-8")
    print(report_json, end="")


if __name__ == "__main__":
    main()
