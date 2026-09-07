"""Generate editable Creature weight proposals; never writes MDL or edits the GLB.

Run Blender in background with --python this_file -- rig.json output.json.
The rig's surface and stable carrier IDs are the input/output contract. Blender
is only a weight-authoring helper. Studio must validate the proposal and motion.
API: https://docs.blender.org/api/current/bpy.ops.object.html
"""
import bpy
import json
import math
import sys
from pathlib import Path
from mathutils import Vector


def main():
    args = sys.argv[sys.argv.index("--") + 1:]
    source, destination = map(Path, args[:2])
    if destination.exists():
        raise RuntimeError("Output already exists")
    rig = json.loads(source.read_text(encoding="utf-8"))
    if len(rig["segments"]) != 1:
        raise RuntimeError("Expected one source segment")
    segment = rig["segments"][0]
    positions = segment["surfacePositions"]
    indices = segment["surfaceIndices"]
    allowed = set(segment["allowedBoneNodeIds"])
    # Exact-position welding affects the solver proxy only, never renderer data.
    unique, remap, by_position = [], [], {}
    for position in positions:
        key = tuple(position)
        if key not in by_position:
            by_position[key] = len(unique)
            unique.append(position)
        remap.append(by_position[key])
    faces = []
    for offset in range(0, len(indices), 3):
        triangle = tuple(remap[i] for i in indices[offset:offset + 3])
        if len(set(triangle)) == 3:
            faces.append(triangle)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    mesh = bpy.data.meshes.new("CreatureWeightProxy")
    mesh.from_pydata(unique, [], faces)
    mesh.update()
    obj = bpy.data.objects.new("CreatureWeightProxy", mesh)
    bpy.context.collection.objects.link(obj)
    from mathutils import Matrix
    worlds = {}
    for node in rig["nodes"]:
        flat = node["bindLocalMatrix"]
        local = Matrix([[flat[r + c * 4] for c in range(4)] for r in range(4)])
        parent = node["parentId"]
        worlds[node["id"]] = worlds[parent] @ local if parent is not None else local
    parent_world = worlds[segment["parentNodeId"]]
    if any(abs(parent_world[r][c] - (1 if r == c else 0)) > 1e-6
           for r in range(4) for c in range(4)):
        raise RuntimeError("Proxy expects an identity surface parent frame")
    pivots = {i: matrix.translation.copy() for i, matrix in worlds.items()}
    nodes = {node["id"]: node for node in rig["nodes"]}
    armature = bpy.data.armatures.new("CreatureWeightCarriers")
    arm = bpy.data.objects.new("CreatureWeightCarriers", armature)
    bpy.context.collection.objects.link(arm)
    bpy.context.view_layer.objects.active = arm
    arm.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    for part in sorted(allowed):
        bone = armature.edit_bones.new(str(part))
        bone.head = pivots[part]
        children = [p for p in allowed if nodes[p]["parentId"] == part]
        if children:
            tip = sum((pivots[p] for p in children), Vector()) / len(children)
        else:
            weighted = [(Vector(p), max((v["value"] for v in row
                         if v["boneNodeId"] == part), default=0.0))
                        for p, row in zip(positions, segment["referenceWeights"])]
            mass = sum(w for _, w in weighted)
            if mass <= 0:
                raise RuntimeError(f"Terminal carrier {part} has no authored surface")
            tip = sum((p * w for p, w in weighted), Vector()) / mass
        if (tip - bone.head).length < 1e-4:
            raise RuntimeError(f"Degenerate envelope for carrier {part}")
        bone.tail = tip
    for part in sorted(allowed):
        parent = nodes[part]["parentId"]
        while parent is not None and parent not in allowed:
            parent = nodes[parent]["parentId"]
        if parent is not None:
            armature.edit_bones[str(part)].parent = armature.edit_bones[str(parent)]
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.ops.object.select_all(action="DESELECT")
    obj.select_set(True)
    arm.select_set(True)
    bpy.context.view_layer.objects.active = arm
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    rows = []
    missing = []
    for vertex in mesh.vertices:
        influences = sorted([(int(obj.vertex_groups[g.group].name), g.weight)
                             for g in vertex.groups if g.weight > 1e-7],
                            key=lambda item: (-item[1], item[0]))[:4]
        mass = sum(w for _, w in influences)
        if mass <= 0:
            missing.append(vertex.index)
            rows.append([])
        else:
            rows.append([{"boneNodeId": part, "value": weight / mass}
                         for part, weight in influences])
    projected = []
    if len(missing) == len(rows):
        raise RuntimeError('Bone heat produced no weights; mesh requires another explicit authoring method')
    if missing:
        from mathutils.kdtree import KDTree
        tree = KDTree(len(rows) - len(missing))
        for index, row in enumerate(rows):
            if row:
                tree.insert(Vector(unique[index]), index)
        tree.balance()
        diagonal = (Vector([max(p[a] for p in unique) for a in range(3)]) -
                    Vector([min(p[a] for p in unique) for a in range(3)])).length
        for index in missing:
            _, neighbor, distance = tree.find(Vector(unique[index]))
            if distance > diagonal * 0.02:
                raise RuntimeError(f"Unweighted vertex {index} is too far from weighted surface: {distance}")
            rows[index] = rows[neighbor]
            projected.append({"proxyVertex": index, "fromProxyVertex": neighbor,
                              "distanceFraction": distance / diagonal})
    output = {
        "schemaVersion": 1,
        "algorithm": "BLENDER_BONE_HEAT_EDITABLE_PROPOSAL",
        "baseRigSha256": rig["contentSha256"],
        "segmentId": segment["id"],
        "rendererVertexCount": len(positions),
        "proxyVertexCount": len(unique),
        "auxiliaryProjection": projected,
        "referenceWeights": [rows[index] for index in remap],
        "admission": "NOT_VALIDATED",
    }
    destination.write_text(json.dumps(output, separators=(",", ":")), encoding="utf-8")
    print(json.dumps({key: value for key, value in output.items() if key != "referenceWeights"}))


main()
