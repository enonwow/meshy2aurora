const body = document.body;
const screenButtons = [...document.querySelectorAll("[data-screen-target]")];
const previewButtons = [...document.querySelectorAll("[data-preview-target]")];
const tabButtons = [...document.querySelectorAll("[data-tab-target]")];

function setScreen(screen) {
  body.dataset.screen = screen;
  document.querySelectorAll("[data-screen-content]").forEach((node) => {
    node.classList.toggle("screen--active", node.dataset.screenContent === screen);
  });
  document.querySelectorAll(".workflow-step").forEach((node) => {
    const target = node.dataset.screenTarget;
    node.classList.toggle("workflow-step--active", target === screen);
    node.classList.toggle(
      "workflow-step--done",
      (screen === "prepare" && target === "source") ||
        (screen === "review" && ["source", "prepare"].includes(target)),
    );
  });
  window.location.hash = screen;
}

function setPreview(preview) {
  body.dataset.preview = preview;
  previewButtons.forEach((node) => {
    node.classList.toggle("view-button--active", node.dataset.previewTarget === preview);
  });
  const label = document.querySelector("#preview-label");
  if (label) label.textContent = preview === "composed" ? "Composed" : preview === "exploded" ? "Exploded parts" : "Inventory icon";
}

function setTab(tab) {
  tabButtons.forEach((node) => node.classList.toggle("inspector-tab--active", node.dataset.tabTarget === tab));
  document.querySelectorAll("[data-tab-content]").forEach((node) => {
    node.classList.toggle("inspector-panel--active", node.dataset.tabContent === tab);
  });
}

screenButtons.forEach((button) => button.addEventListener("click", () => setScreen(button.dataset.screenTarget)));
previewButtons.forEach((button) => button.addEventListener("click", () => setPreview(button.dataset.previewTarget)));
tabButtons.forEach((button) => button.addEventListener("click", () => setTab(button.dataset.tabTarget)));

setScreen(["source", "prepare", "review"].includes(window.location.hash.slice(1)) ? window.location.hash.slice(1) : "source");
setPreview("composed");
setTab("assembly");
