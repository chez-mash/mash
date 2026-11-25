#!/usr/bin/env python3
"""Frame the guest's full-screen captures as Raycast Store screenshots.

Usage: store_frame.py SCREEN_DIR OUT_DIR R G B

Each SCREEN_DIR/<n>-<scene>.png is a 2880x1800 capture of the guest at 2x,
with the light Raycast window on a plain wallpaper of colour R G B. This
writes OUT_DIR/mash-<n>.png: 2000x1250, the window and its shadow centred on
that colour, as Raycast's Window Capture lays them out. Only pixels are
copied, never scaled. It stops when it cannot find exactly one window of
Raycast's size, so a notice, a dialog or a missing window fails the run
instead of landing in the Store.
"""

import pathlib
import sys

from PIL import Image, ImageChops

WIDTH, HEIGHT = 2000, 1250
# Raycast's window is 750x475 points; at 2x that is 1500x950 pixels. Allow a
# few pixels for the border.
WINDOW = (1500, 950)
SLACK = 8
# The window's shadow fades out well inside this many pixels.
SHADOW = 120
# The rows between the menu bar and the Dock, in pixels.
BAND = (100, 1600)


def window_box(screen: Image.Image, background: tuple[int, int, int]) -> tuple[int, int, int, int]:
    """The bounding box of the light window: pixels clearly brighter than the wallpaper.

    The translucent menu bar and Dock are brighter too, so only the band
    between them is searched.
    """
    band = screen.crop((0, BAND[0], screen.width, BAND[1]))
    plain = Image.new("RGB", band.size, background)
    brighter = ImageChops.subtract(band, plain).convert("L").point(lambda v: 255 if v > 12 else 0)
    box = brighter.getbbox()
    if box is None:
        raise SystemExit("no window found")
    return box[0], box[1] + BAND[0], box[2], box[3] + BAND[0]


def frame(path: pathlib.Path, out: pathlib.Path, background: tuple[int, int, int]) -> None:
    screen = Image.open(path).convert("RGB")
    if screen.size != (2880, 1800):
        raise SystemExit(f"{path}: {screen.size} is not a 2x capture of 1440x900")
    # Compare against the wallpaper as captured, which colour management may
    # have shifted slightly from the colour asked for.
    actual = screen.getpixel((40, 900))
    if max(abs(a - b) for a, b in zip(actual, background)) > 6:
        raise SystemExit(f"{path}: the wallpaper is {actual}, not {background}")
    left, top, right, bottom = window_box(screen, actual)
    found = (right - left, bottom - top)
    # The window's bottom rows can be nearly the wallpaper's grey (the Read
    # Messages footer is), so trust the bright top and sides and take the
    # height from Raycast's fixed window size.
    if abs(found[0] - WINDOW[0]) > SLACK or not WINDOW[1] - 60 <= found[1] <= WINDOW[1] + SLACK:
        raise SystemExit(f"{path}: found {found} of bright pixels at {(left, top)}, not one {WINDOW} Raycast window")
    bottom = top + WINDOW[1]
    size = (right - left, WINDOW[1])
    region = screen.crop((left - SHADOW, top - SHADOW, right + SHADOW, bottom + SHADOW))
    canvas = Image.new("RGB", (WIDTH, HEIGHT), actual)
    canvas.paste(region, ((WIDTH - size[0]) // 2 - SHADOW, (HEIGHT - size[1]) // 2 - SHADOW))
    canvas.save(out)
    print(f"{out.name}: window {size} from {path.name}")


def main() -> None:
    screens, outdir = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
    background = tuple(int(c) for c in sys.argv[3:6])
    for path in sorted(screens.glob("[0-9]-*.png")):
        frame(path, outdir / f"mash-{path.name.split('-', 1)[0]}.png", background)


if __name__ == "__main__":
    main()
