import { CreatureAutomationSession, structuredFailure } from "./session";
export interface CreatureTool { name:string; description:string; inputSchema: {type:"object"; properties:Record<string,unknown>; required:string[]; additionalProperties:false}; annotations:{readOnlyHint:boolean}; execute(input:Record<string,unknown>):Promise<string> }
type Context = {registerTool(tool:CreatureTool, options?:{signal:AbortSignal}):Promise<unknown>|unknown; unregisterTool?(name:string):void};
const text={type:"string",minLength:1};const revision={type:"integer",minimum:0};
export function creatureTools(session:CreatureAutomationSession):CreatureTool[] {
  const tool=(name:string,description:string,properties:Record<string,unknown>,required:string[],readOnly:boolean,run:(input:Record<string,unknown>)=>unknown):CreatureTool=>({
    name:"m2a_creature_"+name,description,inputSchema:{type:"object",properties,required,additionalProperties:false},annotations:{readOnlyHint:readOnly},
    execute:async input=>{
      try {
        if(!input||typeof input!=="object"||Array.isArray(input)||Object.keys(input).some(k=>!(k in properties))||required.some(k=>!(k in input)))throw Error("M2A-AGENT-INVALID-ARGUMENTS");
        return JSON.stringify({ok:true,result:await run(input)});
      }catch(e){return JSON.stringify({ok:false,error:structuredFailure(e)});}
    },
  });
  const mutation=(name:string,description:string,properties:Record<string,unknown>,required:string[])=>tool(name,description,{expectedRevision:revision,...properties},["expectedRevision",...required],false,a=>session.start(name,a));
  return [
    tool("status","Read the current source identity, selected chain, revision, asynchronous job, failure and export admission. Poll this after a mutation returns a job ID.",{},[],true,()=>session.state()),
    mutation("load_source","Load an owner-selected GLB from this Studio origin, verify SHA-256, and inspect it with the Creature pipeline. Replaces the source atomically and invalidates prior rig/export.",{url:text,sha256:{type:"string",pattern:"^[a-f0-9]{64}$"},sourceForward:{type:"string",enum:["POSITIVE_Z","NEGATIVE_Z","POSITIVE_X","NEGATIVE_X"]}},["url","sha256","sourceForward"]),
    mutation("select_supermodel","Select any supported retail supermodel by resref. Resolve its exact inherited chain through the Studio KEY/BIF reader, in memory; never write retail payloads.",{resref:{type:"string",pattern:"^[a-z0-9_]{1,16}$"},nwnRootUrl:text},["resref","nwnRootUrl"]),
    mutation("prepare","Prepare the immutable reference rig and skin weights using the existing Creature worker. Keep saved corrections. Quality gates remain enabled; failure is not an exportable model.",{},[]),
    mutation("set_authoring","Validate and apply a source/chain-bound schema-v2 authoring document with component/region/vertex weight corrections, including a blocked editable draft. Reference transforms are immutable and export gates remain enforced.",{authoringJson:text},["authoringJson"]),
    mutation("import_authoring","Import a large source/chain-bound authoring JSON from this Studio origin, verify its SHA-256, and apply it through the same Creature worker. The returned quality status may still require corrections.",{url:text,sha256:{type:"string",pattern:"^[a-f0-9]{64}$"}},["url","sha256"]),
    mutation("preview","Build a diagnostic binary MDL, read it back and evaluate inherited motion using the Creature worker. Export admission is separate from successful preview creation.",{},[]),
    mutation("build","Build the admitted Creature product through the existing worker. Requires full offline admission and SHA-256 of owner-selected appearance.2da. Returns artifact metadata; does not install or run NWN.",{appearanceUrl:text,appearanceSha256:{type:"string",pattern:"^[a-f0-9]{64}$"},identityJson:text},["appearanceUrl","appearanceSha256","identityJson"]),
    tool("report","Read a paged JSON report, authoring document or target rig. Optional JSON Pointer path selects a subtree without dumping geometry. Read failure for the exact blocker.",{name:{type:"string",enum:["source","preparation","authoring","targetRig","preview","product","failure"]},offset:{type:"integer",minimum:0},limit:{type:"integer",minimum:1,maximum:32000},path:{type:"string",maxLength:1024}},["name"],true,a=>session.report(a.name as Parameters<typeof session.report>[0],a.offset as number|undefined,a.limit as number|undefined,a.path as string|undefined)),
    tool("artifact","Read a bounded base64 chunk of a generated artifact by ID. Verify its full SHA-256 after assembling chunks. Retail payloads cannot be retrieved here.",{id:text,offset:{type:"integer",minimum:0},limit:{type:"integer",minimum:1,maximum:49152}},["id"],true,a=>session.artifact(a.id as string,a.offset as number|undefined,a.limit as number|undefined)),
  ];
}
export interface CreatureToolBridge { listTools():Omit<CreatureTool,"execute">[]; executeTool(name:string,args:Record<string,unknown>):Promise<string> }
export function registerCreatureWebMCP(session:CreatureAutomationSession, host:Window=window) {
  const tools=creatureTools(session);
  const context=(host.document as Document & {modelContext?:Context}).modelContext
    ?? (host.navigator as Navigator & {modelContext?:Context}).modelContext;
  const controller=new AbortController();const registered:string[]=[];let disposed=false;
  const bridge:CreatureToolBridge={listTools:()=>tools.map(({execute:_,...metadata})=>metadata),executeTool:async(name,args)=>{
    if(disposed)throw Error("M2A-AGENT-BRIDGE-DISPOSED");
    const tool=tools.find(t=>t.name===name);if(!tool)throw Error("M2A-AGENT-UNKNOWN-TOOL");return tool.execute(args);
  }};
  const extended=host as Window & {m2aCreatureTools?:CreatureToolBridge};
  if(extended.m2aCreatureTools)throw Error("M2A-AGENT-BRIDGE-ALREADY-REGISTERED");
  extended.m2aCreatureTools=bridge;
  const unregister=(name:string)=>{try{context?.unregisterTool?.(name);}catch{/* native abort already unregistered */}};
  const dispose=()=>{disposed=true;controller.abort();for(const name of registered)unregister(name);if(extended.m2aCreatureTools===bridge)delete extended.m2aCreatureTools;};
  const ready=(async()=>{
    if(!context)return {mode:"documented-local-bridge",registrationError:undefined};
    try {
      for(const tool of tools){if(disposed)break;await context.registerTool(tool,{signal:controller.signal});if(disposed){unregister(tool.name);break;}registered.push(tool.name);}
      return {mode:disposed?"disposed":"native-webmcp",registrationError:undefined};
    }catch(e){controller.abort();for(const name of registered)unregister(name);return {mode:"documented-local-bridge",registrationError:e instanceof Error?e.message:String(e)};}
  })();
  return {ready,dispose};
}
