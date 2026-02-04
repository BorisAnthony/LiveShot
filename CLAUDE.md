# LiveShot - Claude Development Instructions

## Project Overview
LiveShot is a command-line tool written in Rust for capturing screenshots of web pages and YouTube live streams. It uses headless Chrome for browser automation and provides a CLI interface with support for full-page captures, multiple formats, quality control, high-DPI screenshots, and video-aware capture.

Forked from [PageShot](https://github.com/kremilly/PageShot) by Kremilly.

**Current Version**: 0.1.0

## Architecture

### Core Dependencies
- **headless_chrome** (1.0.18): Browser automation for screenshot capture
- **clap** (4.5.49): Command-line argument parsing with derive macros
- **anyhow** (1.0.100): Simplified error handling
- **tokio** (1.48.0): Async runtime (full features enabled)

### Project Structure
```
LiveShot/
├── src/
│   ├── main.rs          # CLI args, browser setup, screenshot capture
│   └── youtube.rs       # YouTube mode: consent, ads, play, theatre mode
├── Cargo.toml           # Package configuration
├── README.md            # User documentation
├── LICENSE              # MIT License
└── CLAUDE.md            # This file
```

## Code Organization

### src/main.rs
- **Args struct**: CLI argument definitions using clap derive macros
  - `url`: Target web page URL (required)
  - `width`: Viewport width (default: 1920)
  - `height`: Viewport height (default: 1080)
  - `output`: Output filename (default: "screenshot.png")
  - `full_page`: Boolean flag for full-page capture (default: false)
  - `format`: Output format - "png", "jpeg", or "webp" (default: "png")
  - `quality`: JPEG/WebP quality 0-100 (default: 85)
  - `scale`: Device scale factor / pixel ratio (default: 1.0)
  - `youtube`: Enable YouTube mode (default: false)
  - `wait_timeout`: Max seconds for video readiness in YouTube mode (default: 30)
  - `silent`: Suppress output (default: false)

- **main function**: Core flow
  1. Parse CLI arguments, format, quality, scale
  2. Configure Chrome launch (adds `--autoplay-policy=no-user-gesture-required` when `--youtube`)
  3. If `--youtube`: enable stealth mode (before navigation)
  4. Navigate to URL
  5. If `--youtube`: run `youtube::prepare()`
  6. Full-page resize if needed
  7. Set device metrics override
  8. Capture screenshot and write to file

### src/youtube.rs
YouTube mode preparation, all in a single `prepare()` entry point:
- `prepare(tab, deadline, timeout_secs)`: Runs the full YouTube sequence:
  1. Dismiss GDPR consent dialog (best-effort, multiple selectors)
  2. Wait for pre-roll ads to finish (polls `#movie_player.ad-showing`)
  3. Wait for `<video>` element and attempt playback via `video.play()` and YouTube's `playVideo()` API
  4. Activate theatre mode (clicks `.ytp-size-button`)
  5. Move mouse off player and wait for controls overlay to fade (~3s)
- `poll_js(tab, js, expect, deadline)`: Private helper that polls a boolean JS expression until it matches or deadline expires

**Note**: YouTube functions depend on YouTube's current DOM structure (CSS classes, element IDs). These will need updating if YouTube changes their interface.

## Development Guidelines

### Code Style
- Use Rust 2021 edition conventions
- Leverage `anyhow::Result` for error propagation
- Use `?` operator for clean error handling
- Keep site-specific logic in dedicated modules, not in `main.rs`

### When Adding Features
1. **CLI arguments**: Add new fields to `Args` struct with appropriate clap attributes
2. **Browser options**: Extend `LaunchOptions` configuration as needed
3. **Site-specific logic**: Create a new module (like `youtube.rs`) rather than adding to `main.rs`
4. **Error handling**: Use `anyhow::Result` and provide clear error messages with diagnostics

### Testing Considerations
- Test with various URL types (HTTP/HTTPS, redirects, etc.)
- Validate viewport dimension edge cases
- Test file system permissions and path handling
- Verify Chrome binary detection across platforms
- **YouTube testing**: Test `--youtube` with live streams, VODs, and pages with pre-roll ads
- **Timeout testing**: Verify `--wait-timeout` behaviour when video fails to load
- **Format testing**: Verify PNG, JPEG, WebP produce valid files
- **Scale factor testing**: Verify 1x, 2x, 3x produce expected pixel dimensions

## Build & Release
- **Development**: `cargo build`
- **Release**: `cargo build --release`
- **Install**: `cargo install --path .`
- **Test**: `cargo test` (when tests are added)
- **Clean build**: `cargo clean && cargo build` (fixes rust-analyzer macro issues)

## Version History

### v0.1.0 (Current)
Initial release as LiveShot, forked from PageShot v0.2.0:
- All PageShot v0.2.0 features (full-page, formats, quality, scale)
- `--youtube` flag for YouTube live stream capture (stealth, consent, ads, playback, theatre mode, controls hiding)
- `--autoplay-policy=no-user-gesture-required` Chrome flag for video autoplay
- Code split into modules: `main.rs`, `youtube.rs`

### PageShot v0.2.0 (upstream)
- Full-page screenshots, multiple formats (PNG/JPEG/WebP), quality control, device scale factor

### PageShot v0.1.2 (upstream, initial)
- Basic viewport-only PNG screenshots

## Troubleshooting

### rust-analyzer Macro Errors
If you see "proc-macro panicked" errors with clap_derive:
```bash
cargo clean && cargo build
```

### Chrome Binary Not Found
The tool auto-detects Chrome/Chromium. If detection fails:
- macOS: Install Chrome from official website
- Linux: Install chromium or google-chrome package
- Ensure Chrome is in PATH or standard installation location

### YouTube Features Stopped Working
YouTube may have changed their DOM structure. Check `src/youtube.rs` for:
- CSS selectors (`.ytp-size-button`, `#movie_player`, consent button selectors)
- Class names (`ad-showing`)
- API methods (`playVideo`)

## External Resources
- Repository: https://github.com/BorisAnthony/LiveShot
- Upstream: https://github.com/kremilly/PageShot
