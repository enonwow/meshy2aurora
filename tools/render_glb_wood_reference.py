"""Render a deterministic side-view reference of one GLB in Blender.

Usage:
  blender --background --python tools/render_glb_wood_reference.py -- input.glb output.png
"""

from __future__ import annotations

import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector


def arguments() -> tuple[Path, Path]:
    try:
        separator = sys.argv.index("--")
        source, output = sys.argv[separator + 1 :]
    except (ValueError, IndexError, TypeError):
        raise SystemExit("expected: -- <input.glb> <output.png>")
    return Path(source).resolve(), Path(output).resolve()


def look_at(camera: bpy.types.Object, target: Vector) -> None:
    camera.rotation_euler = (target - camera.location).to_track_quat("-Z", "Y").to_euler()


def add_area_light(name: str, location: Vector, energy: float, size: float, target: Vector) -> None:
    light = bpy.data.lights.new(name=name, type="AREA")
    light.energy = energy
    light.shape = "DISK"
    light.size = size
    obj = bpy.data.objects.new(name, light)
    bpy.context.collection.objects.link(obj)
    obj.location = location
    obj.rotation_euler = (target - location).to_track_quat("-Z", "Y").to_euler()


def main() -> None:
    source, output = arguments()
    if not source.is_file():
        raise SystemExit(f"missing input: {source}")
    if output.exists():
        raise SystemExit(f"refusing to overwrite: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(source), merge_vertices=False)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise SystemExit("imported GLB has no mesh objects")
    for material in bpy.data.materials:
        if not material.use_nodes:
            continue
        for node in material.node_tree.nodes:
            if node.type != "BSDF_PRINCIPLED":
                continue
            emission_strength = node.inputs.get("Emission Strength")
            if emission_strength is not None:
                emission_strength.default_value = 0.0
            emission_color = node.inputs.get("Emission Color")
            if emission_color is not None:
                emission_color.default_value = (0.0, 0.0, 0.0, 1.0)
            for input_name, default_value in (
                ("Normal", (0.0, 0.0, 0.0)),
                ("Metallic", 0.0),
                ("Roughness", 0.8),
            ):
                shader_input = node.inputs.get(input_name)
                if shader_input is None:
                    continue
                for link in list(shader_input.links):
                    material.node_tree.links.remove(link)
                shader_input.default_value = default_value

    corners = [obj.matrix_world @ Vector(corner) for obj in meshes for corner in obj.bound_box]
    minimum = Vector((min(v.x for v in corners), min(v.y for v in corners), min(v.z for v in corners)))
    maximum = Vector((max(v.x for v in corners), max(v.y for v in corners), max(v.z for v in corners)))
    center = (minimum + maximum) * 0.5
    span = maximum - minimum

    camera_data = bpy.data.cameras.new("ComparisonCamera")
    camera = bpy.data.objects.new("ComparisonCamera", camera_data)
    bpy.context.collection.objects.link(camera)
    distance = max(span.y * 3.0, span.x * 0.8, span.z * 0.8, 2.0)
    camera.location = Vector((center.x, minimum.y - distance, center.z))
    look_at(camera, center)
    camera_data.type = "ORTHO"
    aspect = 1400.0 / 900.0
    camera_data.ortho_scale = max(span.z * 1.14, span.x / aspect * 1.14)
    camera_data.lens = 50.0
    bpy.context.scene.camera = camera

    add_area_light(
        "Key",
        Vector((center.x - span.x * 0.35, minimum.y - distance * 0.7, maximum.z + span.z * 0.7)),
        220.0,
        max(span.x, span.z) * 0.9,
        center,
    )
    add_area_light(
        "Fill",
        Vector((center.x + span.x * 0.5, maximum.y + distance * 0.35, center.z + span.z * 0.2)),
        90.0,
        max(span.x, span.z) * 0.7,
        center,
    )

    world = bpy.data.worlds.new("ComparisonWorld")
    world.use_nodes = True
    background = world.node_tree.nodes.get("Background")
    background.inputs["Color"].default_value = (0.010, 0.013, 0.015, 1.0)
    background.inputs["Strength"].default_value = 0.12
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
    scene.render.resolution_percentage = 100
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.render.image_settings.color_depth = "8"
    scene.render.image_settings.compression = 15
    scene.render.engine = "BLENDER_EEVEE"
    bpy.ops.render.render(write_still=True)

    triangles = 0
    for obj in meshes:
        evaluated = obj.evaluated_get(bpy.context.evaluated_depsgraph_get())
        mesh = evaluated.to_mesh()
        mesh.calc_loop_triangles()
        triangles += len(mesh.loop_triangles)
        evaluated.to_mesh_clear()
    print(
        f"rendered={output} meshes={len(meshes)} triangles={triangles} "
        f"bounds=({span.x:.6f},{span.y:.6f},{span.z:.6f})"
    )


if __name__ == "__main__":
    main()
