import type { StudioWorkerRequest, StudioWorkerResponse, WorkerArtifact, CreatureSourceForwardV1 } from "../../worker/types";
import { projectCanonicalReadback } from "../results/projectReadback";
import type { BinaryMdlInspectionReport, SourcePreviewInput } from "../preview/types";
import { parseReferenceSupermodelRigAuthoringV2 } from "../supermodels/rigAuthoring";
import { parseNwnKeyModelIndexV1, parseNwnBifIndexPlanV1, parseNwnBifIndexV1 } from "../supermodels/types";
import { retailNwnResourceUrlV1 } from "../supermodels/RetailSupermodelDiagnostic";

export interface WorkerPort { request(input: StudioWorkerRequest): Promise<StudioWorkerResponse>; dispose(): void }
type Args = Record<string, unknown>;
type ReportName = "source" | "preparation" | "authoring" | "targetRig" | "preview" | "product" | "failure";
const FORWARDS = ["POSITIVE_Z", "NEGATIVE_Z", "POSITIVE_X", "NEGATIVE_X"] as const;
const HASH = /^[a-f0-9]{64}$/;
const RESREF = /^[a-z0-9_]{1,16}$/;
export function sameOriginInputUrl(input: string, base: string) {
  const url = new URL(input, base);
  const origin = new URL(base);
  if (url.origin !== origin.origin || !["http:", "https:"].includes(url.protocol) || url.username || url.password) throw Error("M2A-AGENT-INPUT-ORIGIN");
  return url.href;
}
function str(a: Args, name: string) { if (typeof a[name] !== "string" || !a[name]) throw Error("M2A-AGENT-ARGUMENT: " + name); return a[name] as string; }
function resref(a: Args, name: string) { const v = str(a, name).toLowerCase(); if (!RESREF.test(v)) throw Error("M2A-AGENT-RESREF: " + name); return v; }
function exact<T extends StudioWorkerResponse["type"]>(r: StudioWorkerResponse, type: T): Extract<StudioWorkerResponse, {ok:true; type:T}> {
  if (!r.ok) throw Error(r.message);
  if (r.type !== type) throw Error("M2A-AGENT-WORKER-RESPONSE: " + r.type);
  return r as Extract<StudioWorkerResponse, {ok:true; type:T}>;
}
async function digest(bytes: ArrayBuffer) { return Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)), b => b.toString(16).padStart(2, "0")).join(""); }
export function canBuildCreature(report: unknown): boolean {
  if (!report || typeof report !== "object") return false;
  const r = report as Args;
  return (r.admissionV3 as Args | undefined)?.status === "PASS"
    && ["motionCompatible", "fullCarrierCoverage", "requiredJointCoverage", "skinInfluenceCoverage", "inheritedClipCoverage", "visibleMotionCoverage"].every(k => r[k] === true)
    && r.seamViolationCount === 0 && r.motionQualityStatus === "PASS" && r.runtimeReadiness === "RUNTIME_UNPROVEN";
}
export function structuredFailure(reason: unknown) {
  const message = reason instanceof Error ? reason.message : String(reason);
  try { const parsed = JSON.parse(message); if (parsed && typeof parsed.code === "string") return parsed as Args; } catch { /* preserve ordinary errors below */ }
  return {code: message.split(":")[0], message};
}

/** Session owner shared by visible controls and WebMCP. No alternate rig/export implementation. */
export class CreatureAutomationSession {
  revision = 0;
  busy = false;
  phase = "EMPTY";
  job?: {id:string; operation:string; status:"RUNNING"|"SUCCEEDED"|"FAILED"};
  error?: Args;
  source?: SourcePreviewInput;
  sourceUrl?: string;
  sourceForward: CreatureSourceForwardV1 = "POSITIVE_Z";
  reference?: {resref:string; blob:ArrayBuffer; descriptorsJson:string; reports:BinaryMdlInspectionReport[]; hashes:string[]};
  preview?: BinaryMdlInspectionReport;
  artifacts: WorkerArtifact[] = [];
  readonly reports = new Map<ReportName, unknown>();
  private readonly listeners = new Set<() => void>();
  private disposed = false;
  private readonly controller = new AbortController();
  constructor(private readonly worker: WorkerPort, private readonly baseUrl: string, private readonly fetcher: typeof fetch = (...args) => fetch(...args)) {}
  subscribe = (fn:()=>void) => { this.listeners.add(fn); return () => {this.listeners.delete(fn);}; };
  private notify() { if (!this.disposed) this.listeners.forEach(fn => fn()); }
  state() { return {schemaVersion:1, revision:this.revision, phase:this.phase, busy:this.busy, job:this.job, error:this.error,
    source:this.source ? {url:this.sourceUrl, fileName:this.source.file.name, sha256:this.source.sourceSha256, byteLength:this.source.file.size, sourceForward:this.sourceForward} : null,
    supermodel:this.reference ? {resref:this.reference.resref, hashes:this.reference.hashes, chain:JSON.parse(this.reference.descriptorsJson)} : null,
    reports:[...this.reports.keys()], canExport:!this.busy && this.phase!=="BLOCKED" && canBuildCreature(this.reports.get("preview")),
    artifacts:this.artifacts.map(({bytes:_, ...metadata})=>metadata), nativeRuntime:"NOT_TESTED"}; }
  dispose() { this.disposed=true; this.controller.abort(); this.worker.dispose(); this.listeners.clear(); }
  private async request(request: Omit<StudioWorkerRequest,"requestId">) {
    const response = await this.worker.request({...request, requestId:crypto.randomUUID()} as StudioWorkerRequest);
    if (this.disposed) throw Error("M2A-AGENT-SESSION-DISPOSED");
    return response;
  }
  private async bytes(url: string, range?: [number,number], limit=64*1024*1024) {
    const response = await this.fetcher(sameOriginInputUrl(url,this.baseUrl), {signal:this.controller.signal, redirect:"error", ...(range?{headers:{Range:"bytes="+range[0]+"-"+(range[0]+range[1]-1)}}:{})});
    if (!response.ok || (range && response.status!==206)) throw Error("M2A-AGENT-FETCH: " + response.status);
    if (Number(response.headers.get("content-length"))>limit) throw Error("M2A-AGENT-INPUT-LIMIT");
    const bytes = await response.arrayBuffer();
    if (bytes.byteLength>limit || (range && bytes.byteLength!==range[1])) throw Error("M2A-AGENT-INPUT-LENGTH");
    return bytes;
  }
  private invalidate() { this.preview=undefined; this.artifacts=[]; for(const name of ["preparation","authoring","targetRig","preview","product"] as ReportName[])this.reports.delete(name); }
  private context() {
    if(!this.source || !this.reference)throw Error("M2A-AGENT-SOURCE-AND-SUPERMODEL-REQUIRED");
    return {selectedSupermodelResref:this.reference.resref, referenceChainBlob:this.reference.blob, referenceChainJson:this.reference.descriptorsJson, sourceForward:this.sourceForward, experimentalAllowExcessiveSkinBranchRepair:false as const};
  }
  start(operation:string, args:Args) {
    if(this.disposed)throw Error("M2A-AGENT-SESSION-DISPOSED");
    if(this.busy)throw Error("M2A-AGENT-BUSY");
    if(args.expectedRevision!==this.revision)throw Error("M2A-AGENT-STALE-REVISION");
    if(!["load_source","select_supermodel","prepare","set_authoring","import_authoring","preview","build"].includes(operation))throw Error("M2A-AGENT-UNKNOWN-OPERATION");
    const job = {id:crypto.randomUUID(),operation,status:"RUNNING" as "RUNNING"|"SUCCEEDED"|"FAILED"};
    this.job=job; this.busy=true; this.revision++; this.error=undefined; this.reports.delete("failure"); this.notify();
    void this.perform(operation,args).then(()=>{ if(this.disposed)return; job.status="SUCCEEDED"; }).catch(e=>{
      if(this.disposed)return; job.status="FAILED"; this.phase="BLOCKED"; this.error=structuredFailure(e); this.reports.set("failure",this.error);
    }).finally(()=>{if(!this.disposed){this.busy=false;this.notify();}});
    return this.state();
  }
  private async perform(operation:string,args:Args) {
    if(operation==="load_source") {
      const hash=str(args,"sha256"); if(!HASH.test(hash))throw Error("M2A-AGENT-SHA256");
      const forward=str(args,"sourceForward"); if(!(FORWARDS as readonly string[]).includes(forward))throw Error("M2A-AGENT-FORWARD");
      const url=sameOriginInputUrl(str(args,"url"),this.baseUrl);
      const bytes=await this.bytes(url); if(await digest(bytes)!==hash)throw Error("M2A-AGENT-SOURCE-HASH-MISMATCH");
      const response=exact(await this.request({type:"INSPECT_SOURCE",sourceGlb:bytes,target:"CREATURE",creatureProfile:"PRODUCT_300K"} as Omit<StudioWorkerRequest,"requestId">),"SOURCE_INSPECTED");
      this.source={provenance:"SOURCE",file:new File([bytes],decodeURIComponent(new URL(url).pathname.split("/").pop()||"source.glb")),sourceSha256:hash};
      this.sourceUrl=url;this.sourceForward=forward as CreatureSourceForwardV1;this.invalidate();this.reports.set("source",JSON.parse(response.ingestJson));this.phase="SOURCE_LOADED";return;
    }
    if(operation==="select_supermodel") {
      const selected=resref(args,"resref");
      const root=str(args,"nwnRootUrl");
      const url=(p:string)=>retailNwnResourceUrlV1(root,p,this.baseUrl);
      const key=parseNwnKeyModelIndexV1(exact(await this.request({type:"INDEX_NWN_KEY_MODELS",keyBytes:await this.bytes(url("data/nwn_base.key"))} as Omit<StudioWorkerRequest,"requestId">),"NWN_KEY_MODELS_INDEXED").indexJson);
      const chunks:ArrayBuffer[]=[]; const descriptors:Args[]=[]; const reports:BinaryMdlInspectionReport[]=[]; const hashes:string[]=[];
      let current=selected;let offset=0;const seen=new Set<string>();
      while(current!=="null") {
        if(seen.has(current)||seen.size>=32)throw Error("M2A-AGENT-REFERENCE-CHAIN-CYCLE");seen.add(current);
        const entry=key.models.find(m=>m.resref.toLowerCase()===current);if(!entry)throw Error("M2A-AGENT-REFERENCE-MISSING: "+current);
        const bif=key.bifs.find(b=>b.index===entry.bifIndex);if(!bif)throw Error("M2A-AGENT-BIF-MISSING");
        const bifUrl=url(bif.logicalName);const header=await this.bytes(bifUrl,[0,20]);
        const plan=parseNwnBifIndexPlanV1(exact(await this.request({type:"PLAN_NWN_BIF_INDEX",headerBytes:header} as Omit<StudioWorkerRequest,"requestId">),"NWN_BIF_INDEX_PLANNED").planJson);
        const table=await this.bytes(bifUrl,[plan.tableOffset,plan.tableByteLength]);
        const index=parseNwnBifIndexV1(exact(await this.request({type:"INDEX_NWN_BIF_TABLE",headerBytes:header,tableBytes:table} as Omit<StudioWorkerRequest,"requestId">),"NWN_BIF_TABLE_INDEXED").indexJson);
        const resource=index.resources.find(r=>r.resourceIndex===entry.resourceIndex);if(!resource || resource.resourceType!==2002)throw Error("M2A-AGENT-MDL-RESOURCE-MISSING");
        const mdl=await this.bytes(bifUrl,[resource.payloadOffset,resource.payloadSize]);const hash=await digest(mdl);
        const report=projectCanonicalReadback(exact(await this.request({type:"INSPECT_BINARY_MDL",mdlBytes:mdl} as Omit<StudioWorkerRequest,"requestId">),"BINARY_MDL_INSPECTED").reportJson);
        const parent=report.model?.supermodelName?.toLowerCase()||"null";
        descriptors.push({resref:current,supermodelResref:parent==="null"?"NULL":parent,format:"BINARY",sha256:hash,byteOffset:offset,byteLength:mdl.byteLength});offset+=mdl.byteLength;
        if(offset>64*1024*1024)throw Error("M2A-AGENT-REFERENCE-LIMIT");
        chunks.push(mdl);reports.push(report);hashes.push(hash);current=parent;
      }
      const blob=new Uint8Array(offset);let cursor=0;for(const chunk of chunks){blob.set(new Uint8Array(chunk),cursor);cursor+=chunk.byteLength;}
      this.reference={resref:selected,blob:blob.buffer,descriptorsJson:JSON.stringify(descriptors),reports,hashes};this.invalidate();this.phase="SUPERMODEL_SELECTED";return;
    }
    const context=this.context(); const sourceGlb=await this.source!.file.arrayBuffer();
    if(operation==="prepare" || operation==="set_authoring" || operation==="import_authoring") {
      let authoringJson:string|undefined;
      if(operation==="import_authoring") {
        const hash=str(args,"sha256");if(!HASH.test(hash))throw Error("M2A-AGENT-SHA256");
        const bytes=await this.bytes(str(args,"url"),undefined,32*1024*1024);
        if(await digest(bytes)!==hash)throw Error("M2A-AGENT-AUTHORING-HASH-MISMATCH");
        authoringJson=new TextDecoder("utf-8",{fatal:true}).decode(bytes);
      }
      if(operation==="set_authoring" || operation==="import_authoring") {
        authoringJson??=str(args,"authoringJson");const doc=parseReferenceSupermodelRigAuthoringV2(authoringJson);
        if(doc.sourceSha256!==this.source!.sourceSha256 || doc.selectedSupermodelResref!==this.reference!.resref || doc.sourceForward!==this.sourceForward)throw Error("M2A-AGENT-AUTHORING-IDENTITY");
        if(doc.jointOverrides.length || doc.landmarkOverrides.length)throw Error("M2A-AGENT-IMMUTABLE-BIND");
      }
      // Keep saved corrections when repeating preparation; resetting must be a new source/reference selection.
      authoringJson ??= this.reports.has("authoring")?JSON.stringify(this.reports.get("authoring")):undefined;
      const r=exact(await this.request({type:"PREPARE_REFERENCE_SUPERMODEL_RIG_V2",...context,sourceGlb,authoringJson} as Omit<StudioWorkerRequest,"requestId">),"REFERENCE_SUPERMODEL_RIG_V2_PREPARED");
      const preparation=JSON.parse(r.reportJson);
      this.invalidate();this.reports.set("preparation",preparation);this.reports.set("authoring",JSON.parse(r.authoringJson));this.reports.set("targetRig",JSON.parse(r.targetRigJson));this.phase=String(preparation.status).includes("NEEDS_AUTHORING")?"NEEDS_AUTHORING":"PREPARED";return;
    }
    if(operation==="preview") {
      if(!this.reports.has("authoring"))throw Error("M2A-AGENT-PREPARE-REQUIRED");
      const r=exact(await this.request({type:"BUILD_REFERENCE_SUPERMODEL_AUTHORED_PREVIEW",...context,sourceGlb,authoringJson:JSON.stringify(this.reports.get("authoring"))} as Omit<StudioWorkerRequest,"requestId">),"REFERENCE_SUPERMODEL_APPLIED_PREVIEW_BUILT");
      this.preview=projectCanonicalReadback(r.readbackJson);this.reports.set("preview",JSON.parse(r.applyReportJson));this.reports.set("authoring",JSON.parse(r.authoringJson));this.reports.set("targetRig",JSON.parse(r.targetRigJson));this.artifacts=r.artifacts;
      this.phase=canBuildCreature(this.reports.get("preview"))?"ADMITTED_OFFLINE":"DIAGNOSTIC_ONLY";return;
    }
    if(operation==="build") {
      if(this.phase==="BLOCKED" || !canBuildCreature(this.reports.get("preview")))throw Error("M2A-AGENT-EXPORT-NOT-ADMITTED");
      const appearance=await this.bytes(str(args,"appearanceUrl"));if(await digest(appearance)!==str(args,"appearanceSha256"))throw Error("M2A-AGENT-APPEARANCE-HASH-MISMATCH");
      const identity=JSON.parse(str(args,"identityJson"));for(const key of ["modelResref","textureResref","materialResref","hakResref"])resref(identity,key);
      const r=exact(await this.request({type:"BUILD_MODEL_PACKAGE",packageLane:"REFERENCE_SUPERMODEL_CREATURE",...context,sourceGlb,appearanceTwoDa:appearance,identityJson:JSON.stringify(identity),rigAuthoringJson:JSON.stringify(this.reports.get("authoring"))} as Omit<StudioWorkerRequest,"requestId">),"MODEL_PACKAGE_BUILT");
      for(const artifact of r.artifacts)if(await digest(artifact.bytes)!==artifact.sha256)throw Error("M2A-AGENT-ARTIFACT-HASH-MISMATCH");
      this.artifacts=r.artifacts;this.preview=projectCanonicalReadback(r.readbackJson);this.reports.set("product",{report:JSON.parse(r.reportJson),manifest:JSON.parse(r.manifestJson),summary:JSON.parse(r.summaryJson)});this.phase="EXPORTED_OFFLINE";return;
    }
  }
  report(name:ReportName,offset=0,limit=12000,path="") {
    if(!this.reports.has(name))throw Error("M2A-AGENT-REPORT-MISSING");
    if(!Number.isSafeInteger(offset)||offset<0||!Number.isSafeInteger(limit)||limit<1||limit>32000)throw Error("M2A-AGENT-RANGE");
    let value=this.reports.get(name);
    if(path){
      if(!path.startsWith("/")||path.length>1024)throw Error("M2A-AGENT-REPORT-PATH");
      for(const part of path.slice(1).split("/")){const key=part.replace(/~1/g,"/").replace(/~0/g,"~");if(!value||typeof value!=="object"||!Object.prototype.hasOwnProperty.call(value,key))throw Error("M2A-AGENT-REPORT-PATH");value=(value as Record<string,unknown>)[key];}
    }
    const json=JSON.stringify(value);return {name,path,totalCharacters:json.length,offset,text:json.slice(offset,offset+limit),nextOffset:offset+limit<json.length?offset+limit:null};
  }
  artifact(id:string,offset=0,limit=49152) {
    const a=this.artifacts.find(a=>a.artifactId===id);if(!a)throw Error("M2A-AGENT-ARTIFACT-MISSING");
    if(!Number.isSafeInteger(offset)||offset<0||offset>a.byteLength||!Number.isSafeInteger(limit)||limit<1||limit>49152)throw Error("M2A-AGENT-RANGE");
    const bytes=new Uint8Array(a.bytes,offset,Math.min(limit,a.byteLength-offset));let binary="";for(const byte of bytes)binary+=String.fromCharCode(byte);
    return {artifactId:id,fileName:a.fileName,sha256:a.sha256,totalBytes:a.byteLength,offset,base64:btoa(binary),nextOffset:offset+bytes.length<a.byteLength?offset+bytes.length:null};
  }
}
