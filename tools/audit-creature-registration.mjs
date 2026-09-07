import fs from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';

// Independent source landmarks are required before weight authoring. Geometry
// probes are useful diagnostics but never certify anatomical correspondence.
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const vec = (v) => Array.isArray(v) && v.length === 3 && v.every(Number.isFinite);
const length = (v) => Math.hypot(...v);
const sub = (a,b) => a.map((x,i) => x-b[i]);
const mean = (rows) => [0,1,2].map(a => rows.reduce((s,r) => s+r[a],0)/rows.length);
function requireValue(ok, message) { if (!ok) throw Error(message); }

export function auditRegistration(spec, actualBinding) {
  requireValue(spec.schemaVersion === 1, 'Unsupported registration schema');
  requireValue(vec(spec.forwardAxis) && Math.abs(length(spec.forwardAxis)-1)<1e-6, 'Forward axis must be a unit vector');
  requireValue(Array.isArray(spec.requiredPointIds) && spec.requiredPointIds.length>0 && new Set(spec.requiredPointIds).size===spec.requiredPointIds.length && spec.requiredPointIds.every(x=>typeof x==='string' && x.length>0), 'Expected unique required anatomy point IDs');
  requireValue(Array.isArray(spec.points), 'Missing points array');
  for (const key of ['sourceSha256','referenceSha256']) {
    requireValue(/^[0-9a-f]{64}$/.test(spec[key]) && /^[0-9a-f]{64}$/.test(actualBinding[key]), 'Missing identity: '+key);
    requireValue(spec[key]===actualBinding[key], 'Stale annotation identity: '+key);
  }
  const seen=new Set();
  const points=spec.points.map(p=>{
    requireValue(typeof p.id==='string' && p.id.length>0 && !seen.has(p.id), 'Duplicate or invalid point ID');seen.add(p.id);
    requireValue(typeof p.region==='string' && p.region.length>0 && vec(p.source) && vec(p.target) && Number.isFinite(p.tolerance) && p.tolerance>0, 'Invalid registration point: '+p.id);
    requireValue(['MESH_ANNOTATION','GEOMETRIC_PROXY','RIG_DERIVED'].includes(p.sourceKind) && typeof p.reviewed==='boolean', 'Missing annotation provenance: '+p.id);
    const delta=sub(p.target,p.source), distance=length(delta);
    return {...p,delta,distance,normalizedError:distance/p.tolerance,
      forwardCorrection:delta.reduce((s,x,i)=>s+x*spec.forwardAxis[i],0),
      independentAndReviewed:p.sourceKind==='MESH_ANNOTATION' && p.reviewed};
  });
  const required=spec.requiredPointIds.map(id=>points.find(p=>p.id===id));
  const missing=spec.requiredPointIds.filter((id,i)=>!required[i]);
  const rows=required.filter(Boolean);
  const translation=rows.length ? mean(rows.map(p=>p.delta)) : null;
  const residuals=rows.map(p=>({id:p.id,region:p.region,normalizedErrorAfterTranslation:length(sub(p.delta,translation))/p.tolerance}));
  const regions=[...new Set(rows.map(p=>p.region))].map(region=>{
    const r=rows.filter(p=>p.region===region);
    return {region,meanCorrection:mean(r.map(p=>p.delta)),maxNormalizedError:Math.max(...r.map(p=>p.normalizedError))};
  });
  const unreviewed=rows.filter(p=>!p.independentAndReviewed).map(p=>p.id);
  const violations=rows.filter(p=>p.normalizedError>1).map(p=>p.id);
  const blockers=[];
  if(spec.anatomyCoverageReviewed!==true)blockers.push('ANATOMY_COVERAGE_NOT_REVIEWED');
  if(missing.length)blockers.push('MISSING_REQUIRED_LANDMARKS');
  if(unreviewed.length)blockers.push('INDEPENDENT_SOURCE_ANNOTATIONS_REQUIRED');
  if(violations.length)blockers.push('REGISTRATION_OUTSIDE_TOLERANCE');
  return {schemaVersion:1,sourceSha256:spec.sourceSha256,referenceSha256:spec.referenceSha256,
    status:blockers.length?'BLOCKED':'WITHIN_REVIEWED_LANDMARK_TOLERANCES',
    canAuthorWeights:blockers.length===0,blockers,missing,unreviewed,violations,points,regions,
    translationDiagnostic:{delta:translation,forward:translation?.reduce((s,x,i)=>s+x*spec.forwardAxis[i],0)??null,
      residuals,sufficesForMeasuredPoints:rows.length>0 && missing.length===0 && residuals.every(p=>p.normalizedErrorAfterTranslation<=1),
      automaticCorrectionAllowed:false},
    motionReviewRequired:true,nativeCompatibility:'NOT_ASSESSED'};
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const [specPath,sourcePath,referencePath,outputPath]=process.argv.slice(2);
    requireValue(specPath && sourcePath && referencePath && outputPath,
      'Usage: node tools/audit-creature-registration.mjs review.json exact-source exact-reference report.json');
    const [spec,source,reference]=await Promise.all([fs.readFile(specPath,'utf8'),fs.readFile(sourcePath),fs.readFile(referencePath)]);
    const report=auditRegistration(JSON.parse(spec),{sourceSha256:sha(source),referenceSha256:sha(reference)});
    await fs.writeFile(outputPath,JSON.stringify(report,null,2)+'\n',{flag:'wx'});
    console.log(JSON.stringify({status:report.status,blockers:report.blockers,canAuthorWeights:report.canAuthorWeights,outputPath}));
    if(!report.canAuthorWeights)process.exitCode=2;
  } catch(error) { console.error(error.message);process.exitCode=1; }
}
