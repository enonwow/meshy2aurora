const SOURCE_SHA256 =
  "d3d1e18adb652524a5b3459620fab278892859ffb86a7fbb38ac4c542e44d4f7";
const APPEARANCE_SHA256 =
  "3dc8505bb5848044659cf44f9fe60e3f7c401db772496c4571ec0fff62a761d6";

function viteFileUrl(relativePath: string) {
  return `/@fs/${__M2A_CANONICAL_REPOSITORY_ROOT__}/${relativePath}`;
}

async function fetchExactFile(
  relativePath: string,
  fileName: string,
  mediaType: string,
  expectedSha256: string,
) {
  const response = await fetch(viteFileUrl(relativePath));
  if (!response.ok) {
    throw new Error(`Could not load ${fileName}: HTTP ${response.status}.`);
  }
  const bytes = await response.arrayBuffer();
  const digest = Array.from(
    new Uint8Array(await crypto.subtle.digest("SHA-256", bytes)),
    (value) => value.toString(16).padStart(2, "0"),
  ).join("");
  if (digest !== expectedSha256) {
    throw new Error(
      `${fileName} does not match the approved visual QA lineage (${digest}).`,
    );
  }
  return new File([bytes], fileName, { type: mediaType });
}

export async function loadVoidCrystalKnightVisualQaFixture() {
  const [source, appearance] = await Promise.all([
    fetchExactFile(
      "sample-3d/void-crystal-knight-h1-v1/source.glb",
      "source.glb",
      "model/gltf-binary",
      SOURCE_SHA256,
    ),
    fetchExactFile(
      "proof-output/void-crystal-knight-h1-v1-demo2-product-v3/generated/appearance.2da",
      "appearance.2da",
      "text/plain",
      APPEARANCE_SHA256,
    ),
  ]);
  return { source, appearance };
}
