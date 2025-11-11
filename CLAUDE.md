# PageShot - Claude Development Instructions

## Project Overview
PageShot is a command-line tool written in Rust for capturing screenshots of web pages. It uses headless Chrome for browser automation and provides a powerful CLI interface with support for full-page captures, multiple formats, quality control, and high-DPI (Retina) screenshots.

**Current Version**: 0.2.0

## Architecture

### Core Dependencies
- **headless_chrome** (1.0.18): Browser automation for screenshot capture
- **clap** (4.5.49): Command-line argument parsing with derive macros
- **anyhow** (1.0.100): Simplified error handling
- **tokio** (1.48.0): Async runtime (full features enabled)

### Project Structure
```
PageShot/
├── src/
│   └── main.rs          # Main application logic
├── Cargo.toml           # Package configuration
├── README.md            # User documentation
├── LICENSE              # MIT License
└── CLAUDE.md            # This file
```

## Code Organization

### Current Implementation (src/main.rs:1-145)

#### Imports (lines 1-10)
- Standard library: `std::fs` for file operations
- External crates: clap for CLI, anyhow for errors, headless_chrome for browser automation
- CDP protocols: `Page::CaptureScreenshotFormatOption`, `Emulation` for device metrics

#### Args struct (lines 12-46): CLI argument definitions using clap derive macros
  - `url`: Target web page URL (required)
  - `width`: Viewport width (default: 1920)
  - `height`: Viewport height (default: 1080)
  - `output`: Output filename (default: "screenshot.png")
  - `full_page`: Boolean flag for full-page capture (default: false)
  - `format`: Output format - "png", "jpeg", or "webp" (default: "png")
  - `quality`: JPEG/WebP quality 0-100 (default: 85)
  - `scale`: Device scale factor / pixel ratio (default: 1.0)

#### main function (lines 48-145): Core screenshot capture logic
  1. Parse CLI arguments
  2. Determine output format based on `format` argument (PNG/JPEG/WebP)
  3. Calculate quality parameter (applies only to JPEG/WebP)
  4. Configure Chrome launch options with explicit headless mode
  5. Set viewport window size
  6. Launch headless browser and create new tab
  7. Navigate to URL and wait for page load
  8. Apply device scale factor via `Emulation.SetDeviceMetricsOverride` if not 1.0
  9. Conditional screenshot capture:
     - **Full-page mode**: Evaluate page dimensions via JavaScript, resize viewport to full content size, capture entire page
     - **Viewport mode**: Capture visible viewport only
  10. Write screenshot data to output file

## Development Guidelines

### Code Style
- Use Rust 2021 edition conventions
- Leverage `anyhow::Result` for error propagation
- Use `?` operator for clean error handling
- Keep code concise and idiomatic Rust

### When Adding Features
1. **CLI arguments**: Add new fields to `Args` struct with appropriate clap attributes
2. **Browser options**: Extend `LaunchOptions` configuration as needed
3. **Screenshot options**: Utilize `CaptureScreenshotFormatOption` and screenshot parameters
4. **Error handling**: Use `anyhow::Result` and provide clear error messages

### Implemented Features (v0.2.0)
- ✅ **Multiple output formats**: PNG (lossless), JPEG (lossy), WebP (modern compression)
- ✅ **Full-page screenshots**: Captures entire scrollable content beyond viewport
- ✅ **Quality configuration**: 0-100 quality control for JPEG and WebP formats
- ✅ **Device scale factor**: Retina/HiDPI support with configurable pixel ratio (1x, 2x, 3x, etc.)
- ✅ **Explicit headless mode**: Browser runs in background without UI
- ✅ **Flexible viewport sizing**: Custom width and height for screenshots

### Key Implementation Details

#### Full-Page Capture (src/main.rs:99-120)
Uses JavaScript evaluation to detect full document dimensions:
```javascript
Math.max(document.body.scrollWidth, document.documentElement.scrollWidth)
Math.max(document.body.scrollHeight, document.documentElement.scrollHeight)
```
Then resizes viewport via `tab.set_bounds()` and adds 500ms delay for page adjustment.

#### Format & Quality Selection (src/main.rs:51-58)
Parses format string to `CaptureScreenshotFormatOption` enum, calculates quality parameter (clamped 0-100) only for JPEG/WebP formats.

#### Device Scale Factor (src/main.rs:79-96)
Uses Chrome DevTools Protocol `Emulation.SetDeviceMetricsOverride` to control device pixel ratio:
- Scale 1.0: Standard resolution (800×600 → 800×600 pixels)
- Scale 2.0: Retina 2x (800×600 → 1600×1200 pixels, 4× total pixels)
- Scale 3.0: Ultra HD 3x (800×600 → 2400×1800 pixels, 9× total pixels)

### Potential Future Enhancements
- Batch processing of multiple URLs
- Custom wait times or element selectors
- Custom user agent strings
- Cookie and authentication support
- Retry logic with exponential backoff
- Progress indicators for batch operations
- Configuration file support (JSON/TOML)
- Clipboard integration
- CSS selector-based element capture
- PDF output format
- Viewport screenshots of specific elements
- JavaScript execution before capture
- Network request interception
- Custom HTTP headers

### Testing Considerations
- Test with various URL types (HTTP/HTTPS, redirects, etc.)
- Validate viewport dimension edge cases
- Test file system permissions and path handling
- Verify Chrome binary detection across platforms
- **Format testing**: Verify PNG (lossless), JPEG (quality 0-100), WebP (quality 0-100) produce valid files
- **Full-page testing**: Test with long pages (e.g., news sites, documentation) and pages with dynamic content
- **Scale factor testing**: Verify 1x, 2x, 3x produce expected pixel dimensions
- **Quality comparison**: Compare file sizes at different quality levels (50, 75, 85, 95)
- **Edge cases**: Test scale factors with full-page mode, test various format/quality combinations

### Performance Notes
- Browser launch is the primary bottleneck (~1-2 seconds)
- Consider browser reuse for batch operations (future enhancement)
- Screenshot capture is generally fast (~200-500ms)
- File I/O is negligible for typical image sizes
- **Scale factor impact**: Higher scale factors (2x, 3x) increase:
  - Render time (more pixels to process)
  - Memory usage (larger image buffers)
  - File size (2x scale = ~4× pixels, 3x = ~9× pixels)
- **Format performance**:
  - PNG: Slower encoding, larger files, lossless
  - JPEG: Fast encoding, smaller files at quality 70-85
  - WebP: Best compression ratio, moderate encoding time

### Typical File Sizes (example.com, 1920×1080 viewport)
- PNG: ~20 KB (lossless baseline)
- JPEG Q85: ~22 KB (high quality)
- JPEG Q50: ~17 KB (medium quality, 15% smaller)
- WebP Q90: ~11 KB (best compression, 45% smaller than PNG)

### Typical Scale Factor Output (800×600 viewport)
- 1x: 800×461 pixels, 15 KB
- 2x: 1600×1200 pixels, 36 KB (4× pixel count)
- 3x: 2400×1800 pixels, 57 KB (9× pixel count)

## Build & Release
- **Development**: `cargo build`
- **Release**: `cargo build --release`
- **Install**: `cargo install --path .`
- **Test**: `cargo test` (when tests are added)
- **Clean build**: `cargo clean && cargo build` (fixes rust-analyzer macro issues)

## Version History

### v0.2.0 (Current)
Major feature release with significant enhancements:
- Added full-page screenshot capability via JavaScript dimension detection
- Implemented multiple output formats (PNG, JPEG, WebP)
- Added quality control for lossy formats (JPEG/WebP)
- Implemented device scale factor (Retina/HiDPI support) via Chrome DevTools Protocol Emulation
- Explicit headless mode configuration
- Comprehensive documentation updates

### v0.1.2 (Initial)
Basic screenshot functionality:
- Viewport-only screenshots
- PNG output format only
- Basic CLI with URL, width, height, output parameters

## Troubleshooting

### rust-analyzer Macro Errors
If you see "proc-macro panicked" errors with clap_derive:
```bash
cargo clean && cargo build
```
This clears stale build artifacts and regenerates macro expansion files.

### Chrome Binary Not Found
The tool auto-detects Chrome/Chromium. If detection fails:
- macOS: Install Chrome from official website
- Linux: Install chromium or google-chrome package
- Ensure Chrome is in PATH or standard installation location

### Full-Page Screenshots Cut Off
Some sites use lazy loading. Future enhancements may include:
- Custom wait times before capture
- Scroll simulation to trigger lazy load
- Element-specific waiting conditions

### WebP Not Supported
WebP requires Chrome 32+ (2014). All modern Chrome versions support it. If issues occur, fall back to PNG or JPEG.

## External Resources
- Repository: https://github.com/kremilly/PageShot
- Documentation: https://kremilly.com/docs/pageshot
- Homepage: https://kremilly.com
