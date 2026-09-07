import fs from 'node:fs/promises';
// Explicit local weight authoring from exact motion witnesses; never admission.
const [rigPath,proposalPath,reportPath,outPath]=process.argv.slice(2);
const rig=JSON.parse(await fs.readFile(rigPath,'utf8')),proposal=JSON.parse(await fs.readFile(proposalPath,'utf8')),report=JSON.parse(await fs.readFile(reportPath,'utf8'));
if(proposal.baseRigSha256!==rig.contentSha256)throw Error('Wrong base rig');
const s=rig.segments[0],p=s.surfacePositions,dist=(a,b)=>Math.hypot(...a.map((v,k)=>v-b[k]));
const diag=Math.hypot(...[0,1,2].map(k=>Math.max(...p.map(x=>x[k]))-Math.min(...p.map(x=>x[k]))));
const idsByName=new Map(rig.nodes.map(n=>[n.name,n.id]));
const nearest=q=>{let id=-1,d=Infinity;for(let i=0;i<p.length;i++){const v=dist(q,p[i]);if(v<d){d=v;id=i}}if(d>diag*1e-5)throw Error('Witness position mismatch');return id};
const regions=new Map();
for(const c of report.motionQuality.clips){
 for(const e of [c.worstEdgeCollapse,c.worstEdgeExpansion]){
  if(!e||(e.lengthRatio>=0.0625&&e.lengthRatio<=16))continue;
  const ids=e.bindEndpoints.map(nearest);
  for(let j=0;j<ids.length;j++){
   const expected=new Map(e.vertexInfluences[j].influences.map(w=>[idsByName.get(w.boneNodeName),w.weight]));
   const actual=new Map(proposal.referenceWeights[ids[j]].map(w=>[w.boneNodeId,w.value]));
   for(const key of new Set([...expected.keys(),...actual.keys()]))if(Math.abs((expected.get(key)||0)-(actual.get(key)||0))>1e-5)throw Error('Stale motion witness weights');
  }
  const center=p[ids[0]].map((v,k)=>(v+p[ids[1]][k])/2);
  regions.set(ids.slice().sort((a,b)=>a-b).join(','),{ids,center,kind:'edge',clip:c.clipName});
 }
 const t=c.worstTriangleAreaCollapse;
 if(t&&c.components.some(x=>!x.pass&&x.triangleAreaCollapseCount>0)){
  let best=null,d=Infinity;
  for(let i=0;i<s.surfaceIndices.length;i+=3){const ids=s.surfaceIndices.slice(i,i+3),center=[0,1,2].map(k=>ids.reduce((v,id)=>v+p[id][k]/3,0)),v=dist(center,t.bindCentroid);if(v<d){d=v;best={ids,center,kind:'triangle',clip:c.clipName}}}
  if(d>diag*1e-5)throw Error('Triangle witness mismatch');
  regions.set(best.ids.slice().sort((a,b)=>a-b).join(','),best);
 }
}
const rows=proposal.referenceWeights.map(row=>new Map(row.map(w=>[w.boneNodeId,w.value]))),edits=[];
for(const r of regions.values()){
 const target=new Map();for(const i of r.ids)for(const w of proposal.referenceWeights[i])target.set(w.boneNodeId,(target.get(w.boneNodeId)||0)+w.value/r.ids.length);
 const ordered=[...target].sort((a,b)=>b[1]-a[1]);
 // Bias the center toward its already dominant anatomical carrier. Keep the
 // remaining neighboring support; fade smoothly into the authored field.
 const dominant=ordered[0][0],base=target.get(dominant),boost=0.85;
 for(const [id,w] of target)target.set(id,id===dominant?boost:w*(1-boost)/(1-base));
 const inner=Math.max(...r.ids.map(i=>dist(p[i],r.center)))+diag*0.004,outer=Math.max(inner*3,diag*0.045);
 let changed=0;
 for(let i=0;i<p.length;i++){const d=dist(p[i],r.center);if(d>=outer)continue;const t=Math.max(0,Math.min(1,(d-inner)/(outer-inner))),blend=1-t*t*(3-2*t);const row=new Map([...rows[i]].map(([id,w])=>[id,w*(1-blend)]));for(const [id,w] of target)row.set(id,(row.get(id)||0)+blend*w);rows[i]=row;changed++}
 edits.push({...r,dominant,changed,radius:outer});
}
function compact(row){const sorted=[...row].filter(([,w])=>w>1e-8).sort((a,b)=>b[1]-a[1]);const cutoff=sorted[4]?.[1]||0,a=sorted.slice(0,4).map(([id,w])=>[id,w-cutoff]).filter(([,w])=>w>1e-8),mass=a.reduce((v,[,w])=>v+w,0);if(!mass)throw Error('Empty row');return a.map(([boneNodeId,w])=>({boneNodeId,value:w/mass}))}
await fs.writeFile(outPath,JSON.stringify({...proposal,algorithm:'MOTION_ANATOMICAL_NEIGHBORHOOD_AUTHORING_V1',referenceWeights:rows.map(compact),edits,admission:'NOT_VALIDATED'}),{flag:'wx'});console.log(JSON.stringify(edits));
