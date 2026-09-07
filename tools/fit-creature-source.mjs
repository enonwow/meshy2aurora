import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
// Bake an explicitly reviewed affine source-object fit; preserve topology,
// UVs and embedded image bytes. This tool never creates an Aurora model.
const [input,specPath,output]=process.argv.slice(2);
assert(input&&specPath&&output,'Usage: node tools/fit-creature-source.mjs input.glb fit.json output.glb');
const bytes=await fs.readFile(input), spec=JSON.parse(await fs.readFile(specPath,'utf8'));
const sha=b=>createHash('sha256').update(b).digest('hex');
assert.equal(sha(bytes),spec.originalSourceSha256,'Source identity mismatch');
assert.equal(bytes.readUInt32LE(0),0x46546c67);assert.equal(bytes.readUInt32LE(4),2);
const jsonLen=bytes.readUInt32LE(12),doc=JSON.parse(bytes.subarray(20,20+jsonLen).toString());
const binOffset=20+jsonLen+8,bin=Buffer.from(bytes.subarray(binOffset));
assert.equal(doc.nodes.length,1);assert.equal(doc.meshes.length,1);assert.equal(doc.meshes[0].primitives.length,1);assert(!doc.skins?.length&&!doc.animations?.length);
const matrix=spec.fitMatrixAurora;assert.equal(matrix.length,16);assert(matrix.every(Number.isFinite));
// V1 deliberately supports positive axis scales and translation only.
assert([0,5,10].every(i=>matrix[i]>0));assert([1,2,3,4,6,7,8,9,11].every(i=>matrix[i]===0));assert.equal(matrix[15],1);
const primitive=doc.meshes[0].primitives[0];assert.equal(primitive.mode??4,4);assert(primitive.attributes.TANGENT===undefined,'Tangent-frame baking requires an explicit extension');
function transformAccessor(id,isNormal){const a=doc.accessors[id],view=doc.bufferViews[a.bufferView];assert.equal(a.componentType,5126);assert.equal(a.type,'VEC3');assert(!a.sparse);const min=[Infinity,Infinity,Infinity],max=[-Infinity,-Infinity,-Infinity];for(let i=0;i<a.count;i++){const offset=(view.byteOffset??0)+(a.byteOffset??0)+i*(view.byteStride??12);const p=[0,1,2].map(k=>bin.readFloatLE(offset+k*4));const aurora=[-p[0],p[2],p[1]];let q;if(isNormal){q=[aurora[0]/matrix[0],aurora[1]/matrix[5],aurora[2]/matrix[10]];const length=Math.hypot(...q);assert(length>1e-12);q=q.map(x=>x/length);}else{q=[aurora[0]*matrix[0]+matrix[12],aurora[1]*matrix[5]+matrix[13],aurora[2]*matrix[10]+matrix[14]];}const out=[-q[0],q[2],q[1]];for(let k=0;k<3;k++){bin.writeFloatLE(out[k],offset+k*4);min[k]=Math.min(min[k],out[k]);max[k]=Math.max(max[k],out[k]);}}if(!isNormal){a.min=min;a.max=max;}}
transformAccessor(primitive.attributes.POSITION,false);if(primitive.attributes.NORMAL!==undefined)transformAccessor(primitive.attributes.NORMAL,true);
for(const key of ['translation','rotation','scale','matrix'])delete doc.nodes[0][key];
doc.asset.extras={...doc.asset.extras,m2aReferenceBindV1:spec};
const json=Buffer.from(JSON.stringify(doc));const padded=Buffer.alloc(Math.ceil(json.length/4)*4,0x20);json.copy(padded);const header=Buffer.alloc(20);header.writeUInt32LE(0x46546c67);header.writeUInt32LE(2,4);header.writeUInt32LE(20+padded.length+8+bin.length,8);header.writeUInt32LE(padded.length,12);header.writeUInt32LE(0x4e4f534a,16);const bh=Buffer.alloc(8);bh.writeUInt32LE(bin.length);bh.writeUInt32LE(0x004e4942,4);const result=Buffer.concat([header,padded,bh,bin]);await fs.writeFile(output,result,{flag:'wx'});
console.log(JSON.stringify({sourceSha256:sha(bytes),output,sha256:sha(result),byteLength:result.length,triangles:doc.accessors[primitive.indices].count/3,topologyAndUvsPreserved:true,textureBytesPreserved:true,registrationOnly:true}));
