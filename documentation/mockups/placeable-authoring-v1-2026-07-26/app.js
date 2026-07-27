const body = document.body;
const sourceHeight = 0.6;
const sourceWidth = 0.5502764;
const sourceDepth = 0.479369;

const heightInput = document.querySelector("#target-height");
const preset = document.querySelector("#size-preset");
const yawInput = document.querySelector("#yaw");
const modelStage = document.querySelector("#model-stage");
const heightRuler = document.querySelector(".height-ruler");
const widthRuler = document.querySelector(".width-ruler");

function format(value, digits = 2) {
  return Number(value).toFixed(digits);
}

function applyHeight(nextValue) {
  const height = Math.min(10, Math.max(.1, Number(nextValue) || sourceHeight));
  const scale = height / sourceHeight;
  const width = sourceWidth * scale;
  const depth = sourceDepth * scale;
  heightInput.value = format(height);
  document.querySelector("#height-label").textContent = `${format(height)} m`;
  document.querySelector("#status-height").textContent = `${format(height)} m`;
  document.querySelector("#status-scale").textContent = `${format(scale, 4)}×`;
  document.querySelector("#output-height")?.replaceChildren(`${format(height)} m`);
  document.querySelector("#output-width").textContent = `${format(width)} m`;
  document.querySelector("#output-depth").textContent = `${format(depth)} m`;
  document.querySelector("#output-scale").textContent = `${format(scale, 4)}×`;
  document.querySelector("#width-label").textContent = `${format(width)} m`;
  document.querySelector("#pwk-width").textContent = `${format(width)} m`;
  document.querySelector("#pwk-depth").textContent = `${format(depth)} m`;
  document.querySelector("#diagnostic-size-summary").textContent =
    `Target height ${format(height)} m is within the freestanding-device range.`;
  updateResolvedSummary();

  const visual = Math.max(.55, Math.min(1.22, height / 2.2));
  modelStage.style.width = `${310 * visual}px`;
  modelStage.style.height = `${430 * visual}px`;
  heightRuler.style.height = `${360 * visual}px`;
  widthRuler.style.width = `${238 * visual}px`;
  widthRuler.style.left = `calc(52% - ${119 * visual}px)`;
}

function updateResolvedSummary() {
  document.querySelector("#resolved-transform-summary").textContent =
    `${format(heightInput.value)} m high · grounded · yaw ${yawInput.value}°`;
}

document.querySelectorAll("[data-panel]").forEach((button) => {
  if (!(button instanceof HTMLButtonElement)) return;
  button.addEventListener("click", () => {
    const panel = button.dataset.panel;
    body.dataset.panel = panel;
    document.querySelectorAll(".inspector-tab").forEach((item) =>
      item.classList.toggle("inspector-tab--active", item === button));
    document.querySelectorAll("[data-panel-content]").forEach((item) =>
      item.classList.toggle("inspector-panel--active", item.dataset.panelContent === panel));
    if (panel === "collision") document.querySelector("#toggle-pwk").checked = true;
  });
});

document.querySelectorAll("[data-tool]").forEach((button) => {
  button.addEventListener("click", () => {
    body.dataset.tool = button.dataset.tool;
    document.querySelectorAll(".tool-button").forEach((item) =>
      item.classList.toggle("tool-button--active", item === button));
  });
});

document.querySelectorAll("[data-camera]").forEach((button) => {
  button.addEventListener("click", () => {
    body.dataset.camera = button.dataset.camera;
    document.querySelectorAll("[data-camera]").forEach((item) =>
      item.classList.toggle("segmented--active", item === button));
    document.querySelector("#camera-label").textContent =
      `Isometric · ${button.dataset.camera === "actual" ? "actual scale" : "fit model"}`;
  });
});

document.querySelectorAll(".view-button[data-view]").forEach((button) => {
  button.addEventListener("click", () => {
    document.querySelectorAll(".view-button").forEach((item) =>
      item.classList.toggle("view-button--active", item === button));
    const names = { iso: "Isometric", front: "Front", right: "Right", back: "Back", top: "Top", nwn: "NWN camera" };
    document.querySelector("#camera-label").textContent =
      `${names[button.dataset.view]} · ${body.dataset.camera === "actual" ? "actual scale" : "fit model"}`;
    document.querySelector("#viewport").dataset.view = button.dataset.view;
  });
});

heightInput.addEventListener("input", () => applyHeight(heightInput.value));
preset.addEventListener("change", () => applyHeight(preset.value));
document.querySelectorAll("[data-nudge]").forEach((button) => button.addEventListener("click", () => {
  applyHeight(Number(heightInput.value) + Number(button.dataset.nudge));
}));

document.querySelectorAll("[data-yaw]").forEach((button) => button.addEventListener("click", () => {
  yawInput.value = button.dataset.yaw;
  document.querySelectorAll("[data-yaw]").forEach((item) =>
    item.classList.toggle("quick-button--active", item === button));
  updateResolvedSummary();
}));
yawInput.addEventListener("input", updateResolvedSummary);

document.querySelector("#toggle-player").addEventListener("change", (event) => {
  document.querySelector("#player-reference").classList.toggle("is-hidden", !event.target.checked);
});
document.querySelector("#toggle-tile").addEventListener("change", (event) => {
  document.querySelector(".tile-outline").classList.toggle("is-hidden", !event.target.checked);
});
document.querySelector("#toggle-pwk").addEventListener("change", (event) => {
  document.querySelector("#pwk-footprint").style.opacity = event.target.checked ? "1" : "";
});
document.querySelector("#show-pwk").addEventListener("click", () => {
  document.querySelector("#toggle-pwk").checked = true;
  document.querySelector("#pwk-footprint").style.opacity = "1";
});
document.querySelector("#inspect-right").addEventListener("click", () => {
  document.querySelector('.view-button[data-view="right"]').click();
});
document.querySelector("#frame-model").addEventListener("click", () => {
  document.querySelector('[data-camera="fit"]').click();
});
document.querySelector("#snap").addEventListener("click", (event) => {
  event.currentTarget.classList.toggle("snap-button--active");
});
document.querySelector("#reset").addEventListener("click", () => {
  preset.value = "2.2";
  yawInput.value = "180";
  applyHeight(2.2);
  document.querySelector('[data-camera="actual"]').click();
});

window.addEventListener("keydown", (event) => {
  if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) return;
  const key = event.key.toLowerCase();
  const tool = key === "q" ? "select" : key === "w" ? "move" : key === "e" ? "rotate" : key === "r" ? "size" : undefined;
  if (tool) document.querySelector(`[data-tool="${tool}"]`)?.click();
});

applyHeight(2.2);
