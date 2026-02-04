# LiveShot

A Rust-built command-line tool for capturing screenshots of web pages and YouTube live streams using headless Chrome.

Forked from the excellent [PageShot](https://github.com/kremilly/PageShot) by Kremilly.

## Note

The YT Live functionality is tied to YT's current HTML DOM. When that changes, this code will need to be updated.

Issues and PRs are welcome.


## Features

- Capture screenshots from any URL.
- Customize viewport width and height.
- Full-page screenshots that capture entire scrollable content.
- Multiple output formats: PNG, JPEG, WebP.
- Quality control for JPEG and WebP formats.
- Device scale factor / pixel ratio control (Retina/HiDPI support).
- YouTube mode (`--youtube`): stealth, consent dismissal, ad waiting, video playback, theatre mode, and controls hiding.
- Simple command-line interface.


## Installation

Requires Rust and Cargo. Clone this repository and build:

```sh
cargo install --path .
```

## Usage

```sh
# Basic screenshot (PNG)
liveshot -u https://example.com --width 1920 --height 1080 -o example.png

# Full-page screenshot
liveshot -u https://example.com -f -o example_fullpage.png

# JPEG with quality control
liveshot -u https://example.com --format jpeg --quality 85 -o example.jpg

# WebP for best compression
liveshot -u https://example.com --format webp --quality 90 -o example.webp

# Full-page JPEG with lower quality for smaller file size
liveshot -u https://example.com -f --format jpeg --quality 70 -o fullpage.jpg

# Retina/HiDPI 2x resolution screenshot (doubles pixel dimensions)
liveshot -u https://example.com --width 800 --height 600 --scale 2.0 -o retina_2x.png

# Ultra HD 3x resolution for maximum clarity
liveshot -u https://example.com --scale 3.0 -o ultra_hd_3x.png

# YouTube live stream (stealth, consent, ads, theatre mode, video playback)
liveshot -u "https://www.youtube.com/watch?v=VIDEO_ID" --youtube -o yt-live.png

# YouTube with custom timeout and format options
liveshot -u "https://www.youtube.com/watch?v=VIDEO_ID" --youtube --wait-timeout 60 --format webp --quality 90 -o yt-live.webp

# Silent mode for scripts (no output, only exit code)
liveshot -u https://example.com -s -o screenshot.png
```

### Arguments

- `-u, --url <URL>`: The URL of the web page to capture.
- `--width <WIDTH>`: The width of the viewport (default: 1920).
- `--height <HEIGHT>`: The height of the viewport (default: 1080).
- `-o, --output <FILE>`: The name of the output file (default: `screenshot.png`).
- `-f, --full-page`: Capture the entire scrollable page content, not just the viewport.
- `--format <FORMAT>`: Output format - `png`, `jpeg`, or `webp` (default: `png`).
- `--quality <QUALITY>`: Quality for JPEG/WebP, 0-100 where higher is better (default: 85).
- `--scale <SCALE>`: Device scale factor / pixel ratio (default: 1.0). Use 2.0 for Retina 2x, 3.0 for 3x.
- `--youtube`: Enable YouTube mode: stealth, consent dismissal, ad waiting, video playback, theatre mode, controls hiding.
- `--wait-timeout <SECONDS>`: Maximum seconds to wait for video readiness (default: 30). Used with `--youtube`.
- `-s, --silent`: Suppress success message. Useful for scripts and automation.

### YouTube Mode

The `--youtube` flag enables the full YouTube live stream capture pipeline:

1. **Stealth mode**: Masks headless Chrome signals to avoid bot detection.
2. **Consent dismissal**: Clicks through GDPR cookie consent dialogs.
3. **Ad waiting**: Waits for pre-roll ads to finish.
4. **Playback trigger**: Starts video via YouTube's player API and the `<video>` element.
5. **Theatre mode**: Expands the player to fill the page width.
6. **Controls hiding**: Moves the mouse off the player and waits for the overlay to fade.

Use `--wait-timeout` to control how long to wait across all phases (default: 30 seconds). If the video doesn't start within the timeout, LiveShot exits with a diagnostic error message showing the video's state.

### Format Recommendations

- **PNG**: Lossless quality, best for documentation and pixel-perfect captures. Larger file size.
- **JPEG**: Good for general web captures. Use quality 70-85 for balanced size/quality, 90-100 for high quality.
- **WebP**: Modern format with best compression. Recommended for sharing and storage efficiency.

### Scale Factor / Device Pixel Ratio

The `--scale` parameter controls the device pixel ratio, similar to Retina and HiDPI displays:

- **1.0** (default): Standard resolution. An 800×600 viewport produces an 800×600 pixel image.
- **2.0** (Retina): 2x resolution. An 800×600 viewport produces a 1600×1200 pixel image (4× more pixels).
- **3.0** (Ultra HD): 3x resolution. An 800×600 viewport produces a 2400×1800 pixel image (9× more pixels).

**Use Cases:**
- **Scale 1.0**: Fast captures, smaller file sizes, adequate for most uses
- **Scale 2.0**: High-quality captures for print or detailed analysis, matches macOS Retina displays
- **Scale 3.0**: Maximum detail for zooming or professional use, matches iOS device displays

**Note**: Higher scale factors produce larger file sizes but capture text and images with much greater clarity.

## Acknowledgements

Based on [PageShot](https://github.com/kremilly/PageShot) by [Kremilly](https://kremilly.com), licensed under MIT.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
