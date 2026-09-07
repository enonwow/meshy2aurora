from __future__ import annotations

import hashlib
import json
import math
import sys
from pathlib import Path

import bmesh
import bpy
import numpy as np
from mathutils import Vector


def fail(message: str) -> None:
    raise RuntimeError(message)


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


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


def resized_rgba(image: bpy.types.Image, width: int, height: int) -> np.ndarray:
    copied = image.copy()
    copied.name = f"{image.name}_palette_atlas_copy"
    copied.scale(width, height)
    pixels = np.empty(width * height * 4, dtype=np.float32)
    copied.pixels.foreach_get(pixels)
    bpy.data.images.remove(copied)
    return pixels.reshape((height, width, 4))


def look_at(obj: bpy.types.Object, target: Vector) -> None:
    obj.rotation_euler = (target - obj.location).to_track_quat("-Z", "Y").to_euler()


def add_area_light(
    name: str,
    location: Vector,
    energy: float,
    size: float,
    target: Vector,
) -> None:
    light = bpy.data.lights.new(name=name, type="AREA")
    light.energy = energy
    light.shape = "DISK"
    light.size = size
    obj = bpy.data.objects.new(name, light)
    bpy.context.collection.objects.link(obj)
    obj.location = location
    look_at(obj, target)


def imported_meshes() -> list[bpy.types.Object]:
    return [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]


def import_source(source_path: Path) -> list[bpy.types.Object]:
    bpy.ops.wm.read_factory_settings(use_empty=True)
    result = bpy.ops.import_scene.gltf(filepath=str(source_path), merge_vertices=False)
    if "FINISHED" not in result:
        fail("Blender GLB import did not finish")
    meshes = imported_meshes()
    if not meshes:
        fail("source has no mesh objects")
    return meshes


def render_icon_sources(
    source_path: Path,
    icon_output: Path,
    material_specs: list[dict],
) -> dict:
    if icon_output.exists():
        fail(f"icon output already exists: {icon_output}")
    icon_output.mkdir(parents=True)
    meshes = import_source(source_path)
    material_by_name = {material.name: material for material in bpy.data.materials}
    expected_names = {entry["sourceMaterialName"] for entry in material_specs}
    if set(material_by_name) != expected_names:
        fail(
            "source materials differ from the mapping specification: "
            f"source={sorted(material_by_name)}, expected={sorted(expected_names)}"
        )

    corners = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
    minimum = Vector(tuple(min(point[axis] for point in corners) for axis in range(3)))
    maximum = Vector(tuple(max(point[axis] for point in corners) for axis in range(3)))
    center = (minimum + maximum) * 0.5
    span = maximum - minimum
    largest_span = max(span.x, span.y, span.z)

    camera_data = bpy.data.cameras.new("IconCamera")
    camera = bpy.data.objects.new("IconCamera", camera_data)
    bpy.context.collection.objects.link(camera)
    camera_data.type = "ORTHO"
    camera_data.ortho_scale = max(span.z * 1.10, max(span.x, span.y) * 1.10)
    camera.location = center + Vector((0.0, -1.0, 0.0)) * (largest_span * 4.0 + 1.0)
    look_at(camera, center)

    add_area_light(
        "Key",
        center + Vector((-1.8, -2.2, 2.4)) * largest_span,
        650.0,
        largest_span * 2.0,
        center,
    )
    add_area_light(
        "Fill",
        center + Vector((2.0, -0.7, 0.8)) * largest_span,
        260.0,
        largest_span * 1.6,
        center,
    )
    add_area_light(
        "Rim",
        center + Vector((0.2, 2.2, 1.7)) * largest_span,
        420.0,
        largest_span * 1.4,
        center,
    )

    world = bpy.data.worlds.new("IconWorld")
    world.use_nodes = True
    background = world.node_tree.nodes.get("Background")
    background.inputs["Color"].default_value = (0.012, 0.015, 0.018, 1.0)
    background.inputs["Strength"].default_value = 0.16

    scene = bpy.context.scene
    scene.camera = camera
    scene.world = world
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 1024
    scene.render.resolution_y = 1024
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.image_settings.color_depth = "8"
    scene.render.film_transparent = True
    scene.view_settings.look = "AgX - Medium High Contrast"

    for material in bpy.data.materials:
        if not material.use_nodes:
            continue
        for node in material.node_tree.nodes:
            if node.type != "BSDF_PRINCIPLED":
                continue
            metallic = node.inputs.get("Metallic")
            if metallic is not None:
                metallic.default_value = 0.0
            roughness = node.inputs.get("Roughness")
            if roughness is not None:
                roughness.default_value = 0.72

    beauty_path = icon_output / "icon-beauty.png"
    scene.render.filepath = str(beauty_path)
    bpy.ops.render.render(write_still=True)

    for entry in material_specs:
        material = material_by_name[entry["sourceMaterialName"]]
        principled = next(
            node for node in material.node_tree.nodes if node.type == "BSDF_PRINCIPLED"
        )
        rgb = [float(value) / 255.0 for value in entry["iconIdRgb"]]
        base_color = principled.inputs.get("Base Color")
        for link in list(base_color.links):
            material.node_tree.links.remove(link)
        base_color.default_value = (*rgb, 1.0)
        emission_color = principled.inputs.get("Emission Color")
        if emission_color is not None:
            emission_color.default_value = (*rgb, 1.0)
        emission_strength = principled.inputs.get("Emission Strength")
        if emission_strength is not None:
            emission_strength.default_value = 1.0
        roughness = principled.inputs.get("Roughness")
        if roughness is not None:
            roughness.default_value = 1.0

    scene.world.node_tree.nodes.get("Background").inputs["Strength"].default_value = 0.0
    scene.view_settings.view_transform = "Standard"
    material_id_path = icon_output / "icon-material-id.png"
    scene.render.filepath = str(material_id_path)
    bpy.ops.render.render(write_still=True)

    return {
        "beautyPath": beauty_path.name,
        "beautySha256": sha256_file(beauty_path),
        "materialIdPath": material_id_path.name,
        "materialIdSha256": sha256_file(material_id_path),
        "dimensions": [1024, 1024],
        "camera": "orthographic-front-negative-y",
        "filmTransparent": True,
    }


def validate_spec(spec: dict, source_materials: list[bpy.types.Material]) -> list[dict]:
    if spec.get("schemaVersion") != 1:
        fail("mapping specification requires schemaVersion 1")
    atlas_size = spec.get("atlasSize")
    if not isinstance(atlas_size, int) or atlas_size <= 0:
        fail("atlasSize must be a positive integer")
    materials = spec.get("materials")
    if not isinstance(materials, list) or not materials:
        fail("mapping specification has no materials")
    expected_ids = list(range(len(source_materials)))
    ids = [entry.get("sourceMaterialId") for entry in materials]
    if ids != expected_ids:
        fail(f"source material IDs must be ordered and contiguous: {ids}")
    names = [material.name for material in source_materials]
    expected_names = [entry.get("sourceMaterialName") for entry in materials]
    if names != expected_names:
        fail(f"source material order differs from mapping: {names} != {expected_names}")

    occupied: list[tuple[int, int, int, int]] = []
    for index, entry in enumerate(materials):
        rect = entry.get("pixelRectTopLeft")
        if (
            not isinstance(rect, list)
            or len(rect) != 4
            or any(not isinstance(value, int) for value in rect)
        ):
            fail(f"material {index} has an invalid pixelRectTopLeft")
        x, y, width, height = rect
        if x < 0 or y < 0 or width <= 0 or height <= 0:
            fail(f"material {index} has a non-positive atlas rectangle")
        if x + width > atlas_size or y + height > atlas_size:
            fail(f"material {index} atlas rectangle exceeds atlas bounds")
        for other_x, other_y, other_width, other_height in occupied:
            overlap = not (
                x + width <= other_x
                or other_x + other_width <= x
                or y + height <= other_y
                or other_y + other_height <= y
            )
            if overlap:
                fail(f"material {index} atlas rectangle overlaps an earlier rectangle")
        occupied.append((x, y, width, height))
        layer = entry.get("nwnPltLayerIndex")
        if not isinstance(layer, int) or layer < 0 or layer > 9:
            fail(f"material {index} has an invalid NWN PLT layer index")
        rgb = entry.get("iconIdRgb")
        if (
            not isinstance(rgb, list)
            or len(rgb) != 3
            or any(not isinstance(value, int) or value < 0 or value > 255 for value in rgb)
        ):
            fail(f"material {index} has an invalid iconIdRgb")
    return materials


def build_atlas(source_path: Path, output_path: Path, spec: dict) -> dict:
    if output_path.exists():
        fail(f"output already exists: {output_path}")
    mesh_objects = import_source(source_path)
    specified_names = [entry["sourceMaterialName"] for entry in spec.get("materials", [])]
    imported_by_name = {material.name: material for material in bpy.data.materials}
    if set(imported_by_name) != set(specified_names):
        fail(
            "source materials differ from the mapping specification: "
            f"source={sorted(imported_by_name)}, expected={sorted(specified_names)}"
        )
    source_materials = [imported_by_name[name] for name in specified_names]
    material_specs = validate_spec(spec, source_materials)
    spec_by_name = {entry["sourceMaterialName"]: entry for entry in material_specs}
    atlas_size = int(spec["atlasSize"])

    source_images = [base_color_image(material) for material in source_materials]
    source_dimensions = [[int(image.size[0]), int(image.size[1])] for image in source_images]
    atlas_pixels = np.zeros((atlas_size, atlas_size, 4), dtype=np.float32)
    for entry, image in zip(material_specs, source_images, strict=True):
        x, y, width, height = entry["pixelRectTopLeft"]
        blender_y = atlas_size - y - height
        atlas_pixels[blender_y : blender_y + height, x : x + width, :] = resized_rgba(
            image, width, height
        )

    atlas = bpy.data.images.new(
        "bronze_mask_item_palette_atlas_v2",
        width=atlas_size,
        height=atlas_size,
        alpha=True,
        float_buffer=False,
    )
    atlas.colorspace_settings.name = "sRGB"
    atlas.pixels.foreach_set(atlas_pixels.reshape(-1))
    atlas.update()
    atlas.pack()

    atlas_material = bpy.data.materials.new("bronze_mask_item_palette_atlas_material_v2")
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

    uv_min = [math.inf, math.inf]
    uv_max = [-math.inf, -math.inf]
    source_polygon_counts = [0 for _ in material_specs]
    for mesh_object in mesh_objects:
        mesh = mesh_object.data
        if not mesh.uv_layers or mesh.uv_layers.active is None:
            fail(f"mesh {mesh_object.name!r} has no active UV layer")
        uv_layer = mesh.uv_layers.active.data
        for polygon in mesh.polygons:
            material = mesh.materials[polygon.material_index]
            entry = spec_by_name.get(material.name)
            if entry is None:
                fail(f"polygon references unmapped material {material.name!r}")
            material_index = int(entry["sourceMaterialId"])
            source_polygon_counts[material_index] += 1
            x, y, width, height = entry["pixelRectTopLeft"]
            for loop_index in polygon.loop_indices:
                uv = uv_layer[loop_index].uv
                u = float(uv.x)
                v = float(uv.y)
                uv_min[0] = min(uv_min[0], u)
                uv_min[1] = min(uv_min[1], v)
                uv_max[0] = max(uv_max[0], u)
                uv_max[1] = max(uv_max[1], v)
                if u < -1.0e-5 or u > 1.00001 or v < -1.0e-5 or v > 1.00001:
                    fail(f"source UV outside [0,1]: {(u, v)}")
                uv.x = (x + min(1.0, max(0.0, u)) * width) / atlas_size
                uv.y = 1.0 - (y + height) / atlas_size + min(1.0, max(0.0, v)) * height / atlas_size
            polygon.material_index = 0
        mesh.materials.clear()
        mesh.materials.append(atlas_material)

    bpy.ops.object.select_all(action="DESELECT")
    for mesh_object in mesh_objects:
        mesh_object.select_set(True)
    bpy.context.view_layer.objects.active = mesh_objects[0]
    result = bpy.ops.object.join()
    if "FINISHED" not in result:
        fail("Blender mesh join did not finish")
    joined = bpy.context.view_layer.objects.active
    joined.name = "bronze_mask_palette_atlas_static_v2"
    joined.data.materials.clear()
    joined.data.materials.append(atlas_material)
    for polygon in joined.data.polygons:
        polygon.material_index = 0

    mesh = joined.data
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bm.faces.ensure_lookup_table()
    degenerate_faces = [
        face
        for face in bm.faces
        # The glTF writer stores positions as f32. Faces below this bound can
        # collapse to exact collinearity after that conversion even when
        # Blender's pre-export area remains non-zero.
        if not math.isfinite(face.calc_area()) or face.calc_area() <= 1.0e-10
    ]
    removed_degenerate_triangle_count = len(degenerate_faces)
    if degenerate_faces:
        bmesh.ops.delete(bm, geom=degenerate_faces, context="FACES")
    bm.to_mesh(mesh)
    bm.free()
    mesh.update(calc_edges=True)

    bpy.ops.object.select_all(action="DESELECT")
    joined.select_set(True)
    bpy.context.view_layer.objects.active = joined
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

    return {
        "outputSha256": sha256_file(output_path),
        "outputByteLength": output_path.stat().st_size,
        "sourceMeshObjectCount": len(mesh_objects),
        "outputMeshObjectCount": 1,
        "sourceMaterialSlotCount": len(source_materials),
        "outputMaterialSlotCount": len(joined.data.materials),
        "sourceImageDimensions": source_dimensions,
        "atlasDimensions": [atlas_size, atlas_size],
        "sourceUvBounds": {"min": uv_min, "max": uv_max},
        "sourcePolygonCounts": source_polygon_counts,
        "removedDegenerateTriangleCount": removed_degenerate_triangle_count,
        "triangleCount": len(mesh.polygons),
    }


def main() -> None:
    args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
    if len(args) != 4:
        fail(
            "usage: blender --background --python prepare-static-item-palette-atlas-v2.py "
            "-- source.glb mapping.json output.glb icon-output-dir"
        )
    source_path = Path(args[0]).resolve()
    mapping_path = Path(args[1]).resolve()
    output_path = Path(args[2]).resolve()
    icon_output = Path(args[3]).resolve()
    if not source_path.is_file():
        fail(f"source does not exist: {source_path}")
    if not mapping_path.is_file():
        fail(f"mapping specification does not exist: {mapping_path}")
    spec = json.loads(mapping_path.read_text(encoding="utf-8"))

    preview = render_icon_sources(source_path, icon_output, spec["materials"])
    atlas = build_atlas(source_path, output_path, spec)
    report = {
        "schemaVersion": 2,
        "sourceSha256": sha256_file(source_path),
        "mappingSha256": sha256_file(mapping_path),
        "mapping": spec,
        "preview": preview,
        "atlas": atlas,
    }
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
