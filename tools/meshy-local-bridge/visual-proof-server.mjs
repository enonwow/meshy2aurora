import { readFile } from "node:fs/promises";
import { createLocalBridge } from "./index.mjs";

if (!process.argv.includes("--visual-proof")) {
  throw new Error("Pass --visual-proof to run the synthetic local visual-proof Bridge.");
}

const glb = await readFile(new URL("../../test-assets/meshy/incoming/s1-static-prop-1500.glb", import.meta.url));
const thumbnail = Uint8Array.from(Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=", "base64"));
const task = {
  id: "proof-refine-task", type: "text-to-3d-refine", status: "SUCCEEDED", prompt: "Verified stone lantern proof asset",
  created_at: 1763481600000, finished_at: 1763481660000, consumed_credits: 20,
  thumbnail_url: "https://proof.local/thumbnail.png?signature=private",
  model_urls: { glb: "https://proof.local/model.glb?signature=private" },
};

const meshFetch = async (url) => {
  if (url === "https://proof.local/model.glb?signature=private") return new Response(glb, { headers: { "Content-Type": "model/gltf-binary" } });
  if (url === "https://proof.local/thumbnail.png?signature=private") return new Response(thumbnail, { headers: { "Content-Type": "image/png" } });
  if (url.endsWith("/openapi/v1/balance")) return new Response(JSON.stringify({ balance: 120 }));
  if (url.includes("/openapi/v2/text-to-3d?page_num=1&page_size=50&sort_by=-created_at")) return new Response(JSON.stringify([task]));
  if (url.endsWith("/openapi/v2/text-to-3d/proof-refine-task")) return new Response(JSON.stringify(task));
  return new Response(JSON.stringify({ message: "Unexpected synthetic proof request." }), { status: 500 });
};

const bridge = createLocalBridge({
  apiKey: "visual-proof-local-key",
  pairingCode: "visual-proof-pair",
  allowedOrigin: "http://127.0.0.1:5179",
  meshFetch,
  startRuns: false,
});
const origin = await bridge.listen(43120);
process.stdout.write(`Synthetic visual-proof Bridge listening at ${origin}\n`);
