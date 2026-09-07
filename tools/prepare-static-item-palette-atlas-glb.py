import hashlib
import json
import math
import sys
from pathlib import Path

import bpy
import numpy as np


ATLAS_SIZE = 2048
REGION_SIZE = 1024


def fail(message: str) -> None:
    raise RuntimeError(message)


def base_color_image(material: bpy.types.Material) -> bpy.types.Image:
    if material is None or not material.use_nodes or material.node_tree is None:
        fail("every source material must use nodes")
    principled = next(
        (node for node in material.node_tree.nodes if node.type == "BSDF_PRINCIPLED"),
        None,
    )
    if principled is None:
        fail(f"material {material.name!r} has no Principled BSDF")
    base_color = principled.inputs.get("Base Color")
    if base_color is None or not base_color.is_linked:
        fail(f"material {material.name!r} has no linked base-color image")
    image_node = base_color.links[0].from_node
    if image_node.type != "TEX_IMAGE" or image_node.image is None:
        fail(f"material {material.name!r} base color is not an image texture")
    return image_node.image


def resized_rgba(image: bpy.types.Image, size: int) -> np.ndarray:
    copied = image.copy()
    copied.name = f"{image.name}_palette_atlas_copy"
    copied.scale(size, size)
    pixels = np.empty(size * size * 4, dtype=np.float32)
    copied.pixels.foreach_get(pixels)
    bpy.data.images.remove(copied)
    return pixels.reshape((size, size, 4))


args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
if len(args) != 2:
    fail(
        "usage: blender --background --python prepare-static-item-palette-atlas-glb.py "
        "-- source.glb output.glb"
    )

source_path = Path(args[0]).resolve()
output_path = Path(args[1]).resolve()
if not source_path.is_file():
    fail(f"source does not exist: {source_path}")
if output_path.exists():
    fail(f"output already exists: {output_path}")

bpy.ops.wm.read_factory_settings(use_empty=True)
result = bpy.ops.import_scene.gltf(filepath=str(source_path))
if "FINISHED" not in result:
    fail("Blender GLB import did not finish")

mesh_objects = sorted(
    (candidate for candidate in bpy.context.scene.objects if candidate.type == "MESH"),
    key=lambda candidate: candidate.name,
)
if len(mesh_objects) != 1:
    fail(f"expected exactly one normalized mesh object, got {len(mesh_objects)}")
mesh_object = mesh_objects[0]
mesh = mesh_object.data
if len(mesh.materials) != 2:
    fail(f"expected exactly two source material slots, got {len(mesh.materials)}")
if not mesh.uv_layers or mesh.uv_layers.active is None:
    fail("source mesh has no active UV layer")

source_images = [base_color_image(material) for material in mesh.materials]
source_dimensions = [[int(image.size[0]), int(image.size[1])] for image in source_images]
if any(width <= 0 or height <= 0 for width, height in source_dimensions):
    fail(f"source material has invalid image dimensions: {source_dimensions}")

# Blender exposes image rows bottom-up. The upper-left and upper-right atlas
# quadrants therefore occupy rows 1024..2048. Both source images are normalized
# to a square 1024 region; the model samples only those two regions.
atlas_pixels = np.zeros((ATLAS_SIZE, ATLAS_SIZE, 4), dtype=np.float32)
atlas_pixels[REGION_SIZE:ATLAS_SIZE, 0:REGION_SIZE, :] = resized_rgba(
    source_images[0], REGION_SIZE
)
atlas_pixels[REGION_SIZE:ATLAS_SIZE, REGION_SIZE:ATLAS_SIZE, :] = resized_rgba(
    source_images[1], REGION_SIZE
)

atlas = bpy.data.images.new(
    "bronze_mask_item_palette_atlas",
    width=ATLAS_SIZE,
    height=ATLAS_SIZE,
    alpha=True,
    float_buffer=False,
)
atlas.colorspace_settings.name = "sRGB"
atlas.pixels.foreach_set(atlas_pixels.reshape(-1))
atlas.update()
atlas.pack()

source_material_indices = [polygon.material_index for polygon in mesh.polygons]
if set(source_material_indices) != {0, 1}:
    fail(
        "source polygons do not exercise exactly material slots 0 and 1: "
        f"{sorted(set(source_material_indices))}"
    )

uv_layer = mesh.uv_layers.active.data
uv_min = [math.inf, math.inf]
uv_max = [-math.inf, -math.inf]
for polygon in mesh.polygons:
    material_index = polygon.material_index
    x_offset = 0.0 if material_index == 0 else 0.5
    for loop_index in polygon.loop_indices:
        uv = uv_layer[loop_index].uv
        u = float(uv.x)
        v = float(uv.y)
        uv_min[0] = min(uv_min[0], u)
        uv_min[1] = min(uv_min[1], v)
        uv_max[0] = max(uv_max[0], u)
        uv_max[1] = max(uv_max[1], v)
        if u < -1.0e-5 or u > 1.00001 or v < -1.0e-5 or v > 1.00001:
            fail(f"source UV outside the supported [0,1] atlas contract: {(u, v)}")
        uv.x = x_offset + 0.5 * min(1.0, max(0.0, u))
        uv.y = 0.5 + 0.5 * min(1.0, max(0.0, v))
    polygon.material_index = 0

atlas_material = bpy.data.materials.new("bronze_mask_item_palette_atlas_material")
atlas_material.use_nodes = True
nodes = atlas_material.node_tree.nodes
links = atlas_material.node_tree.links
for node in list(nodes):
    nodes.remove(node)
output_node = nodes.new("ShaderNodeOutputMaterial")
principled = nodes.new("ShaderNodeBsdfPrincipled")
image_node = nodes.new("ShaderNodeTexImage")
image_node.image = atlas
links.new(image_node.outputs["Color"], principled.inputs["Base Color"])
links.new(image_node.outputs["Alpha"], principled.inputs["Alpha"])
links.new(principled.outputs["BSDF"], output_node.inputs["Surface"])

mesh.materials.clear()
mesh.materials.append(atlas_material)

bpy.ops.object.select_all(action="DESELECT")
mesh_object.select_set(True)
bpy.context.view_layer.objects.active = mesh_object
result = bpy.ops.export_scene.gltf(
    filepath=str(output_path),
    export_format="GLB",
    use_selection=True,
    export_materials="EXPORT",
    export_texcoords=True,
    export_normals=True,
    export_tangents=False,
    export_animations=False,
    export_skins=False,
    export_yup=True,
)
if "FINISHED" not in result or not output_path.is_file():
    fail("Blender GLB export did not finish")

payload = output_path.read_bytes()
report = {
    "schemaVersion": 1,
    "sourceSha256": hashlib.sha256(source_path.read_bytes()).hexdigest(),
    "outputSha256": hashlib.sha256(payload).hexdigest(),
    "outputByteLength": len(payload),
    "sourceMaterialSlotCount": 2,
    "outputMaterialSlotCount": len(mesh.materials),
    "sourceImageDimensions": source_dimensions,
    "atlasDimensions": [ATLAS_SIZE, ATLAS_SIZE],
    "sourceUvBounds": {"min": uv_min, "max": uv_max},
    "regions": [
        {
            "sourceMaterialId": 0,
            "semantic": "cloth",
            "pixelRectTopLeft": [0, 0, REGION_SIZE, REGION_SIZE],
            "uvRect": [0.0, 0.5, 0.5, 1.0],
        },
        {
            "sourceMaterialId": 1,
            "semantic": "metal",
            "pixelRectTopLeft": [REGION_SIZE, 0, REGION_SIZE, REGION_SIZE],
            "uvRect": [0.5, 0.5, 0.5, 1.0],
        },
    ],
    "unusedLowerHalfTransparent": True,
    "triangleCount": len(mesh.polygons),
}
print(json.dumps(report, indent=2))
