// Export an editable owned GLB from the exact Creature rig. This is not a
// game-product exporter or admission bypass; inherited retail clips are omitted.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {Matrix4} from '../apps/studio-web/node_modules/three/build/three.module.js';
const [sourcePath,rigPath,authoringPath,destination]=process.argv.slice(2);
const bytes=await fs.readFile(sourcePath),rig=JSON.parse(await fs.readFile(rigPath,'utf8')),authoring=JSON.parse(await fs.readFile(authoringPath,'utf8'));
if(createHash('sha256').update(bytes).digest('hex')!==authoring.sourceSha256)throw Error('Wrong source SHA');
if(bytes.toString('ascii',0,4)!=='glTF'||bytes.readUInt32LE(4)!==2)throw Error('Expected GLB2');
const jn=bytes.readUInt32LE(12),doc=JSON.parse(bytes.subarray(20,20+jn).toString()),bin=bytes.subarray(28+jn);
if(doc.meshes.length!==1||doc.meshes[0].primitives.length!==1||rig.segments.length!==1||doc.skins?.length)throw Error('Expected one unskinned owned surface');
const prim=doc.meshes[0].primitives[0],seg=rig.segments[0];
function accessor(index){const a=doc.accessors[index],v=doc.bufferViews[a.bufferView];return {a,v,offset:(v.byteOffset||0)+(a.byteOffset||0)}}
const pos=accessor(prim.attributes.POSITION),ind=accessor(prim.indices);
if(pos.a.count!==seg.surfacePositions.length||pos.a.componentType!==5126||ind.a.count!==seg.surfaceIndices.length)throw Error('Surface count mismatch');
for(let i=0;i<pos.a.count;i++){const o=pos.offset+i*(pos.v.byteStride||12),p=[-bin.readFloatLE(o),bin.readFloatLE(o+8),bin.readFloatLE(o+4)];if(p.some((v,k)=>Math.abs(v-seg.surfacePositions[i][k])>2e-6))throw Error('Source/rig vertex order or frame mismatch')}
const width={5121:1,5123:2,5125:4}[ind.a.componentType];if(!width)throw Error('Index type');
for(let i=0;i<ind.a.count;i++)if(bin.readUIntLE(ind.offset+i*width,width)!==seg.surfaceIndices[i])throw Error('Topology mismatch');
let chunks=[bin],length=bin.length;
function append(data,type,componentType,count,target){const pad=(4-length%4)%4;if(pad){chunks.push(Buffer.alloc(pad));length+=pad}const bv=doc.bufferViews.length;doc.bufferViews.push({buffer:0,byteOffset:length,byteLength:data.length,...(target?{target}:{})});chunks.push(data);length+=data.length;const id=doc.accessors.length;doc.accessors.push({bufferView:bv,componentType,count,type});return id}
const matrix=new Matrix4().set(-1,0,0,0,0,0,1,0,0,1,0,0,0,0,0,1),worlds=new Map(),nodeBase=doc.nodes.length;
const idToJoint=new Map(rig.nodes.map((n,i)=>[n.id,i]));
for(const node of rig.nodes){const local=new Matrix4().fromArray(node.bindLocalMatrix),world=node.parentId===null?local.clone():worlds.get(node.parentId).clone().multiply(local);worlds.set(node.id,world);const converted=matrix.clone().multiply(local).multiply(matrix);doc.nodes.push({name:node.name,matrix:converted.elements,children:rig.nodes.filter(n=>n.parentId===node.id).map(n=>nodeBase+idToJoint.get(n.id))})}
const ib=Buffer.alloc(rig.nodes.length*64);
rig.nodes.forEach((n,i)=>matrix.clone().multiply(worlds.get(n.id)).multiply(matrix).invert().elements.forEach((v,k)=>ib.writeFloatLE(v,i*64+k*4)));
const joints=Buffer.alloc(pos.a.count*8),weights=Buffer.alloc(pos.a.count*16);
seg.referenceWeights.forEach((row,i)=>{if(row.length>4||!row.length||Math.abs(row.reduce((v,w)=>v+w.value,0)-1)>1e-5)throw Error('Invalid weights');row.forEach((w,k)=>{const id=idToJoint.get(w.boneNodeId);if(id===undefined)throw Error('Unknown joint');joints.writeUInt16LE(id,i*8+k*2);weights.writeFloatLE(w.value,i*16+k*4)})});
prim.attributes.JOINTS_0=append(joints,'VEC4',5123,pos.a.count,34962);prim.attributes.WEIGHTS_0=append(weights,'VEC4',5126,pos.a.count,34962);
doc.skins=[{name:'Creature authored c_wolf carriers',inverseBindMatrices:append(ib,'MAT4',5126,rig.nodes.length),joints:rig.nodes.map((_,i)=>nodeBase+i),skeleton:nodeBase}];
for(const node of doc.nodes.slice(0,nodeBase))if(node.mesh===0){if(node.matrix||node.translation||node.rotation||node.scale)throw Error('Transformed source not supported');node.skin=0}
const scene=doc.scenes[doc.scene||0];scene.nodes.push(...rig.nodes.filter(n=>n.parentId===null).map(n=>nodeBase+idToJoint.get(n.id)));
doc.asset.extras={...doc.asset.extras,m2aEditableCreatureV1:{sourceSha256:authoring.sourceSha256,rigSha256:rig.contentSha256,authoringSha256:authoring.contentSha256,supermodel:authoring.selectedSupermodelResref,status:'EDITABLE_DIAGNOSTIC_NOT_GAME_ADMITTED',retailAnimationsCopied:false}};
const padding=(4-length%4)%4;if(padding){chunks.push(Buffer.alloc(padding));length+=padding}doc.buffers[0].byteLength=length;
let json=Buffer.from(JSON.stringify(doc));json=Buffer.concat([json,Buffer.alloc((4-json.length%4)%4,32)]);const h=Buffer.alloc(20);h.write('glTF');h.writeUInt32LE(2,4);h.writeUInt32LE(28+json.length+length,8);h.writeUInt32LE(json.length,12);h.writeUInt32LE(0x4e4f534a,16);const bh=Buffer.alloc(8);bh.writeUInt32LE(length);bh.writeUInt32LE(0x004e4942,4);
const result=Buffer.concat([h,json,bh,...chunks]);await fs.writeFile(destination,result,{flag:'wx'});console.log(JSON.stringify({destination,sha256:createHash('sha256').update(result).digest('hex'),vertices:pos.a.count,triangles:ind.a.count/3,joints:rig.nodes.length,status:'EDITABLE_DIAGNOSTIC_NOT_GAME_ADMITTED'}));
