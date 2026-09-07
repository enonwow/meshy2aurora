"""Render a deterministic front view of a GLB with an override base-colour image.

Usage:
  blender --background --python tools/render_glb_texture_override.py -- \
    input.glb override.png output.png
"""

from __future__ import annotations

import sys
from pathlib import Path

import bpy
from mathutils import Vector


def arguments() -> tuple[Path, Path, Path, str]:
    try:
        separator = sys.argv.index("--")
        values = sys.argv[separator + 1 :]
        if len(values) not in (3, 4):
            raise ValueError
        source, texture, output = values[:3]
        view = values[3] if len(values) == 4 else "front"
    except (ValueError, IndexError, TypeError):
        raise SystemExit("expected: -- <input.glb> <override.png> <output.png> [front|back|left|right]")
    if view not in {"front", "back", "left", "right"}:
        raise SystemExit(f"unsupported view: {view}")
    return Path(source).resolve(), Path(texture).resolve(), Path(output).resolve(), view


def look_at(camera: bpy.types.Object, target: Vector) -> None:
    camera.rotation_euler = (target - camera.location).to_track_quat("-Z", "Y").to_euler()


def main() -> None:
    source, texture, output, view = arguments()
    for path in (source, texture):
        if not path.is_file():
            raise SystemExit(f"missing input: {path}")
    if output.exists():
        raise SystemExit(f"refusing to overwrite: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(source), merge_vertices=False)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise SystemExit("imported GLB has no mesh objects")

    override = bpy.data.images.load(str(texture), check_existing=False)
    for material in bpy.data.materials:
        if not material.use_nodes:
            continue
        nodes = material.node_tree.nodes
        shader = next((node for node in nodes if node.type == "BSDF_PRINCIPLED"), None)
        if shader is None:
            continue
        image_node = next((node for node in nodes if node.type == "TEX_IMAGE"), None)
        if image_node is None:
            image_node = nodes.new("ShaderNodeTexImage")
        image_node.image = override
        base_color = shader.inputs.get("Base Color")
        for link in list(base_color.links):
            material.node_tree.links.remove(link)
        material.node_tree.links.new(image_node.outputs["Color"], base_color)
        emission_color = shader.inputs.get("Emission Color")
        if emission_color is not None:
            for link in list(emission_color.links):
                material.node_tree.links.remove(link)
            material.node_tree.links.new(image_node.outputs["Color"], emission_color)
        emission_strength = shader.inputs.get("Emission Strength")
        if emission_strength is not None:
            emission_strength.default_value = 1.0
        shader.inputs["Metallic"].default_value = 0.0
        shader.inputs["Roughness"].default_value = 1.0

    corners = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
    minimum = Vector((min(v.x for v in corners), min(v.y for v in corners), min(v.z for v in corners)))
    maximum = Vector((max(v.x for v in corners), max(v.y for v in corners), max(v.z for v in corners)))
    center = (minimum + maximum) * 0.5
    span = maximum - minimum

    camera_data = bpy.data.cameras.new("SemanticMaskCamera")
    camera = bpy.data.objects.new("SemanticMaskCamera", camera_data)
    bpy.context.collection.objects.link(camera)
    distance = max(span.y * 3.0, span.x * 0.8, span.z * 0.8, 2.0)
    camera.location = {
        "front": Vector((center.x, minimum.y - distance, center.z)),
        "back": Vector((center.x, maximum.y + distance, center.z)),
        "left": Vector((minimum.x - distance, center.y, center.z)),
        "right": Vector((maximum.x + distance, center.y, center.z)),
    }[view]
    look_at(camera, center)
    camera_data.type = "ORTHO"
    horizontal_span = span.x if view in {"front", "back"} else span.y
    camera_data.ortho_scale = max(span.z * 1.14, horizontal_span / (1400.0 / 900.0) * 1.14)
    bpy.context.scene.camera = camera

    world = bpy.data.worlds.new("SemanticMaskWorld")
    world.use_nodes = True
    background = world.node_tree.nodes.get("Background")
    background.inputs["Color"].default_value = (0.015, 0.015, 0.015, 1.0)
    background.inputs["Strength"].default_value = 0.2
    bpy.context.scene.world = world

    scene = bpy.context.scene
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x = 1400
    scene.render.resolution_y = 900
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.film_transparent = False
    scene.render.filepath = str(output)
    scene.view_settings.look = "None"

    bpy.ops.render.render(write_still=True)
    print(f"rendered={output} texture={texture} view={view}")


if __name__ == "__main__":
    main()
