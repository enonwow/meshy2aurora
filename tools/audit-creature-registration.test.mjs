import test from 'node:test';
import assert from 'node:assert/strict';
import {auditRegistration} from './audit-creature-registration.mjs';
const binding={sourceSha256:'a'.repeat(64),referenceSha256:'b'.repeat(64)};
function fixture(){return {schemaVersion:1,...binding,forwardAxis:[0,1,0],anatomyCoverageReviewed:true,requiredPointIds:['front','rear'],points:[
{id:'front',region:'front',source:[0,1,0],target:[0,1,0],tolerance:.02,sourceKind:'MESH_ANNOTATION',reviewed:true},
{id:'rear',region:'rear',source:[0,-1,0],target:[0,-1,0],tolerance:.02,sourceKind:'MESH_ANNOTATION',reviewed:true}]};}
test('reviewed fit authorizes only weighting, still requires motion review',()=>{const r=auditRegistration(fixture(),binding);assert.equal(r.canAuthorWeights,true);assert.equal(r.motionReviewRequired,true);assert.equal(r.nativeCompatibility,'NOT_ASSESSED');});
test('uniform backwards offset is blocked and diagnosed as forward translation',()=>{const s=fixture();for(const p of s.points)p.source[1]-=.1;const r=auditRegistration(s,binding);assert.equal(r.canAuthorWeights,false);assert(Math.abs(r.translationDiagnostic.forward-.1)<1e-9);assert.equal(r.translationDiagnostic.sufficesForMeasuredPoints,true);assert.equal(r.translationDiagnostic.automaticCorrectionAllowed,false);});
test('opposite regional errors cannot disappear through the mean',()=>{const s=fixture();s.points[0].source[1]-=.1;s.points[1].source[1]+=.1;const r=auditRegistration(s,binding);assert.equal(r.canAuthorWeights,false);assert.equal(r.violations.length,2);assert.equal(r.translationDiagnostic.sufficesForMeasuredPoints,false);});
test('missing annotations cannot pass with excellent measurements elsewhere',()=>{const s=fixture();s.points.pop();const r=auditRegistration(s,binding);assert.deepEqual(r.missing,['rear']);assert.equal(r.canAuthorWeights,false);});
for(const sourceKind of ['RIG_DERIVED','GEOMETRIC_PROXY'])test(sourceKind+' cannot certify anatomy even at zero error',()=>{const s=fixture();s.points[0].sourceKind=sourceKind;const r=auditRegistration(s,binding);assert.equal(r.canAuthorWeights,false);assert(r.blockers.includes('INDEPENDENT_SOURCE_ANNOTATIONS_REQUIRED'));});
test('changing the fitted model invalidates old review',()=>assert.throws(()=>auditRegistration(fixture(),{...binding,sourceSha256:'c'.repeat(64)}),/Stale annotation/));
test('coverage and unreviewed source annotations block',()=>{const s=fixture();s.anatomyCoverageReviewed=false;s.points[0].reviewed=false;assert.equal(auditRegistration(s,binding).blockers.length,2);});
test('malformed numeric and duplicate data fail closed',()=>{for(const mutate of [s=>s.points[0].source[0]=NaN,s=>s.points[0].tolerance=0,s=>s.points[1].id='front',s=>s.forwardAxis=[0,0,0],s=>s.requiredPointIds=[]]){const s=fixture();mutate(s);assert.throws(()=>auditRegistration(s,binding));}});
