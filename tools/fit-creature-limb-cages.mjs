// Explicit limb registration and joint-centered weights on an owned source.
// This authors a proposal; independent Creature validation remains mandatory.
import fs from 'node:fs/promises';import assert from 'node:assert/strict';import {createHash} from 'node:crypto';
import {Matrix4} from '../apps/studio-web/node_modules/three/build/three.module.js';
const [sourcePath,rigPath,recipePath,outPath,proposalPath]=process.argv.slice(2);
const raw=await fs.readFile(sourcePath),rig=JSON.parse(await fs.readFile(rigPath,'utf8')),recipe=JSON.parse(await fs.readFile(recipePath,'utf8'));
const sha=b=>createHash('sha256').update(b).digest('hex');assert.equal(sha(raw),recipe.sourceSha256);
assert.equal(rig.segments.length,1);const s=rig.segments[0],worlds=new Map();for(const n of rig.nodes){const m=new Matrix4().fromArray(n.bindLocalMatrix);worlds.set(n.id,n.parentId===null?m:worlds.get(n.parentId).clone().multiply(m))}
const pivot=id=>worlds.get(id).elements.slice(12,15);const smooth=t=>{t=Math.min(1,Math.max(0,t));return t*t*(3-2*t)};
function crossSection(z,limb){const hits=new Map();for(let f=0;f<s.surfaceIndices.length;f+=3)for(const [a,b] of [[0,1],[1,2],[2,0]]){const p=s.surfacePositions[s.surfaceIndices[f+a]],q=s.surfacePositions[s.surfaceIndices[f+b]];if((p[2]<z)===(q[2]<z))continue;const t=(z-p[2])/(q[2]-p[2]),h=p.map((v,k)=>v+t*(q[k]-v));
 const mass=id=>s.referenceWeights[id].filter(w=>limb.nodes.includes(w.boneNodeId)).reduce((v,w)=>v+w.value,0);
 if(z>0.4 && mass(s.surfaceIndices[f+a])*(1-t)+mass(s.surfaceIndices[f+b])*t<0.6)continue;
 hits.set(h.map(v=>Math.round(v*1e7)).join(','),h)}return [...hits.values()]}
const percentile=(a,t)=>a.sort((a,b)=>a-b)[Math.round((a.length-1)*t)];
const limbs=recipe.limbs.map(l=>{
 const centers=recipe.sampleHeights.map(z=>{const ps=crossSection(z,l).filter(p=>p[0]*l.side>recipe.centerExclusion&&p[1]*l.end>recipe.longitudinalExclusion);assert(ps.length>=6,'Insufficient limb section');return {z,xy:[0,1].map(k=>(percentile(ps.map(p=>p[k]),0.15)+percentile(ps.map(p=>p[k]),0.85))/2),n:ps.length}});
 return {...l,centers,joints:l.nodes.map(pivot)};
});
function interpolate(z,points){if(z<=points[0].z)return points[0].xy;if(z>=points.at(-1).z)return points.at(-1).xy;let i=1;while(points[i].z<z)i++;const a=points[i-1],b=points[i],t=(z-a.z)/(b.z-a.z);return a.xy.map((v,k)=>v*(1-t)+b.xy[k]*t)}
function offset(l,z){const source=interpolate(z,l.centers),target=interpolate(z,l.joints.map(p=>({z:p[2],xy:p.slice(0,2)})).sort((a,b)=>a.z-b.z)),fade=1-smooth((z-recipe.fullFitHeight)/(recipe.fadeHeight-recipe.fullFitHeight));return target.map((v,k)=>(v-source[k])*fade)}
const transformed=[],normalDerivatives=[],rows=[],changes=[];
for(let i=0;i<s.surfacePositions.length;i++){
 const p=s.surfacePositions[i],old=s.referenceWeights[i];
 const l=limbs.find(l=>p[0]*l.side>0&&p[1]*l.end>0);
 let q=p.slice(),row=old,derivative=[0,0];
 if(l&&p[2]<recipe.fadeHeight){
  const mass=old.filter(w=>l.nodes.includes(w.boneNodeId)).reduce((v,w)=>v+w.value,0);
  const sourceCenter=interpolate(p[2],l.centers),radial=Math.hypot(p[0]-sourceCenter[0],p[1]-sourceCenter[1]);
  const radialMask=p[2]<.12?1:1-smooth((radial-.06)/.06),heightBlend=smooth((p[2]-.30)/.15);
  const membership=radialMask*(1-heightBlend)+smooth((mass-.4)/.4)*heightBlend;
  const d=offset(l,p[2]);q[0]+=membership*d[0];q[1]+=membership*d[1];
  const a=offset(l,p[2]-.0001),b=offset(l,p[2]+.0001);derivative=a.map((v,k)=>membership*(b[k]-v)/.0002);
  const local=new Map([[l.nodes[0],1]]);
  for(let j=1;j<l.nodes.length;j++){
   const center=l.joints[j].slice();
   const incoming=center.map((v,k)=>v-l.joints[j-1][k]),above=Math.hypot(...incoming);incoming.forEach((v,k)=>incoming[k]=v/above);
   const outgoing=j+1<l.nodes.length?l.joints[j+1].map((v,k)=>v-center[k]):[0,1,0];
   const below=j+1<l.nodes.length?Math.hypot(...outgoing):above*.65,ol=Math.hypot(...outgoing);outgoing.forEach((v,k)=>outgoing[k]=v/ol);
   if(j===l.nodes.length-1)center.forEach((v,k)=>center[k]=v-incoming[k]*above*.15);
   const direction=incoming.map((v,k)=>v+outgoing[k]),dl=Math.hypot(...direction);assert(dl>1e-4,'Folded bind chain');direction.forEach((v,k)=>direction[k]=v/dl);
   const width=Math.min(above,below)*recipe.jointBlendFraction;
   const signed=q.reduce((v,x,k)=>v+(x-center[k])*direction[k],0);
   const t=smooth((signed+width)/(2*width));
   for(const [id,w] of local)local.set(id,w*(1-t));local.set(l.nodes[j],t);
  }
  const blend=(1-smooth((p[2]-recipe.fullWeightHeight)/(recipe.fadeWeightHeight-recipe.fullWeightHeight)))*membership;
  const mix=new Map(old.map(w=>[w.boneNodeId,w.value*(1-blend)]));for(const [id,w] of local)mix.set(id,(mix.get(id)||0)+blend*w);
  const ordered=[...mix].filter(([,w])=>w>1e-7).sort((a,b)=>b[1]-a[1]);const cutoff=ordered[4]?.[1]||0,top=ordered.slice(0,4).map(([id,w])=>[id,w-cutoff]).filter(([,w])=>w>1e-7),total=top.reduce((v,[,w])=>v+w,0);row=top.map(([boneNodeId,w])=>({boneNodeId,value:w/total}));
  if(Math.hypot(q[0]-p[0],q[1]-p[1])>1e-6)changes.push(i);
 }
 transformed.push(q);normalDerivatives.push(derivative);rows.push(row);
}
const jn=raw.readUInt32LE(12),doc=JSON.parse(raw.subarray(20,20+jn).toString()),bin=Buffer.from(raw.subarray(28+jn)),pr=doc.meshes[0].primitives[0];
const originalAccessor=doc.accessors[pr.attributes.POSITION],originalView=doc.bufferViews[originalAccessor.bufferView];assert.equal(originalAccessor.count,s.surfacePositions.length);for(let i=0;i<originalAccessor.count;i++){const o=(originalView.byteOffset||0)+(originalAccessor.byteOffset||0)+i*(originalView.byteStride||12),p=[-bin.readFloatLE(o),bin.readFloatLE(o+8),bin.readFloatLE(o+4)];assert(Math.hypot(...p.map((v,k)=>v-s.surfacePositions[i][k]))<1e-5,'Rig/source position mismatch')}
for(const [attribute,isNormal] of [['POSITION',false],['NORMAL',true]]){
 const a=doc.accessors[pr.attributes[attribute]],v=doc.bufferViews[a.bufferView];assert.equal(a.componentType,5126);assert.equal(a.count,transformed.length);const min=[Infinity,Infinity,Infinity],max=[-Infinity,-Infinity,-Infinity];
 for(let i=0;i<a.count;i++){const o=(v.byteOffset||0)+(a.byteOffset||0)+i*(v.byteStride||12);let p=transformed[i];if(isNormal){const n=[-bin.readFloatLE(o),bin.readFloatLE(o+8),bin.readFloatLE(o+4)],d=normalDerivatives[i];p=[n[0],n[1],n[2]-d[0]*n[0]-d[1]*n[1]];const length=Math.hypot(...p);p=p.map(v=>v/length)}const q=[-p[0],p[2],p[1]];q.forEach((n,k)=>{bin.writeFloatLE(n,o+k*4);min[k]=Math.min(min[k],n);max[k]=Math.max(max[k],n)})}
 if(!isNormal){a.min=min;a.max=max}
}
doc.asset.extras.m2aLimbCageV1={recipe,baseRigSha256:rig.contentSha256,algorithm:'PIECEWISE_LIMB_CENTERLINE_REGISTRATION_V1'};
let j=Buffer.from(JSON.stringify(doc));j=Buffer.concat([j,Buffer.alloc((4-j.length%4)%4,32)]);const h=Buffer.alloc(20);h.write('glTF');h.writeUInt32LE(2,4);h.writeUInt32LE(28+j.length+bin.length,8);h.writeUInt32LE(j.length,12);h.writeUInt32LE(0x4e4f534a,16);const bh=Buffer.alloc(8);bh.writeUInt32LE(bin.length);bh.writeUInt32LE(0x004e4942,4);const out=Buffer.concat([h,j,bh,bin]);const existing=await fs.readFile(outPath).catch(e=>{if(e.code==='ENOENT')return null;throw e});if(existing)assert.equal(sha(existing),sha(out),'Existing output differs');else await fs.writeFile(outPath,out,{flag:'wx'});
await fs.writeFile(proposalPath,JSON.stringify({algorithm:'JOINT_CENTERED_LIMB_AUTHORING_V1',sourceSha256:sha(out),previousRigSha256:rig.contentSha256,referenceWeights:rows,limbs,changedVertices:changes.length,admission:'NOT_VALIDATED'}),{flag:'wx'});console.log(JSON.stringify({sha256:sha(out),bytes:out.length,changedVertices:changes.length,limbs}));
