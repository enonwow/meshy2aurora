// @vitest-environment jsdom
import { describe,it,expect,vi } from "vitest";
import { webcrypto } from "node:crypto";
import { CreatureAutomationSession,canBuildCreature,sameOriginInputUrl,type WorkerPort } from "./session";
import { creatureTools,registerCreatureWebMCP } from "./webmcp";
Object.defineProperty(globalThis,"crypto",{value:webcrypto,configurable:true});
const BASE="http://127.0.0.1:5186/?creatureAgent=1";
function port(){return {request:vi.fn(async()=>{throw Error("unexpected worker call")}),dispose:vi.fn()} as unknown as WorkerPort;}
const wait=async(s:CreatureAutomationSession)=>{await vi.waitFor(()=>expect(s.busy).toBe(false));};
describe("Creature automation",()=>{
 it("keeps an editable blocked rig visible without admitting export",async()=>{
  const worker=port();vi.mocked(worker.request).mockResolvedValue({ok:true,type:"REFERENCE_SUPERMODEL_RIG_V2_PREPARED",requestId:"r",reportJson:JSON.stringify({status:"REFERENCE_SUPERMODEL_FITTED_RIG_V2_NEEDS_AUTHORING"}),authoringJson:"{}",targetRigJson:"{}"});
  const s=new CreatureAutomationSession(worker,BASE);s.source={provenance:"SOURCE",file:{arrayBuffer:async()=>new ArrayBuffer(8)} as File,sourceSha256:"1".repeat(64)};s.reference={resref:"c_wolf",blob:new ArrayBuffer(8),descriptorsJson:"[]",reports:[],hashes:[]};
  s.start("prepare",{expectedRevision:0});await wait(s);expect(s.phase).toBe("NEEDS_AUTHORING");expect(s.reports.has("targetRig")).toBe(true);expect(s.state().canExport).toBe(false);
 });
 it("verifies imported authoring before invoking the worker",async()=>{
  const worker=port();const s=new CreatureAutomationSession(worker,BASE,vi.fn(async()=>new Response(new TextEncoder().encode("{}"))) as typeof fetch);
  s.source={provenance:"SOURCE",file:{arrayBuffer:async()=>new ArrayBuffer(8)} as File,sourceSha256:"1".repeat(64)};s.reference={resref:"c_wolf",blob:new ArrayBuffer(8),descriptorsJson:"[]",reports:[],hashes:[]};
  s.start("import_authoring",{expectedRevision:0,url:"/weights.json",sha256:"0".repeat(64)});await wait(s);expect(s.error?.code).toBe("M2A-AGENT-AUTHORING-HASH-MISMATCH");expect(worker.request).not.toHaveBeenCalled();
 });
 it("requires the complete central admission and rejects a diagnostic-only preview",()=>{
  const valid={admissionV3:{status:"PASS"},motionCompatible:true,fullCarrierCoverage:true,requiredJointCoverage:true,skinInfluenceCoverage:true,inheritedClipCoverage:true,visibleMotionCoverage:true,seamViolationCount:0,motionQualityStatus:"PASS",runtimeReadiness:"RUNTIME_UNPROVEN"};
  expect(canBuildCreature(valid)).toBe(true);for(const key of Object.keys(valid))expect(canBuildCreature({...valid,[key]:undefined})).toBe(false);
  expect(canBuildCreature({...valid,seamViolationCount:1})).toBe(false);expect(canBuildCreature({...valid,admissionV3:{status:"BLOCKED"}})).toBe(false);
 });
 it("rejects off-origin, file and credential-bearing source URLs",()=>{
  expect(sameOriginInputUrl("/sample.glb",BASE)).toBe("http://127.0.0.1:5186/sample.glb");
  for(const url of ["https://elsewhere.test/a","file:///C:/a.glb","http://user:pass@127.0.0.1:5186/a"])expect(()=>sameOriginInputUrl(url,BASE)).toThrow();
 });
 it("keeps a source hash mismatch atomic and never calls the rig worker",async()=>{
  const worker=port();const s=new CreatureAutomationSession(worker,BASE,vi.fn(async()=>new Response(new Uint8Array([1,2,3]))) as typeof fetch);
  s.start("load_source",{expectedRevision:0,url:"/source.glb",sha256:"0".repeat(64),sourceForward:"POSITIVE_Z"});await wait(s);
  expect(s.state().error?.code).toBe("M2A-AGENT-SOURCE-HASH-MISMATCH");expect(s.source).toBeUndefined();expect(worker.request).not.toHaveBeenCalled();expect(s.state().canExport).toBe(false);
 });
 it("rejects concurrent jobs and stale revisions",async()=>{
  let finish!:(r:Response)=>void;const s=new CreatureAutomationSession(port(),BASE,(()=>new Promise(r=>finish=r)) as typeof fetch);
  s.start("load_source",{expectedRevision:0,url:"/s.glb",sha256:"0".repeat(64),sourceForward:"POSITIVE_Z"});
  expect(()=>s.start("prepare",{expectedRevision:1})).toThrow("M2A-AGENT-BUSY");finish(new Response(new Uint8Array([1])));await wait(s);
  expect(()=>s.start("prepare",{expectedRevision:0})).toThrow("M2A-AGENT-STALE-REVISION");
 });
 it("preserves structured pipeline failure with no preview or export",async()=>{
  const error={schemaVersion:2,code:"M2A-REFERENCE-SUPERMODEL-SKIN-BRANCH-REPAIR-EXCESSIVE",path:"skinning.branchBoundaryRepair",message:"2140 exceeds 1291"};
  const worker=port();vi.mocked(worker.request).mockRejectedValue(new Error(JSON.stringify(error)));
  const s=new CreatureAutomationSession(worker,BASE);s.source={provenance:"SOURCE",file:{arrayBuffer:async()=>new ArrayBuffer(8),name:"source.glb",size:8} as File,sourceSha256:"1".repeat(64)};s.reference={resref:"c_wolf",blob:new ArrayBuffer(8),descriptorsJson:"[]",reports:[],hashes:[]};
  s.start("prepare",{expectedRevision:0});await wait(s);expect(s.error).toEqual(error);expect(s.artifacts).toEqual([]);expect(s.state().canExport).toBe(false);
  expect(vi.mocked(worker.request).mock.calls[0]![0]).toMatchObject({type:"PREPARE_REFERENCE_SUPERMODEL_RIG_V2",experimentalAllowExcessiveSkinBranchRepair:false});
 });
 it("cannot fetch appearance or call export without admission",async()=>{
  const worker=port();const fetcher=vi.fn();const s=new CreatureAutomationSession(worker,BASE,fetcher);s.source={provenance:"SOURCE",file:{arrayBuffer:async()=>new ArrayBuffer(8)} as File,sourceSha256:"1".repeat(64)};s.reference={resref:"c_wolf",blob:new ArrayBuffer(8),descriptorsJson:"[]",reports:[],hashes:[]};
  s.start("build",{expectedRevision:0,appearanceUrl:"/a.2da"});await wait(s);expect(s.error?.code).toBe("M2A-AGENT-EXPORT-NOT-ADMITTED");expect(fetcher).not.toHaveBeenCalled();expect(worker.request).not.toHaveBeenCalled();
 });
 it("bounds report/artifact reads and cannot export retail bytes",()=>{
  const s=new CreatureAutomationSession(port(),BASE);s.reports.set("failure",{message:"abcdef"});const p=s.report("failure",0,5);expect(p.nextOffset).toBe(5);expect(p.text).toHaveLength(5);expect(()=>s.report("failure",0,32001)).toThrow();expect(()=>s.artifact("c_wolf")).toThrow();
 });
 it("selects bounded report subtrees without exposing prototypes",()=>{
  const s=new CreatureAutomationSession(port(),BASE);s.reports.set("source",{report:{triangles:20534},ir:{primitives:[{positions:[1,2,3]}]}});
  expect(JSON.parse(s.report("source",0,100,"/report").text)).toEqual({triangles:20534});expect(()=>s.report("source",0,100,"/__proto__")).toThrow("M2A-AGENT-REPORT-PATH");
 });
 it("does not expose a generic worker/eval or experimental bypass tool",async()=>{
  const tools=creatureTools(new CreatureAutomationSession(port(),BASE));expect(tools).toHaveLength(10);
  expect(tools.some(t=>/eval|bypass|worker_request/.test(t.name))).toBe(false);
  const result=JSON.parse(await tools.find(t=>t.name==="m2a_creature_prepare")!.execute({expectedRevision:0,experimentalAllowExcessiveSkinBranchRepair:true}));expect(result.ok).toBe(false);
 });
 it("registers actual native tools, cleans up synchronously and supports StrictMode remount",async()=>{
  const registerTool=vi.fn();const unregisterTool=vi.fn();const host={document:{modelContext:{registerTool,unregisterTool}},navigator:{}} as unknown as Window;
  const first=registerCreatureWebMCP(new CreatureAutomationSession(port(),BASE),host);await first.ready;expect(registerTool).toHaveBeenCalledTimes(10);first.dispose();
  const second=registerCreatureWebMCP(new CreatureAutomationSession(port(),BASE),host);expect((await second.ready).mode).toBe("native-webmcp");second.dispose();expect(unregisterTool).toHaveBeenCalled();
 });
 it("honestly labels fallback when native registration fails",async()=>{
  const host={document:{modelContext:{registerTool:()=>{throw Error("unsupported");}}},navigator:{}} as unknown as Window;
  const bridge=registerCreatureWebMCP(new CreatureAutomationSession(port(),BASE),host);expect(await bridge.ready).toMatchObject({mode:"documented-local-bridge",registrationError:"unsupported"});bridge.dispose();
 });
});
