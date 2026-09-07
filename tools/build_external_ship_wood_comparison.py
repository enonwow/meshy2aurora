"""Build a labeled comparison board from an external preview and the TLC ship.

The external image is used only as a visual reference. No model or texture
payload from the third-party asset is copied into Meshy2Aurora output.
"""

import argparse
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFont


ROOT = Path(r"C:\Projects\meshy2aurora")
REFERENCE = ROOT / (
    "artifacts/material-separation/ship-wood-comparison/"
    "external-half-built-viking-boat/render-01.jpg"
)
TARGET = ROOT / (
    "artifacts/material-separation/tlc-ship-under-construction-v2/"
    "offline-preview-v9-external-reference-midtones/retextured-side-xy.png"
)
OUTPUT = ROOT / (
    "artifacts/material-separation/ship-wood-comparison/"
    "external-half-built-viking-vs-tlc-v9.png"
)


def font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    path = Path(r"C:\Windows\Fonts\segoeui.ttf")
    return ImageFont.truetype(str(path), size) if path.is_file() else ImageFont.load_default()


def crop_foreground(image: Image.Image, padding: int = 36) -> Image.Image:
    """Crop a mostly uniform render background while retaining a safe margin."""

    image = image.convert("RGB")
    background_color = image.getpixel((0, 0))
    background = Image.new("RGB", image.size, background_color)
    difference = ImageChops.difference(image, background).convert("L")
    mask = difference.point(lambda value: 255 if value > 12 else 0)
    bbox = mask.getbbox()
    if bbox is None:
        return image
    left, top, right, bottom = bbox
    return image.crop(
        (
            max(0, left - padding),
            max(0, top - padding),
            min(image.width, right + padding),
            min(image.height, bottom + padding),
        )
    )


def fit_panel(image: Image.Image, width: int, height: int) -> Image.Image:
    image = crop_foreground(image)
    image.thumbnail((width, height), Image.Resampling.LANCZOS)
    panel = Image.new("RGB", (width, height), (28, 31, 33))
    panel.paste(image, ((width - image.width) // 2, (height - image.height) // 2))
    return panel


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", type=Path, default=TARGET)
    parser.add_argument("--output", type=Path, default=OUTPUT)
    parser.add_argument(
        "--label", default="OUR MODEL: TLC ship - Material Separation V2 / v9"
    )
    parser.add_argument(
        "--note",
        default="152,574 triangles - source atlas - no cleanup - current wood grade",
    )
    args = parser.parse_args()
    target = args.target if args.target.is_absolute() else ROOT / args.target
    output = args.output if args.output.is_absolute() else ROOT / args.output
    if output.exists():
        raise SystemExit(f"refusing to overwrite: {output}")
    for path in (REFERENCE, target):
        if not path.is_file():
            raise SystemExit(f"missing input: {path}")

    panel_width, panel_height = 900, 590
    reference = fit_panel(Image.open(REFERENCE), panel_width, panel_height)
    target = fit_panel(Image.open(target), panel_width, panel_height)
    canvas = Image.new("RGB", (panel_width * 2, 710), (20, 23, 25))
    canvas.paste(reference, (0, 58))
    canvas.paste(target, (panel_width, 58))

    draw = ImageDraw.Draw(canvas)
    title_font = font(27)
    note_font = font(19)
    draw.text(
        (22, 13),
        "EXTERNAL REFERENCE: Half Built Viking Boat",
        font=title_font,
        fill=(235, 232, 224),
    )
    draw.text(
        (panel_width + 22, 13),
        args.label,
        font=title_font,
        fill=(235, 232, 224),
    )
    draw.text(
        (22, 662),
        "17,233 polygons - game-ready preview - separate planks, grain and fasteners",
        font=note_font,
        fill=(188, 190, 188),
    )
    draw.text(
        (panel_width + 22, 662),
        args.note,
        font=note_font,
        fill=(188, 190, 188),
    )
    draw.line((panel_width, 0, panel_width, 710), fill=(96, 102, 104), width=2)

    output.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(output, format="PNG", optimize=True)
    print(output)


if __name__ == "__main__":
    main()
