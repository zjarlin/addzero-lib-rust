#!/usr/bin/env python3
from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter


ROOT = Path(__file__).resolve().parent
MASTER_ICON = ROOT / "icon.png"
PNG_TARGETS = {
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
}
ICO_SIZES = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
ICNS_SIZES = [16, 32, 128, 256, 512]
CANVAS = 1024


def vertical_gradient(size: tuple[int, int], top: tuple[int, int, int], bottom: tuple[int, int, int]) -> Image.Image:
    width, height = size
    base = Image.new("RGBA", size)
    pixels = []
    for y in range(height):
        ratio = y / max(height - 1, 1)
        color = tuple(int(top[i] * (1 - ratio) + bottom[i] * ratio) for i in range(3))
        pixels.extend([(*color, 255)] * width)
    base.putdata(pixels)
    return base


def horizontal_alpha_gradient(size: tuple[int, int], start: int, end: int) -> Image.Image:
    width, height = size
    mask = Image.new("L", size)
    pixels = []
    for _y in range(height):
        for x in range(width):
            ratio = x / max(width - 1, 1)
            pixels.append(int(start * (1 - ratio) + end * ratio))
    mask.putdata(pixels)
    return mask


def draw_glow(base: Image.Image, mask: Image.Image, color: tuple[int, int, int, int], blur: int) -> None:
    glow = Image.new("RGBA", base.size, color)
    blurred = Image.new("RGBA", base.size, (0, 0, 0, 0))
    blurred.paste(glow, mask=mask.filter(ImageFilter.GaussianBlur(blur)))
    base.alpha_composite(blurred)


def make_symbol_mask() -> Image.Image:
    mask = Image.new("L", (CANVAS, CANVAS), 0)
    draw = ImageDraw.Draw(mask)
    draw.line((312, 796, 488, 248), fill=255, width=120)
    draw.line((712, 796, 536, 248), fill=255, width=120)
    draw.rounded_rectangle((388, 474, 638, 560), radius=42, fill=255)
    draw.rounded_rectangle((676, 438, 824, 586), radius=46, fill=255)
    return mask


def render_master_icon() -> Image.Image:
    image = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))

    shadow_mask = Image.new("L", (CANVAS, CANVAS), 0)
    shadow_draw = ImageDraw.Draw(shadow_mask)
    shadow_draw.rounded_rectangle((110, 126, 914, 930), radius=212, fill=255)
    draw_glow(image, shadow_mask, (5, 10, 18, 120), blur=48)

    panel_mask = Image.new("L", (CANVAS, CANVAS), 0)
    panel_draw = ImageDraw.Draw(panel_mask)
    panel_draw.rounded_rectangle((96, 96, 928, 928), radius=208, fill=255)

    panel = vertical_gradient((CANVAS, CANVAS), (13, 19, 29), (26, 32, 45))

    diagonal_wash = Image.new("RGBA", (CANVAS, CANVAS), (246, 191, 76, 0))
    diagonal_wash.putalpha(horizontal_alpha_gradient((CANVAS, CANVAS), 30, 0))
    diagonal_wash = diagonal_wash.rotate(-22, resample=Image.Resampling.BICUBIC)
    panel = Image.alpha_composite(panel, diagonal_wash)

    cool_wash = Image.new("RGBA", (CANVAS, CANVAS), (76, 178, 255, 0))
    cool_wash.putalpha(horizontal_alpha_gradient((CANVAS, CANVAS), 0, 42))
    cool_wash = cool_wash.rotate(18, resample=Image.Resampling.BICUBIC)
    panel = Image.alpha_composite(panel, cool_wash)

    image.paste(panel, mask=panel_mask)

    overlay = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    overlay_draw = ImageDraw.Draw(overlay)
    overlay_draw.arc((184, 168, 884, 868), start=198, end=324, fill=(255, 255, 255, 34), width=18)
    overlay_draw.arc((184, 168, 884, 868), start=24, end=112, fill=(148, 211, 255, 30), width=12)
    overlay_draw.rounded_rectangle((96, 96, 928, 928), radius=208, outline=(255, 255, 255, 54), width=8)
    image.alpha_composite(overlay)

    symbol_shadow_mask = make_symbol_mask()
    shifted_shadow = ImageChops.offset(symbol_shadow_mask, 0, 22)
    draw_glow(image, shifted_shadow, (4, 7, 14, 180), blur=28)

    left_bar = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    left_draw = ImageDraw.Draw(left_bar)
    left_draw.line((312, 796, 488, 248), fill=(247, 163, 55, 255), width=120)
    left_draw.line((296, 760, 470, 232), fill=(255, 224, 170, 72), width=34)
    image.alpha_composite(left_bar)

    right_bar = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    right_draw = ImageDraw.Draw(right_bar)
    right_draw.line((712, 796, 536, 248), fill=(74, 188, 255, 255), width=120)
    right_draw.line((730, 758, 554, 228), fill=(194, 241, 255, 72), width=34)
    image.alpha_composite(right_bar)

    bridge = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    bridge_draw = ImageDraw.Draw(bridge)
    bridge_draw.rounded_rectangle((388, 474, 638, 560), radius=42, fill=(247, 243, 232, 255))
    bridge_draw.rounded_rectangle((400, 488, 622, 528), radius=20, fill=(255, 255, 255, 84))
    image.alpha_composite(bridge)

    module_shadow_mask = Image.new("L", (CANVAS, CANVAS), 0)
    module_shadow_draw = ImageDraw.Draw(module_shadow_mask)
    module_shadow_draw.rounded_rectangle((676, 438, 824, 586), radius=46, fill=255)
    draw_glow(image, module_shadow_mask, (181, 255, 154, 108), blur=18)

    module = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    module_draw = ImageDraw.Draw(module)
    module_draw.rounded_rectangle((676, 438, 824, 586), radius=46, fill=(165, 237, 94, 255))
    module_draw.rounded_rectangle((694, 456, 804, 520), radius=24, fill=(240, 255, 215, 86))
    module_draw.rounded_rectangle((712, 552, 786, 568), radius=8, fill=(79, 117, 32, 88))
    image.alpha_composite(module)

    flare = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    flare_draw = ImageDraw.Draw(flare)
    flare_draw.ellipse((192, 132, 432, 344), fill=(255, 255, 255, 18))
    flare = flare.filter(ImageFilter.GaussianBlur(22))
    image.alpha_composite(flare)

    return image


def save_png_variants(master: Image.Image) -> None:
    master.save(MASTER_ICON)
    for filename, size in PNG_TARGETS.items():
        master.resize((size, size), Image.Resampling.LANCZOS).save(ROOT / filename)


def save_ico(master: Image.Image) -> None:
    master.save(ROOT / "icon.ico", sizes=ICO_SIZES)


def save_icns(master: Image.Image) -> None:
    if shutil.which("iconutil") is None:
        raise RuntimeError("iconutil is required to build icon.icns on macOS")

    with tempfile.TemporaryDirectory(prefix="aio-iconset-") as temp_dir:
        iconset = Path(temp_dir) / "icon.iconset"
        iconset.mkdir(parents=True, exist_ok=True)

        for base_size in ICNS_SIZES:
            normal_name = iconset / f"icon_{base_size}x{base_size}.png"
            retina_name = iconset / f"icon_{base_size}x{base_size}@2x.png"
            master.resize((base_size, base_size), Image.Resampling.LANCZOS).save(normal_name)
            master.resize((base_size * 2, base_size * 2), Image.Resampling.LANCZOS).save(retina_name)

        subprocess.run(
            ["iconutil", "-c", "icns", str(iconset), "-o", str(ROOT / "icon.icns")],
            check=True,
        )


def main() -> None:
    ROOT.mkdir(parents=True, exist_ok=True)
    master = render_master_icon()
    save_png_variants(master)
    save_ico(master)
    save_icns(master)


if __name__ == "__main__":
    main()
