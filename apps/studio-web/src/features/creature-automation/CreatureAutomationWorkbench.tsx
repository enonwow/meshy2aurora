import { useEffect, useState } from "react";
import { StudioWorkerClient } from "../../worker/client";
import { SourceViewport } from "../preview/SourceViewport";
import { SupermodelPreviewViewport } from "../supermodels/SupermodelPreviewViewport";
import { CreatureAutomationSession } from "./session";
import { registerCreatureWebMCP } from "./webmcp";
import { useOwnedPreviewMaterial } from "./useOwnedPreviewMaterial";
import "./creature-automation.css";

export function CreatureAutomationWorkbench() {
  const [session,setSession]=useState<CreatureAutomationSession>();
  const [,refresh]=useState(0);
  const [mode,setMode]=useState("Łączenie narzędzi…");
  const [localError,setLocalError]=useState<string>();
  const params=new URLSearchParams(location.search);
  const [url,setUrl]=useState(params.get("source")??"");
  const [hash,setHash]=useState(params.get("sha256")??"");
  const [forward,setForward]=useState("POSITIVE_Z");
  const [resref,setResref]=useState(params.get("supermodel")??"c_wolf");
  const [root,setRoot]=useState("/__m2a_nwn_reference");
  const [authoring,setAuthoring]=useState("");
  const [appearance,setAppearance]=useState("");
  const [appearanceHash,setAppearanceHash]=useState("");
  const [identity,setIdentity]=useState("");
  useEffect(()=>{
    const current=new CreatureAutomationSession(new StudioWorkerClient(),location.href);setSession(current);
    const unsubscribe=current.subscribe(()=>{refresh(n=>n+1);if(current.source){setUrl(current.sourceUrl??"");setHash(current.source.sourceSha256);setForward(current.sourceForward);}if(current.reference)setResref(current.reference.resref);});let unmounted=false;const bridge=registerCreatureWebMCP(current);
    void bridge.ready.then(result=>{if(unmounted)return;setMode(result.mode==="native-webmcp"?"WebMCP aktywny":"Lokalny interfejs narzędzi aktywny; przeglądarka bez natywnego WebMCP");if(result.registrationError)setLocalError(result.registrationError);}).catch(e=>{if(!unmounted)setLocalError(String(e));});
    return()=>{unmounted=true;bridge.dispose();unsubscribe();current.dispose();};
  },[]);
  const state=session?.state();
  const ownedMaterial=useOwnedPreviewMaterial(session?.source);
  const start=(op:string,args:Record<string,unknown>={})=>{try{setLocalError(undefined);session?.start(op,{...args,expectedRevision:session.revision});}catch(e){setLocalError(String(e));}};
  const disabled=!session||state?.busy;
  const saveReport=()=>{if(!session)return;const packet={state:session.state(),reports:Object.fromEntries(session.reports)};download(new Blob([JSON.stringify(packet,null,2)],{type:"application/json"}),"creature-session-report.json");};
  return <main className="creature-automation">
    <header><div><p className="eyebrow">Meshy2Aurora Studio</p><h1>Creature — model i supermodel</h1><p>Źródło → rzeczywisty szkielet → wagi → kontrola ruchu → eksport.</p></div><a href="./">Wróć do Studio</a></header>
    <p className="creature-automation__mode">{mode}</p>
    <div className="creature-automation__layout">
      <aside>
        <section><h2>1. Źródło</h2><label>URL modelu w Studio<input aria-label="URL modelu" value={url} onChange={e=>setUrl(e.target.value)}/></label><label>SHA-256 źródła<input aria-label="SHA-256 źródła" value={hash} onChange={e=>setHash(e.target.value)}/></label><label>Kierunek pyska w GLB<select value={forward} onChange={e=>setForward(e.target.value)}>{["POSITIVE_Z","NEGATIVE_Z","POSITIVE_X","NEGATIVE_X"].map(v=><option key={v}>{v}</option>)}</select></label><button disabled={disabled} onClick={()=>start("load_source",{url,sha256:hash,sourceForward:forward})}>Wczytaj i sprawdź źródło</button></section>
        <section><h2>2. Supermodel</h2><label>Nazwa modelu<input aria-label="Supermodel" value={resref} onChange={e=>setResref(e.target.value)}/></label><details><summary>Lokalne zasoby NWN</summary><input aria-label="URL zasobów NWN" value={root} onChange={e=>setRoot(e.target.value)}/></details><button disabled={disabled} onClick={()=>start("select_supermodel",{resref,nwnRootUrl:root})}>Wybierz supermodel</button></section>
        <section><h2>3. Dopasowanie i kontrola</h2><button disabled={disabled||!state?.source||!state.supermodel} onClick={()=>start("prepare")}>Przygotuj rig i wagi</button><button disabled={disabled||!session?.reports.has("authoring")} onClick={()=>start("preview")}>Sprawdź odziedziczony ruch</button><details><summary>Korekty komponentów i wag</summary><p>Dokument musi należeć do tego źródła i dokładnego szkieletu. Przeguby referencji pozostają niezmienne.</p><textarea aria-label="Dokument korekt" value={authoring} onChange={e=>setAuthoring(e.target.value)}/><button disabled={disabled||!authoring} onClick={()=>start("set_authoring",{authoringJson:authoring})}>Zastosuj korekty</button></details></section>
        <section><h2>4. Eksport</h2><p>{state?.canExport?"Kontrole offline dopuściły eksport.":"Eksport wymaga pozytywnej kontroli deformacji."}</p><details><summary>Tożsamość i bazowe appearance.2da</summary><label>URL appearance.2da<input value={appearance} onChange={e=>setAppearance(e.target.value)}/></label><label>SHA-256 appearance.2da<input value={appearanceHash} onChange={e=>setAppearanceHash(e.target.value)}/></label><label>Identity JSON<textarea value={identity} onChange={e=>setIdentity(e.target.value)}/></label></details><button disabled={disabled||!state?.canExport} onClick={()=>start("build",{appearanceUrl:appearance,appearanceSha256:appearanceHash,identityJson:identity})}>Zbuduj produkt Creature</button></section>
      </aside>
      <article>
        <section aria-label="Stan pipeline Creature"><h2>Stan pipeline’u</h2><p role="status">{state?.busy?"Trwa: "+state.job?.operation:state?.phase??"Uruchamianie…"}</p>{state?.source?<p>Źródło: <strong>{state.source.fileName}</strong> · {state.source.byteLength.toLocaleString("pl-PL")} B · SHA-256 <code>{state.source.sha256}</code></p>:null}{state?.supermodel?<p>Supermodel: <strong>{state.supermodel.resref}</strong> · łańcuch sprawdzony w KEY/BIF</p>:null}{state?.error?<div role="alert"><strong>Pipeline zatrzymał przygotowanie modelu.</strong><pre>{JSON.stringify(state.error,null,2)}</pre><p>Nie powstał zatwierdzony model. Błąd wymaga korekty w pipeline lub danych wejściowych.</p></div>:null}{localError?<p role="alert">{localError}</p>:null}<button onClick={saveReport} disabled={!session}>Zapisz raport sesji</button></section>
        {session?.source?<section><h2>Twój model źródłowy</h2><SourceViewport input={session.source} sourceForward={session.sourceForward}/></section>:null}
        {session?.reference?<section><h2>{session.preview?"Odczyt wyniku z odziedziczonymi animacjami":"Rzeczywisty supermodel — referencja"}</h2>{session.preview&&ownedMaterial.error?<p role="alert">{ownedMaterial.error}</p>:null}<SupermodelPreviewViewport reports={session.reference.reports} carrier={session.preview??session.reference.reports[0]} materialResolver={session.preview?ownedMaterial.resolver:undefined} detail={session.preview?"Geometria i rig z MDL; podgląd tekstury z GLB. Test NWN nie został wykonany.":"Oryginalny szkielet i animacje; to nie jest wynik konwersji psa"}/></section>:null}
        {session?.artifacts.length?<section><h2>Artefakty</h2>{session.artifacts.map(a=><button key={a.artifactId} onClick={()=>download(new Blob([a.bytes],{type:a.mediaType}),a.fileName)}>{a.fileName}</button>)}</section>:null}
        <details><summary>Narzędzia dla agenta</summary><p>Natywna rejestracja WebMCP, gdy przeglądarka ją obsługuje. Dostępny jest też jawny interfejs <code>window.m2aCreatureTools.listTools()</code> i <code>executeTool(name, args)</code>. Każda zmiana wymaga aktualnego expectedRevision. Odczytaj status po uruchomieniu operacji.</p><p>Brak obejścia limitów jakości, zapisu do instalacji gry i uruchamiania NWN.</p></details>
      </article>
    </div>
  </main>;
}
function download(blob:Blob,name:string){const url=URL.createObjectURL(blob);const a=document.createElement("a");a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
