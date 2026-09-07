// Deterministic authoring proposal. All decisions are derived from the supplied
// rig hierarchy, positions and weights; source geometry and carrier frames stay intact.
import fs from 'node:fs/promises';
const [rigPath, proposalPath, outputPath] = process.argv.slice(2);
const rig = JSON.parse(await fs.readFile(rigPath, 'utf8'));
const proposal = JSON.parse(await fs.readFile(proposalPath, 'utf8'));
if (proposal.baseRigSha256 !== rig.contentSha256) throw Error('Proposal belongs to a different rig');
const seg = rig.segments[0], nodes = new Map(rig.nodes.map(n => [n.id,n]));
const allowed = new Set(seg.allowedBoneNodeIds), world = new Map();
const mul = (a,b) => Array.from({length:16},(_,i)=>[0,1,2,3].reduce((v,k)=>v+a[i%4+k*4]*b[k+Math.floor(i/4)*4],0));
for (const n of rig.nodes) world.set(n.id,n.parentId === null?n.bindLocalMatrix:mul(world.get(n.parentId),n.bindLocalMatrix));
const parents = new Map();
for (const id of allowed) { let p=nodes.get(id).parentId; while(p!==null&&!allowed.has(p))p=nodes.get(p).parentId; parents.set(id,p); }
const children = new Map([...allowed].map(p=>[p,[...allowed].filter(c=>parents.get(c)===p)]));
const bounds=[0,1,2].map(a=>[Math.min(...seg.surfacePositions.map(p=>p[a])),Math.max(...seg.surfacePositions.map(p=>p[a]))]);
const diagonal=Math.hypot(...bounds.map(([a,b])=>b-a));
const sideTolerance=diagonal*0.015;
const sideBranches=new Map();
for(const id of allowed) {
  if(Math.abs(world.get(id)[12])<=sideTolerance)continue;
  let branch=id;
  while(parents.get(branch)!==null && Math.abs(world.get(parents.get(branch))[12])>sideTolerance)branch=parents.get(branch);
  const descendants=[];
  function collect(n){descendants.push(n);for(const c of children.get(n))collect(c);}
  collect(branch);
  sideBranches.set(id,{branch,parent:parents.get(branch),low:Math.min(...descendants.map(n=>world.get(n)[14]))});
}
function normalize(values) {
  let rows=[...values].filter(([,w])=>w>1e-8).sort((a,b)=>b[1]-a[1]||a[0]-b[0]);
  // Subtract the fifth influence before normalization to avoid an abrupt
  // top-four cutoff when equally weighted carriers exchange order.
  const cutoff=Math.max(0.005,rows.length>4?rows[4][1]:0);
  rows=rows.slice(0,4).map(([id,w])=>[id,w-cutoff]).filter(([,w])=>w>1e-8);
  const sum=rows.reduce((s,[,w])=>s+w,0);
  if(!sum)throw Error('Empty compact weight row');
  return rows.map(([boneNodeId,w])=>({boneNodeId,value:w/sum}));
}
// Smooth in world space rather than renderer-index space: UV seams and uneven
// tessellation must not introduce a discontinuity into the same skin surface.
const radius=diagonal*0.04, sigma=radius/2.5, cells=new Map(), unique=[];
const uniquePosition=new Set();
seg.surfacePositions.forEach((p,i)=>{
  const key=p.join(',');if(uniquePosition.has(key))return;uniquePosition.add(key);
  const cell=p.map(x=>Math.floor(x/radius)).join(',');
  if(!cells.has(cell))cells.set(cell,[]);cells.get(cell).push(i);unique.push(i);
});
const smoothed=new Map();
for(const i of unique) {
  const p=seg.surfacePositions[i], c=p.map(x=>Math.floor(x/radius));
  const accum=new Map();let sum=0;
  for(let x=-1;x<=1;x++)for(let y=-1;y<=1;y++)for(let z=-1;z<=1;z++)
    for(const j of cells.get([c[0]+x,c[1]+y,c[2]+z].join(','))||[]) {
      const q=seg.surfacePositions[j], d=p.reduce((s,v,k)=>s+(v-q[k])**2,0);
      if(d>radius*radius)continue;
      const g=Math.exp(-d/(2*sigma*sigma));sum+=g;
      for(const w of proposal.referenceWeights[j])accum.set(w.boneNodeId,(accum.get(w.boneNodeId)||0)+g*w.value);
    }
  smoothed.set(p.join(','),[...accum].map(([boneNodeId,w])=>({boneNodeId,value:w/sum})));
}
const constrained=seg.surfacePositions.map((point,index)=>{
  const row=smoothed.get(point.join(','));
  const p=seg.surfacePositions[index], masses=new Map();
  for(const {boneNodeId:id,value:w} of row) {
    const x=world.get(id)[12];
    let parent=id, factor=1;
    if(Math.abs(x)>sideTolerance) {
      const b=sideBranches.get(id), branchPosition=world.get(b.branch);
      // The blend to the trunk belongs at the limb attachment. It must taper
      // to zero at the distal end, otherwise medial paw vertices follow the trunk.
      const height=Math.max(0,Math.min(1,(p[2]-b.low)/Math.max(branchPosition[14]-b.low,diagonal*0.01)));
      const width=Math.max(diagonal*1e-5,Math.abs(branchPosition[12])*0.8*height);
      let t=Math.max(0,Math.min(1,p[0]*Math.sign(x)/width));
      factor=t*t*(3-2*t);
      parent=b.parent??b.branch;
    }
    masses.set(id,(masses.get(id)||0)+w*factor);
    masses.set(parent,(masses.get(parent)||0)+w*(1-factor));
  }
  // Local forks may blend across siblings through their inherited parent.
  // Do not manufacture direct weights for a passive global-motion dummy.
  return normalize(masses);
});
// Find connected surfaces using exact position welding. Small detached details
// receive one component-wide carrier, preventing eyes/teeth from stretching.
const uf=Array.from({length:seg.surfacePositions.length},(_,i)=>i);
function find(i){while(uf[i]!==i){uf[i]=uf[uf[i]];i=uf[i];}return i;}
function union(a,b){a=find(a);b=find(b);if(a!==b)uf[Math.max(a,b)]=Math.min(a,b);}
const positions=new Map();
seg.surfacePositions.forEach((p,i)=>{let key=p.join(',');if(positions.has(key))union(i,positions.get(key));else positions.set(key,i);});
for(let i=0;i<seg.surfaceIndices.length;i+=3){union(seg.surfaceIndices[i],seg.surfaceIndices[i+1]);union(seg.surfaceIndices[i],seg.surfaceIndices[i+2]);}
const groups=new Map();
uf.forEach((_,i)=>{let g=find(i);if(!groups.has(g))groups.set(g,[]);groups.get(g).push(i);});
const sorted=[...groups.values()].sort((a,b)=>b.length-a.length), componentEdits=[];
for(const group of sorted.slice(1)) {
  const ext=[0,1,2].map(a=>Math.max(...group.map(i=>seg.surfacePositions[i][a]))-Math.min(...group.map(i=>seg.surfacePositions[i][a])));
  if(Math.hypot(...ext)>diagonal*0.08)continue;
  const totals=new Map();for(const i of group)for(const w of constrained[i])totals.set(w.boneNodeId,(totals.get(w.boneNodeId)||0)+w.value);
  const part=[...totals].sort((a,b)=>b[1]-a[1]||a[0]-b[0])[0][0];
  for(const i of group)constrained[i]=[{boneNodeId:part,value:1}];
  componentEdits.push({vertexCount:group.length,carrier:part});
}
const out={...proposal,algorithm:'LOCAL_NEIGHBORHOOD_HEAT_PROPOSAL_V3',referenceWeights:constrained,componentEdits,admission:'NOT_VALIDATED'};
await fs.writeFile(outputPath,JSON.stringify(out),{flag:'wx'});
console.log(JSON.stringify({vertices:constrained.length,components:sorted.length,componentEdits}));
