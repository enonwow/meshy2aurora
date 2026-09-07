"""Create an experimental owned mesh LOD; external topology validation is required."""
import bpy
import json
import sys
import struct
import hashlib
from pathlib import Path

args=sys.argv[sys.argv.index('--')+1:]
source,destination,report=map(Path,args[:3])
if destination.exists(): raise RuntimeError('Destination already exists')
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
bpy.ops.import_scene.gltf(filepath=str(source))
meshes=[o for o in bpy.context.scene.objects if o.type=='MESH']
before=sum(len(o.data.polygons) for o in meshes)
records=[]
for obj in meshes:
    bpy.context.view_layer.objects.active=obj
    obj.select_set(True)
    modifier=obj.modifiers.new('Game mesh LOD','DECIMATE')
    modifier.ratio=0.35
    modifier.use_collapse_triangulate=True
    modifier.delimit={'UV','MATERIAL','SEAM'}
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    records.append({'mesh':obj.name,'vertices':len(obj.data.vertices),'triangles':len(obj.data.polygons)})
bpy.ops.export_scene.gltf(filepath=str(destination),export_format='GLB',export_extras=True,export_animations=False,export_image_format='AUTO')
# Preserve the explicitly fitted coordinate-frame provenance. Exporting through
# Blender does not retain asset-level glTF extras automatically.
original_bytes=source.read_bytes()
original_json_size=struct.unpack_from('<I',original_bytes,12)[0]
original_json=json.loads(original_bytes[20:20+original_json_size])
exported=destination.read_bytes()
json_size=struct.unpack_from('<I',exported,12)[0]
document=json.loads(exported[20:20+json_size])
extras=dict(original_json.get('asset',{}).get('extras',{}))
extras['m2aOwnedLodV1']={'sourceSha256':hashlib.sha256(original_bytes).hexdigest(),'algorithm':'BLENDER_DECIMATE_UV_DELIMITED','ratio':0.35}
document['asset']['extras']=extras
json_bytes=json.dumps(document,separators=(',',':')).encode('utf8')
json_bytes+=b' '*((-len(json_bytes))%4)
remaining_chunks=exported[20+json_size:]
result=struct.pack('<4sIIII',b'glTF',2,20+len(json_bytes)+len(remaining_chunks),len(json_bytes),0x4e4f534a)+json_bytes+remaining_chunks
destination.write_bytes(result)
report.write_text(json.dumps({'operation':'UV-preserving quadric mesh simplification','ratio':0.35,'beforeTriangles':before,'meshes':records,'status':'GEOMETRY_PROPOSAL_NOT_VALIDATED'}),encoding='utf8')
print('GAME_LOD_PROPOSAL_WRITTEN')
