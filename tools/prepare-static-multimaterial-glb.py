import hashlib
import json
import math
import sys
from pathlib import Path

import bmesh
import bpy


def fail(message: str) -> None:
    raise RuntimeError(message)


args = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else []
if len(args) != 2:
    fail("usage: blender --background --python prepare-static-multimaterial-glb.py -- source.glb output.glb")

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
if len(mesh_objects) < 2:
    fail(f"expected at least two source mesh objects, got {len(mesh_objects)}")
if any(candidate.parent is not None and candidate.parent.type == "MESH" for candidate in mesh_objects):
    fail("nested mesh-object hierarchy is outside the static material normalization contract")

bpy.ops.object.select_all(action="DESELECT")
for candidate in mesh_objects:
    candidate.select_set(True)
bpy.context.view_layer.objects.active = mesh_objects[0]
result = bpy.ops.object.join()
if "FINISHED" not in result:
    fail("Blender mesh join did not finish")
joined = bpy.context.view_layer.objects.active
joined.name = "bronze_mask_split_static"

mesh = joined.data
bm = bmesh.new()
bm.from_mesh(mesh)
bm.faces.ensure_lookup_table()
degenerate_faces = [
    face
    for face in bm.faces
    if not math.isfinite(face.calc_area()) or face.calc_area() <= 1.0e-12
]
removed_degenerate_triangle_count = len(degenerate_faces)
if removed_degenerate_triangle_count != 2:
    fail(
        "expected exact two degenerate source triangles, got "
        f"{removed_degenerate_triangle_count}"
    )
bmesh.ops.delete(bm, geom=degenerate_faces, context="FACES")
bm.to_mesh(mesh)
bm.free()
mesh.update(calc_edges=True)

material_names = [slot.material.name if slot.material else None for slot in joined.material_slots]
if len(material_names) != 2:
    fail(f"expected exact two joined material slots, got {len(material_names)}")

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
    "sourceMeshObjectCount": len(mesh_objects),
    "outputMeshObjectCount": 1,
    "materialSlotCount": len(material_names),
    "materialNames": material_names,
    "removedDegenerateTriangleCount": removed_degenerate_triangle_count,
}
print(json.dumps(report, indent=2))
