# Headless 1.5: Offscreen Screenshots

**Enables visual output from headless generation.** Every experiment produces at least one thumbnail, making gallery browsing and experiment comparison possible without loading full worlds.

**Source:** RFC-Headless-Gen-Experiment-Pipeline.md, Phase 1.5 (Section 6.4)

**Dependencies:** H1 (Headless Bevy Gen Mode)

**Priority:** 2 of 6 (~14h)

---

## Spec H1.5.1: `OffscreenRenderTarget` Resource

**Goal:** Create a Bevy resource that manages an offscreen render texture for screenshot capture.

### Implementation

1. Create `OffscreenRenderTarget` resource with configurable width/height (default 1280x720)
2. Allocate a `TextureDescriptor` with `Bgra8UnormSrgb` format
3. Set `TextureUsages::TEXTURE_BINDING | COPY_SRC | RENDER_ATTACHMENT`
4. Store the `Handle<Image>` for later readback

### Acceptance Criteria

- [ ] Resource initializes with configurable dimensions
- [ ] Render texture is created with correct format and usages
- [ ] Image handle is accessible for camera targeting

---

## Spec H1.5.2: Offscreen Camera Setup

**Goal:** Spawn a camera that renders to the offscreen texture instead of a window.

### Implementation

1. Spawn `Camera3d` with `RenderTarget::Image(image_handle)`
2. Default transform: `from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y)`
3. Mark with `OffscreenCamera` component for identification
4. In headless mode, this is the only camera (no window camera exists)

### Acceptance Criteria

- [ ] Camera renders the scene to the offscreen texture
- [ ] Camera position provides a reasonable default overview of the scene
- [ ] Camera can be repositioned by gen tools (same as interactive camera)

---

## Spec H1.5.3: `capture_offscreen_screenshot` — Render Target to PNG

**Goal:** Read pixels from the offscreen render target and save as a PNG file.

### Implementation

1. Get the `Image` asset from the handle stored in `OffscreenRenderTarget`
2. Convert raw pixel data to `image::RgbaImage` via the `image` crate
3. Handle BGRA → RGBA channel swizzle if needed
4. Save as PNG to the specified path
5. Return error if image not ready (not yet rendered)

### Acceptance Criteria

- [ ] Produces a valid PNG file at the specified path
- [ ] Image dimensions match the configured width/height
- [ ] Colors are correct (no channel swizzle issues)
- [ ] Error case handled gracefully (image not ready)

---

## Spec H1.5.4: Screenshot Integration in Headless Pipeline

**Goal:** Wire screenshot capture into the headless gen flow — capture after world save, before exit.

### Implementation

1. After `gen_save_world` completes, trigger screenshot capture
2. Save to `<output_path>/screenshots/thumbnail.png`
3. Create `screenshots/` directory if it doesn't exist
4. Set `HeadlessCompletionFlag.done = true` only after screenshot is saved
5. If `--screenshot false`, skip capture and exit immediately after save

### Acceptance Criteria

- [ ] Headless gen with default settings produces a thumbnail PNG alongside the world
- [ ] Thumbnail is saved in `<output>/screenshots/thumbnail.png`
- [ ] `--screenshot false` skips thumbnail generation
- [ ] Screenshot dimensions respect `--screenshot-width` and `--screenshot-height`

---

## Spec H1.5.5: Software Rendering Fallback for Headless Linux

**Goal:** Ensure headless gen works on Linux servers without a GPU or display.

### Implementation

1. wgpu with `Backends::all()` enables Vulkan headless surface (`VK_KHR_headless_surface`)
2. If no GPU available, fall back to llvmpipe (Mesa software renderer)
3. Detect fallback and log a warning about degraded performance
4. If neither GPU nor software renderer available, exit with code 3

### Test Plan

1. Test with `DISPLAY=` unset on Linux
2. Test with `LIBGL_ALWAYS_SOFTWARE=1` to force software rendering
3. Verify screenshots are produced (may be slower but correct)

### Acceptance Criteria

- [ ] Headless gen works on Linux without `$DISPLAY` set
- [ ] Software rendering fallback produces valid screenshots
- [ ] Exit code 3 when no rendering backend is available
- [ ] Warning logged when using software rendering
