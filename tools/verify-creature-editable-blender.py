"""Read back the owned skinned GLB, verify bind and joint response, save editable scene."""
import bpy,json,sys,math
from pathlib import Path
from mathutils import Vector,Quaternion
from mathutils.kdtree import KDTree
source,rig_path,out_dir=map(Path,sys.argv[sys.argv.index('--')+1:][:3]);rig=json.loads(rig_path.read_text())
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
bpy.ops.import_scene.gltf(filepath=str(source))
scene=bpy.context.scene
arms=[o for o in scene.objects if o.type=='ARMATURE'];meshes=[o for o in scene.objects if o.type=='MESH' and any(m.type=='ARMATURE' for m in o.modifiers)]
if len(meshes)!=1 or len(arms)!=1: raise RuntimeError('Expected one mesh and armature: '+str([(o.name,o.type) for o in scene.objects]))
mesh,arm=meshes[0],arms[0]
if len(arm.data.bones)!=len(rig['nodes']): raise RuntimeError('Carrier count mismatch')
def positions():
 bpy.context.view_layer.update();obj=mesh.evaluated_get(bpy.context.evaluated_depsgraph_get());return [obj.matrix_world@v.co for v in obj.data.vertices]
rest=positions();tree=KDTree(len(rest))
for i,p in enumerate(rest):tree.insert(p,i)
tree.balance()
# Aurora (-gltfX,gltfZ,gltfY) -> Blender (gltfX,-gltfZ,gltfY).
max_bind=max(tree.find(Vector((-p[0],-p[1],p[2])))[2] for p in rig['segments'][0]['surfacePositions'])
if max_bind>1e-5: raise RuntimeError('Bind readback mismatch')
probes=[]
for name in ['Wolf_Lfrontpaw','Wolf_Rfrontpaw','Wolf_Lbackpaw','Wolf_Rbackpaw','Wolf_tail','Wolf_tailend','Wolf_head']:
 bone=arm.pose.bones.get(name)
 if bone is None: raise RuntimeError('Missing joint '+name)
 original=bone.matrix_basis.copy();bone.rotation_mode='QUATERNION';bone.rotation_quaternion=Quaternion((1,0,0),0.2)
 posed=positions();distances=[(a-b).length for a,b in zip(rest,posed)]
 if max(distances)<1e-5: raise RuntimeError('No surface motion '+name)
 probes.append({'joint':name,'changedVertices':sum(d>1e-5 for d in distances),'maxDisplacement':max(distances)})
 bone.matrix_basis=original
bpy.context.view_layer.update()
scene['creature_status']='EDITABLE_DIAGNOSTIC_NOT_GAME_ADMITTED';scene['supermodel']='c_wolf';scene['retail_animations_copied']=False
scene.render.engine='CYCLES';scene.cycles.samples=32
scene.render.resolution_x=1280;scene.render.resolution_y=900;scene.render.resolution_percentage=100
scene.world.color=(0.12,0.12,0.12)
for name,location,energy,size in [('Key',(3,-3,4),500,4),('Fill',(-3,-1,2.5),300,3),('Rim',(0,3,3),400,3)]:
 data=bpy.data.lights.new(name,'AREA');data.energy=energy;data.shape='DISK';data.size=size;obj=bpy.data.objects.new(name,data);scene.collection.objects.link(obj);obj.location=location;obj.rotation_euler=(Vector((0,0,0.5))-obj.location).to_track_quat('-Z','Y').to_euler()
data=bpy.data.cameras.new('Review camera');camera=bpy.data.objects.new('Review camera',data);scene.collection.objects.link(camera);camera.location=(3,-2.8,1.65);target=Vector((0,0.08,0.55));camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler();data.type='ORTHO';data.ortho_scale=2.15;scene.camera=camera
scene.render.image_settings.file_format='PNG';scene.render.filepath=str(out_dir/'borzoi-cwolf-preview.png')
bpy.ops.file.pack_all();bpy.ops.wm.save_as_mainfile(filepath=str(out_dir/'borzoi-cwolf-rigged.blend'))
(out_dir/'editable-readback.json').write_text(json.dumps({'bindMaxError':max_bind,'carrierCount':len(arm.data.bones),'meshVertices':len(rest),'triangleCount':len(mesh.data.polygons),'probes':probes,'nativeRuntime':'NOT_TESTED','status':'EDITABLE_DIAGNOSTIC_NOT_GAME_ADMITTED'}))
bpy.ops.render.render(write_still=True)
print('EDITABLE_MODEL_VERIFIED')
