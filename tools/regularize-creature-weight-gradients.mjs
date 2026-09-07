// Bounded weight authoring on exact-position surface groups. Studio remains the
// independent authority for locality, coverage, bind and inherited-motion gates.
import fs from 'node:fs/promises';
const [rigPath,proposalPath,outPath]=process.argv.slice(2);
const rig=JSON.parse(await fs.readFile(rigPath,'utf8'));
const proposal=JSON.parse(await fs.readFile(proposalPath,'utf8'));
if(proposal.baseRigSha256!==rig.contentSha256)throw Error('Wrong proposal base');
const seg=rig.segments[0], n=rig.nodes.length;
const mul=(a,b)=>Array.from({length:16},(_,i)=>[0,1,2,3].reduce((v,k)=>v+a[i%4+k*4]*b[k+Math.floor(i/4)*4],0));
const worlds=new Map();for(const node of rig.nodes)worlds.set(node.id,node.parentId===null?node.bindLocalMatrix:mul(worlds.get(node.parentId),node.bindLocalMatrix));
const lockedVertices=new Set(proposal.lockedVertexIndices||[]);
const groups=[], map=new Map(), vertexGroup=[];
for(let i=0;i<seg.surfacePositions.length;i++){
  const p=seg.surfacePositions[i],key=p.join(',');
  if(!map.has(key)){map.set(key,groups.length);const w=new Float64Array(n);for(const v of proposal.referenceWeights[i])w[v.boneNodeId]=v.value;groups.push({p,w,locked:false});}
  vertexGroup.push(map.get(key));if(lockedVertices.has(i))groups[map.get(key)].locked=true;
}
const diagonal=Math.hypot(...[0,1,2].map(a=>Math.max(...groups.map(g=>g.p[a]))-Math.min(...groups.map(g=>g.p[a]))));
const edgeSet=new Set(),edges=[];
for(let i=0;i<seg.surfaceIndices.length;i+=3)for(const [u,v] of [[0,1],[1,2],[2,0]]){
  let a=vertexGroup[seg.surfaceIndices[i+u]],b=vertexGroup[seg.surfaceIndices[i+v]];if(a===b)continue;if(a>b)[a,b]=[b,a];
  const key=a+','+b;if(edgeSet.has(key))continue;edgeSet.add(key);
  edges.push({a,b,length:Math.hypot(...groups[a].p.map((v,k)=>v-groups[b].p[k]))});
}
const bindSpans=rig.nodes.filter(node=>node.parentId!==null).map(node=>({a:node.id,b:node.parentId,d:Math.hypot(...[12,13,14].map(k=>worlds.get(node.id)[k]-worlds.get(node.parentId)[k]))}));
function limit(edge,a,b){let span=0;for(const s of bindSpans)if((a[s.a]+b[s.a]>1e-8)&&(a[s.b]+b[s.b]>1e-8))span=Math.max(span,s.d);
  return Math.max(1e-6,Math.min(1,edge.length/diagonal*Math.max(2,Math.min(64,3/Math.max(span/diagonal,0.01)))))*0.95;
}
function compact(w){const ids=Array.from({length:n},(_,i)=>i).filter(i=>w[i]>1e-10).sort((a,b)=>w[b]-w[a]||a-b);const cut=ids.length>4?w[ids[4]]:0;const out=new Float64Array(n);let sum=0;for(const i of ids.slice(0,4)){out[i]=w[i]-cut;sum+=out[i];}if(sum<=0)throw Error('No weights after compaction');for(let i=0;i<n;i++)out[i]/=sum;return out;}
let updates=0;const history=[];
for(let round=0;round<80;round++){
  let violations=0,worst=0;
  for(const e of edges){const a=groups[e.a].w,b=groups[e.b].w,l=limit(e,a,b);let delta=0;for(let i=0;i<n;i++)delta=Math.max(delta,Math.abs(a[i]-b[i]));worst=Math.max(worst,delta/l);if(delta<=l*1.00001)continue;violations++;updates++;
    const factor=l/delta;const lockA=groups[e.a].locked,lockB=groups[e.b].locked;if(lockA&&lockB)continue;for(let i=0;i<n;i++){if(lockA){b[i]=a[i]+(b[i]-a[i])*factor;}else if(lockB){a[i]=b[i]+(a[i]-b[i])*factor;}else{const mid=(a[i]+b[i])/2,half=(a[i]-b[i])/2*factor;a[i]=mid+half;b[i]=mid-half;}}
  }
  for(const g of groups)if(!g.locked)g.w=compact(g.w);
  history.push({round,violations,worst});
  if(violations===0 || updates>250000)break;
}
const referenceWeights=vertexGroup.map(g=>[...groups[g].w].map((value,boneNodeId)=>({boneNodeId,value})).filter(w=>w.value>1e-8).sort((a,b)=>b.value-a.value||a.boneNodeId-b.boneNodeId));
const output={...proposal,algorithm:'GEOMETRY_SCALED_PAIR_PROJECTION_V1',referenceWeights,gradientProjection:{updates,history},admission:'NOT_VALIDATED'};
await fs.writeFile(outPath,JSON.stringify(output),{flag:'wx'});
console.log(JSON.stringify({groups:groups.length,edges:edges.length,updates,rounds:history.length,last:history.at(-1)}));
