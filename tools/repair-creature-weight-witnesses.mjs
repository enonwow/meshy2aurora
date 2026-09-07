// Propose local edits around exact vertices reported by the full motion oracle.
// This is authoring, never admission. Re-run the complete pipeline on the result.
import fs from 'node:fs/promises';
const [rigPath, proposalPath, reportPath, outPath] = process.argv.slice(2);
const rig=JSON.parse(await fs.readFile(rigPath,'utf8'));
const proposal=JSON.parse(await fs.readFile(proposalPath,'utf8'));
const report=JSON.parse(await fs.readFile(reportPath,'utf8'));
if(proposal.baseRigSha256!==rig.contentSha256)throw Error('Wrong proposal base rig');
const segment=rig.segments[0], points=segment.surfacePositions;
const diagonal=Math.hypot(...[0,1,2].map(a=>Math.max(...points.map(p=>p[a]))-Math.min(...points.map(p=>p[a]))));
const witnesses=new Map();
for(const clip of report.motionQuality.clips) for(const edge of [clip.worstEdgeCollapse,clip.worstEdgeExpansion]) {
  if(!edge || (edge.lengthRatio>=0.0625 && edge.lengthRatio<=16))continue;
  if(edge.skinNodeName!==`m2a_seg_${segment.id}`)throw Error('Unknown surface segment');
  const ids=edge.vertexIndices, key=[...ids].sort((a,b)=>a-b).join(',');
  if(!witnesses.has(key))witnesses.set(key,{...edge,clip:clip.clipName});
}
const rows=proposal.referenceWeights.map(row=>new Map(row.map(w=>[w.boneNodeId,w.value])));
const distance=(p,q)=>Math.hypot(...p.map((v,a)=>v-q[a]));
const edits=[];
for(const witness of witnesses.values()) {
  // The MDL writer may reorder renderer vertices. Resolve the measured bind
  // positions back to the source surface, never reuse MDL indices as GLB IDs.
  const [a,b]=witness.bindEndpoints.map(p=>{
    let best=-1,nearest=Infinity;
    for(let i=0;i<points.length;i++){const d=distance(points[i],p);if(d<nearest){nearest=d;best=i;}}
    if(nearest>diagonal*1e-5)throw Error('Motion witness is not from this surface');
    return best;
  });
  const target=new Map();
  for(const id of [a,b])for(const [bone,w] of rows[id])target.set(bone,(target.get(bone)||0)+w/2);
  const center=points[a].map((v,k)=>(v+points[b][k])/2), half=distance(points[a],points[b])/2;
  const radius=Math.max(half*4,diagonal*0.016);
  let changed=0;
  for(let i=0;i<points.length;i++) {
    const d=distance(points[i],center);
    if(d>=radius)continue;
    const t=Math.max(0,Math.min(1,(d-half)/Math.max(radius-half,1e-8)));
    const blend=1-t*t*(3-2*t);
    const merged=new Map([...rows[i]].map(([id,w])=>[id,w*(1-blend)]));
    for(const [id,w] of target)merged.set(id,(merged.get(id)||0)+w*blend);
    rows[i]=merged;changed++;
  }
  edits.push({mdlVertices:witness.vertexIndices,sourceVertices:[a,b],clip:witness.clip,ratio:witness.lengthRatio,radiusFraction:radius/diagonal,changed});
}
function compact(row){
  let a=[...row].filter(([,w])=>w>1e-8).sort((a,b)=>b[1]-a[1]||a[0]-b[0]);
  const cutoff=a[4]?.[1]||0;a=a.slice(0,4).map(([id,w])=>[id,w-cutoff]).filter(([,w])=>w>1e-8);
  const sum=a.reduce((s,[,w])=>s+w,0);if(!sum)throw Error('Empty weight row');
  return a.map(([boneNodeId,w])=>({boneNodeId,value:w/sum}));
}
const output={...proposal,algorithm:'MOTION_WITNESS_LOCAL_COHERENCE_V1',referenceWeights:rows.map(compact),motionWitnessEdits:edits,admission:'NOT_VALIDATED'};
await fs.writeFile(outPath,JSON.stringify(output),{flag:'wx'});
console.log(JSON.stringify({witnesses:edits.length,edits}));
