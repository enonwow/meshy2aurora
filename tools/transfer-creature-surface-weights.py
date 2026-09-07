"""Transfer an owned, authored weight field to an owned LOD surface. Not validation."""
import bpy, json, sys, math
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
source_path,target_path,out_path=map(Path,sys.argv[sys.argv.index('--')+1:][:3])
if out_path.exists(): raise RuntimeError('Destination exists')
source=json.loads(source_path.read_text());target=json.loads(target_path.read_text())
if source['nodes']!=target['nodes']: raise RuntimeError('Rig carrier mismatch')
a,b=source['segments'][0],target['segments'][0]
points=[Vector(p) for p in a['surfacePositions']]
faces=[tuple(a['surfaceIndices'][i:i+3]) for i in range(0,len(a['surfaceIndices']),3)]
bvh=BVHTree.FromPolygons(points,faces,all_triangles=True)
diagonal=math.sqrt(sum((max(p[k] for p in points)-min(p[k] for p in points))**2 for k in range(3)))
rows=[];max_distance=0
for position in b['surfacePositions']:
    hit,normal,index,distance=bvh.find_nearest(Vector(position))
    if index is None or distance>diagonal*0.015: raise RuntimeError('LOD outside source surface tolerance')
    max_distance=max(max_distance,distance)
    ids=faces[index];p,q,r=[points[i] for i in ids]
    v0,v1,v2=q-p,r-p,hit-p
    d00,d01,d11,d20,d21=v0.dot(v0),v0.dot(v1),v1.dot(v1),v2.dot(v0),v2.dot(v1)
    denominator=d00*d11-d01*d01
    if abs(denominator)<1e-22:
        closest=min(range(3),key=lambda i:(points[ids[i]]-hit).length_squared)
        factors=[1.0 if i==closest else 0.0 for i in range(3)]
    else:
        v=(d11*d20-d01*d21)/denominator;w=(d00*d21-d01*d20)/denominator
        factors=[max(0.0,min(1.0,x)) for x in [1-v-w,v,w]]
    influences={}
    for vertex,factor in zip(ids,factors):
        for item in a['referenceWeights'][vertex]:
            bone=item['boneNodeId'];influences[bone]=influences.get(bone,0)+factor*item['value']
    ordered=sorted(influences.items(),key=lambda x:(-x[1],x[0]))[:4];mass=sum(w for _,w in ordered)
    if mass<=0: raise RuntimeError('Empty interpolation')
    rows.append([{'boneNodeId':bone,'value':w/mass} for bone,w in ordered if w>0])
out={'baseRigSha256':target['contentSha256'],'sourceRigSha256':source['contentSha256'],'algorithm':'BARYCENTRIC_AUTHORED_SURFACE_TRANSFER_V1','referenceWeights':rows,'maxDistance':max_distance,'admission':'NOT_VALIDATED'}
out_path.write_text(json.dumps(out));print(json.dumps({'vertices':len(rows),'maxDistance':max_distance,'diagonal':diagonal}))
