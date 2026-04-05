#!/usr/bin/env python3
"""Capture a tmux pane with ANSI colors and render to a PNG image."""

import subprocess
import sys
import re
from PIL import Image, ImageDraw, ImageFont

SESSION = sys.argv[1] if len(sys.argv) > 1 else "chess"
OUTPUT = sys.argv[2] if len(sys.argv) > 2 else "/tmp/tmux_screen.png"

# Capture pane with escape sequences
result = subprocess.run(
    ["tmux", "capture-pane", "-t", SESSION, "-p", "-e", "-S", "-40"],
    capture_output=True, text=True
)
raw = result.stdout

# 256-color palette (standard xterm-256color)
PALETTE = [
    # 0-7: standard colors
    (0, 0, 0), (205, 0, 0), (0, 205, 0), (205, 205, 0),
    (0, 0, 238), (205, 0, 205), (0, 205, 205), (229, 229, 229),
    # 8-15: bright colors
    (127, 127, 127), (255, 0, 0), (0, 255, 0), (255, 255, 0),
    (92, 92, 255), (255, 0, 255), (0, 255, 255), (255, 255, 255),
]
# 16-231: 6x6x6 color cube
for r in range(6):
    for g in range(6):
        for b in range(6):
            PALETTE.append((r * 51, g * 51, b * 51))
# 232-255: grayscale
for i in range(24):
    v = 8 + i * 10
    PALETTE.append((v, v, v))

def parse_color(code, is_fg=True):
    """Parse an SGR color code number to RGB."""
    if code == 0:
        return None  # reset
    if 30 <= code <= 37:
        return PALETTE[code - 30]
    if 40 <= code <= 47:
        return PALETTE[code - 40]
    if 90 <= code <= 97:
        return PALETTE[code - 90 + 8]
    if 100 <= code <= 107:
        return PALETTE[code - 100 + 8]
    return None

def parse_ansi(text):
    """Parse text with ANSI escape sequences into a list of (char, fg, bg) per cell."""
    lines = []
    current_line = []
    fg = (229, 229, 229)  # default light gray
    bg = (18, 18, 18)     # dark terminal background

    i = 0
    while i < len(text):
        if text[i] == '\n':
            lines.append(current_line)
            current_line = []
            i += 1
            continue

        if text[i] == '\x1b' and i + 1 < len(text) and text[i + 1] == '[':
            # Parse CSI sequence
            j = i + 2
            while j < len(text) and text[j] not in 'mHJKABCDfsu':
                j += 1
            if j < len(text) and text[j] == 'm':
                params = text[i+2:j]
                codes = [int(x) if x else 0 for x in params.split(';')] if params else [0]

                k = 0
                while k < len(codes):
                    c = codes[k]
                    if c == 0:
                        fg = (229, 229, 229)
                        bg = (18, 18, 18)
                    elif c == 1:
                        pass  # bold - ignore for now
                    elif 30 <= c <= 37:
                        fg = PALETTE[c - 30]
                    elif 40 <= c <= 47:
                        bg = PALETTE[c - 40]
                    elif 90 <= c <= 97:
                        fg = PALETTE[c - 90 + 8]
                    elif 100 <= c <= 107:
                        bg = PALETTE[c - 100 + 8]
                    elif c == 38 and k + 1 < len(codes):
                        if codes[k + 1] == 5 and k + 2 < len(codes):
                            idx = codes[k + 2]
                            if 0 <= idx < len(PALETTE):
                                fg = PALETTE[idx]
                            k += 2
                        elif codes[k + 1] == 2 and k + 4 < len(codes):
                            fg = (codes[k+2], codes[k+3], codes[k+4])
                            k += 4
                    elif c == 48 and k + 1 < len(codes):
                        if codes[k + 1] == 5 and k + 2 < len(codes):
                            idx = codes[k + 2]
                            if 0 <= idx < len(PALETTE):
                                bg = PALETTE[idx]
                            k += 2
                        elif codes[k + 1] == 2 and k + 4 < len(codes):
                            bg = (codes[k+2], codes[k+3], codes[k+4])
                            k += 4
                    elif c == 39:
                        fg = (229, 229, 229)
                    elif c == 49:
                        bg = (18, 18, 18)
                    k += 1
            i = j + 1
            continue

        current_line.append((text[i], fg, bg))
        i += 1

    if current_line:
        lines.append(current_line)

    return lines

def render(lines, output_path):
    """Render parsed lines to a PNG image."""
    font_path = "/System/Library/Fonts/Menlo.ttc"
    font_size = 14
    try:
        font = ImageFont.truetype(font_path, font_size)
    except:
        font = ImageFont.load_default()

    # Measure character size
    bbox = font.getbbox("M")
    char_w = bbox[2] - bbox[0]
    char_h = int(font_size * 1.4)

    # Find max width
    max_cols = max((len(line) for line in lines), default=80)
    num_rows = len(lines)

    bg_color = (18, 18, 18)
    img_w = max_cols * char_w + 20
    img_h = num_rows * char_h + 20

    img = Image.new("RGB", (img_w, img_h), bg_color)
    draw = ImageDraw.Draw(img)

    pad_x, pad_y = 10, 10

    for row_idx, line in enumerate(lines):
        y = pad_y + row_idx * char_h
        for col_idx, (ch, fg, bg) in enumerate(line):
            x = pad_x + col_idx * char_w
            # Draw background
            if bg != bg_color:
                draw.rectangle([x, y, x + char_w, y + char_h], fill=bg)
            # Draw character
            if ch != ' ':
                draw.text((x, y), ch, fill=fg, font=font)

    img.save(output_path)
    print(f"Screenshot saved to {output_path} ({img_w}x{img_h})")

lines = parse_ansi(raw)
render(lines, OUTPUT)
