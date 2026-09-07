"""Build a labeled, deterministic comparison board for the ship wood audit."""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(r"C:\Projects\meshy2aurora")
REFERENCE = ROOT / "artifacts/material-separation/ship-wood-comparison/reference-wooden-sailboat-deck-v3-basecolor-only.png"
TARGET = ROOT / "artifacts/material-separation/tlc-ship-under-construction-v2/offline-preview-v7-reference-contrast/retextured-side-xy.png"
OUTPUT = ROOT / "artifacts/material-separation/ship-wood-comparison/reference-vs-target-v7.png"


def font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    path = Path(r"C:\Windows\Fonts\segoeui.ttf")
    return ImageFont.truetype(str(path), size) if path.is_file() else ImageFont.load_default()


def main() -> None:
    if OUTPUT.exists():
        raise SystemExit(f"refusing to overwrite: {OUTPUT}")
    images = [Image.open(path).convert("RGB") for path in (REFERENCE, TARGET)]
    panel_width, panel_height = 900, 579
    panels = [image.resize((panel_width, panel_height), Image.Resampling.LANCZOS) for image in images]
    canvas = Image.new("RGB", (panel_width * 2, 680), (20, 23, 25))
    canvas.paste(panels[0], (0, 54))
    canvas.paste(panels[1], (panel_width, 54))
    draw = ImageDraw.Draw(canvas)
    title_font = font(28)
    note_font = font(20)
    draw.text((24, 12), "REFERENCJA: wooden-sailboat-deck — base color only", font=title_font, fill=(235, 232, 224))
    draw.text((panel_width + 24, 12), "NASZ PIPELINE: TLC ship — Material Separation V2", font=title_font, fill=(235, 232, 224))
    draw.text((24, 642), "1 971 346 trójkątów · wyłączone normal/PBR · wyraźne deski i ciemne szczeliny", font=note_font, fill=(188, 190, 188))
    draw.text((panel_width + 24, 642), "152 574 trójkąty · Wood/Rope/Sail/Cloth/Metal · bez cleanup · source UV0", font=note_font, fill=(188, 190, 188))
    draw.line((panel_width, 0, panel_width, 680), fill=(96, 102, 104), width=2)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(OUTPUT, format="PNG", optimize=True)
    print(OUTPUT)


if __name__ == "__main__":
    main()
