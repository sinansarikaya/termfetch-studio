#!/usr/bin/env python3
import json
import os
from PIL import Image, ImageDraw, ImageFont

FONT_PATH = "/usr/share/fonts/TTF/MesloLGLNerdFontMono-Regular.ttf"
if not os.path.exists(FONT_PATH):
    FONT_PATH = "/usr/share/fonts/TTF/JetBrainsMonoNLNerdFont-SemiBold.ttf"

CELL_W = 9
CELL_H = 19
FONT_SIZE = 14

font = ImageFont.truetype(FONT_PATH, FONT_SIZE)

with open("qa/screens_dump.json", "r") as f:
    dumps = json.load(f)

screens_by_name = {d["name"]: d for d in dumps}

def render_screen(screen_data, out_path):
    w = screen_data["width"]
    h = screen_data["height"]
    img_w = w * CELL_W + 16
    img_h = h * CELL_H + 16
    img = Image.new("RGB", (img_w, img_h), (14, 16, 24))
    draw = ImageDraw.Draw(img)

    for cell in screen_data["cells"]:
        x0 = 8 + cell["x"] * CELL_W
        y0 = 8 + cell["y"] * CELL_H
        x1 = x0 + CELL_W
        y1 = y0 + CELL_H
        sym = cell["symbol"]
        fg = tuple(cell["fg"])
        bg = tuple(cell["bg"])

        # Fill background
        draw.rectangle([x0, y0, x1, y1], fill=bg)

        # Handle half-block characters perfectly
        if sym == "▀":
            # Upper half fg, lower half bg
            draw.rectangle([x0, y0, x1, y0 + CELL_H // 2], fill=fg)
        elif sym == "▄":
            # Lower half fg, upper half bg
            draw.rectangle([x0, y0 + CELL_H // 2, x1, y1], fill=fg)
        elif sym == "█":
            draw.rectangle([x0, y0, x1, y1], fill=fg)
        elif sym and sym != " ":
            # Draw text glyph
            draw.text((x0 + 1, y0 + 1), sym, font=font, fill=fg)

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    img.save(out_path)
    print(f"Rendered: {out_path} ({img_w}x{img_h})")

# Render 7 main screens
screen_names = [
    "dashboard",
    "themes",
    "presets",
    "logos",
    "modules",
    "settings",
    "preview-focus",
]

for name in screen_names:
    if name in screens_by_name:
        out_p = f"qa/screenshots/{name}.png"
        render_screen(screens_by_name[name], out_p)

# Render Theme Gallery (6x5 grid of 30 themes)
theme_ids = [
    "catppuccin-mocha", "catppuccin-latte", "rose-pine", "kanagawa", "everforest", "dracula",
    "nord", "tokyo-night", "gruvbox-dark", "one-dark", "synthwave", "oceanic-next",
    "monokai-pro", "palenight", "monochrome", "solarized-light", "ayu-light", "gruvbox-light",
    "one-light", "sakura", "lavender", "candy", "matrix", "cyberpunk",
    "naruto", "pokemon", "spiderman", "doom", "valorant", "minecraft"
]

def render_grid_gallery(prefix, cell_scale_w, cell_scale_h, out_path, title):
    cols = 6
    rows = 5
    first = screens_by_name[f"{prefix}_{theme_ids[0]}"]
    cw = first["width"]
    ch = first["height"]

    card_pixel_w = cw * cell_scale_w
    card_pixel_h = ch * cell_scale_h
    pad_x = 24
    pad_y = 36
    margin = 32

    total_w = margin * 2 + cols * card_pixel_w + (cols - 1) * pad_x
    total_h = margin * 2 + 50 + rows * card_pixel_h + (rows - 1) * pad_y

    img = Image.new("RGB", (total_w, total_h), (12, 14, 20))
    draw = ImageDraw.Draw(img)

    # Header title
    title_font = ImageFont.truetype(FONT_PATH, 20)
    draw.text((margin, margin - 10), title, font=title_font, fill=(240, 240, 255))

    for idx, tid in enumerate(theme_ids):
        c = idx % cols
        r = idx // cols
        x0 = margin + c * (card_pixel_w + pad_x)
        y0 = margin + 50 + r * (card_pixel_h + pad_y)

        # Label above card
        lbl_font = ImageFont.truetype(FONT_PATH, 11)
        name_clean = tid.replace("-", " ").title()
        draw.text((x0, y0 - 18), f"{idx+1}. {name_clean}", font=lbl_font, fill=(180, 190, 210))

        # Card border
        draw.rectangle([x0 - 2, y0 - 2, x0 + card_pixel_w + 1, y0 + card_pixel_h + 1], outline=(45, 52, 70), width=1)

        key = f"{prefix}_{tid}"
        if key in screens_by_name:
            sdata = screens_by_name[key]
            for cell in sdata["cells"]:
                cx0 = x0 + cell["x"] * cell_scale_w
                cy0 = y0 + cell["y"] * cell_scale_h
                cx1 = cx0 + cell_scale_w
                cy1 = cy0 + cell_scale_h
                fg = tuple(cell["fg"])
                bg = tuple(cell["bg"])
                # Half blocks
                draw.rectangle([cx0, cy0, cx1, cy0 + cell_scale_h // 2], fill=fg)
                draw.rectangle([cx0, cy0 + cell_scale_h // 2, cx1, cy1], fill=bg)

    img.save(out_path)
    print(f"Rendered: {out_path} ({total_w}x{total_h})")

render_grid_gallery("gallery", 3, 6, "qa/screenshots/theme-gallery.png", "TermFetch Studio v2 — Pixel Art 2.0 (Gallery Mode)")
render_grid_gallery("compact", 6, 14, "qa/screenshots/compact-gallery.png", "TermFetch Studio v2 — Pixel Art 2.0 (Compact Mode)")

# Render Pixel Art Comparison for the 8 focus themes
focus_themes = [
    ("everforest", "Everforest Dark 🍃", "Layered pine forest & spirit lantern"),
    ("one-dark", "One Dark 🌌", "Cosmic twin peaks & cyan ridge lighting"),
    ("monokai-pro", "Monokai Pro 👾", "6-stripe spectrum rainbow & digital grid"),
    ("tokyo-night", "Tokyo Night 🌃", "Centered Tokyo Tower & cyber skyline"),
    ("nord", "Nord Aurora ❄️", "Turquoise aurora ribbon & glacial horn"),
    ("dracula", "Dracula 🧛", "Gothic vampire castle & blood-moon"),
    ("catppuccin-mocha", "Catppuccin Mocha ☕", "Peach crescent moon & dreamy lilac cloud"),
    ("synthwave", "Synthwave '84 🎸", "Retro striped sun & cyan perspective grid"),
]

def render_comparison():
    card_w = 400
    card_h = 100
    row_h = 140
    pad = 30
    total_w = pad * 2 + card_w * 2 + 50
    total_h = pad * 2 + 70 + len(focus_themes) * row_h

    img = Image.new("RGB", (total_w, total_h), (11, 13, 20))
    draw = ImageDraw.Draw(img)

    title_font = ImageFont.truetype(FONT_PATH, 22)
    col_font = ImageFont.truetype(FONT_PATH, 14)
    item_font = ImageFont.truetype(FONT_PATH, 13)
    desc_font = ImageFont.truetype(FONT_PATH, 11)

    draw.text((pad, pad), "TermFetch Studio v2 — Pixel Art 2.0 Profile Comparison", font=title_font, fill=(245, 245, 255))
    draw.text((pad, pad + 38), "Direct comparison between Gallery Mode (Themes Page) and Compact Mode (Dashboard & Sidebar)", font=desc_font, fill=(140, 150, 175))

    # Column headers
    gx = pad
    cx = pad + card_w + 50
    draw.text((gx, pad + 65), "GALLERY MODE (Full 64x16 Landscape Canvas)", font=col_font, fill=(137, 180, 250))
    draw.text((cx, pad + 65), "COMPACT MODE (26x5 High-Contrast Silhouette)", font=col_font, fill=(166, 227, 161))

    for idx, (tid, name, desc) in enumerate(focus_themes):
        y0 = pad + 100 + idx * row_h

        # Theme Name and Description
        draw.text((gx, y0 - 18), f"{idx+1}. {name} — {desc}", font=item_font, fill=(230, 235, 245))

        # Render Gallery card
        gdata = screens_by_name.get(f"gallery_{tid}")
        if gdata:
            cw = gdata["width"]
            ch = gdata["height"]
            scale_w = card_w // cw
            scale_h = card_h // ch
            draw.rectangle([gx - 1, y0 - 1, gx + cw * scale_w, y0 + ch * scale_h], outline=(50, 60, 80), width=1)
            for cell in gdata["cells"]:
                px0 = gx + cell["x"] * scale_w
                py0 = y0 + cell["y"] * scale_h
                px1 = px0 + scale_w
                py1 = py0 + scale_h
                fg = tuple(cell["fg"])
                bg = tuple(cell["bg"])
                draw.rectangle([px0, py0, px1, py0 + scale_h // 2], fill=fg)
                draw.rectangle([px0, py0 + scale_h // 2, px1, py1], fill=bg)

        # Render Compact card
        cdata = screens_by_name.get(f"compact_{tid}")
        if cdata:
            cw = cdata["width"]
            ch = cdata["height"]
            scale_w = card_w // cw
            scale_h = card_h // ch
            draw.rectangle([cx - 1, y0 - 1, cx + cw * scale_w, y0 + ch * scale_h], outline=(50, 60, 80), width=1)
            for cell in cdata["cells"]:
                px0 = cx + cell["x"] * scale_w
                py0 = y0 + cell["y"] * scale_h
                px1 = px0 + scale_w
                py1 = py0 + scale_h
                fg = tuple(cell["fg"])
                bg = tuple(cell["bg"])
                draw.rectangle([px0, py0, px1, py0 + scale_h // 2], fill=fg)
                draw.rectangle([px0, py0 + scale_h // 2, px1, py1], fill=bg)

    out_p = "qa/screenshots/pixel-art-comparison.png"
    img.save(out_p)
    print(f"Rendered: {out_p} ({total_w}x{total_h})")

render_comparison()
print("All screenshots generated successfully!")
