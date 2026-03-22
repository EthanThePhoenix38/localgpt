# Headless 4: Gallery UI

**Makes experiment results browsable.** An in-app egui overlay for viewing thumbnails, comparing experiment variations, and loading worlds for further editing. No database — reads directly from the filesystem.

**Source:** RFC-Headless-Gen-Experiment-Pipeline.md, Phase 4 (Section 9)

**Dependencies:** H1.5 (Offscreen Screenshots — for thumbnails), existing `WorldManifest` in `localgpt-world-types`

**Priority:** 5 of 6 (~22h)

---

## Spec H4.1: `scan_world_gallery` — Filesystem Scanner

**Goal:** Scan `workspace/skills/` for world skills and build gallery entries from their metadata.

### Data Model

```rust
pub struct WorldGalleryEntry {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub entity_count: usize,
    pub created_at: Option<DateTime<Utc>>,
    pub thumbnail_path: Option<PathBuf>,
    pub style_tags: Vec<String>,
    pub variation_group: Option<String>,
    pub source: String,  // "interactive", "headless", "experiment", "mcp"
}
```

### Implementation

1. Iterate `workspace/skills/*/`
2. Filter: must have `world.ron` to be a world skill
3. Parse `world.ron` → `WorldManifest` for metadata
4. Find best thumbnail: most recent PNG/JPG in `screenshots/` subdirectory
5. Get creation date from filesystem metadata
6. Sort by creation date, newest first

### Acceptance Criteria

- [ ] Scanner finds all world skills with `world.ron`
- [ ] Metadata extracted: name, description, entity count, tags, source
- [ ] Thumbnail path resolved from `screenshots/` directory
- [ ] Entries sorted newest-first
- [ ] Empty `skills/` directory returns empty list (no crash)

---

## Spec H4.2: `WorldMeta` Extensions

**Goal:** Add fields to `WorldManifest.meta` to support gallery display and experiment tracking.

### New Fields

```rust
pub tags: Option<Vec<String>>,              // style tags for filtering
pub source: Option<String>,                 // generation source
pub variation_group: Option<String>,        // experiment variation group ID
pub variation: Option<(String, String)>,    // axis + value
pub prompt: Option<String>,                 // original generation prompt
pub model: Option<String>,                  // LLM model used
pub generation_duration_ms: Option<u64>,    // how long generation took
pub style_ref: Option<String>,             // style name from memory
```

### Implementation

1. Add fields to `WorldMeta` struct in `localgpt-world-types`
2. All new fields are `Option` with `#[serde(default, skip_serializing_if = "Option::is_none")]`
3. Backward compatible: existing `world.ron` files deserialize without the new fields
4. `gen_save_world` populates these fields from generation context

### Acceptance Criteria

- [ ] New fields serialize/deserialize correctly in RON format
- [ ] Existing `world.ron` files without new fields still load (backward compatible)
- [ ] `gen_save_world` populates source, prompt, model, duration when available

---

## Spec H4.3: Gallery egui Overlay

**Goal:** Render a gallery window in interactive gen mode showing world cards with thumbnails.

### UI Layout

- **Filter bar:** text input for searching by name or tag, plus Refresh button
- **Card grid:** scrollable grid of world cards, each showing:
  - Thumbnail (200x150) or "No Preview" placeholder
  - World name (bold)
  - Metadata line: entity count, source
  - Style tags (small text)
  - "Load" button
- **Window:** resizable egui window, default 800x600

### Implementation

1. `GalleryState` resource: visibility toggle, entries list, filter string, selected index, texture cache
2. `gallery_ui_system`: egui render system, filtered card grid
3. Thumbnail textures: lazy-loaded from PNG files into egui texture handles
4. Card click behavior: set `selected` index, highlight card

### Acceptance Criteria

- [ ] Gallery displays all world skills as cards
- [ ] Filter narrows results by name and tag (case-insensitive)
- [ ] Refresh button rescans the filesystem
- [ ] Thumbnails display correctly (or placeholder when missing)
- [ ] Cards show name, entity count, source, and tags

---

## Spec H4.4: Thumbnail Loading and Caching

**Goal:** Load PNG thumbnails into egui textures efficiently with caching.

### Implementation

1. On first display of a card, load its thumbnail PNG from disk
2. Decode to RGBA pixels via `image` crate
3. Upload to egui as `TextureHandle` via `ctx.load_texture`
4. Cache in `GalleryState.thumbnails` HashMap keyed by path
5. Only load visible thumbnails (scroll-aware lazy loading)

### Acceptance Criteria

- [ ] Thumbnails load without blocking the UI (or with minimal stutter)
- [ ] Cached textures are reused on subsequent frames
- [ ] Missing thumbnails show a placeholder (no crash)

---

## Spec H4.5: Gallery Keybinds and Commands

**Goal:** Wire gallery toggle to keyboard and slash commands.

| Trigger | Action |
|---------|--------|
| `G` key (when not in text input) | Toggle gallery overlay |
| `/gallery` command | Toggle gallery overlay |
| `/gallery refresh` | Rescan skills/ and reload entries |
| `/gallery filter <tag>` | Set filter text |

### Implementation

1. `gallery_toggle_system`: check `KeyCode::KeyG` press, toggle `gallery.visible`
2. On toggle to visible with empty entries: auto-scan workspace
3. Wire `/gallery` slash command in the gen command handler

### Acceptance Criteria

- [ ] `G` key toggles gallery (not triggered while typing in egui text fields)
- [ ] `/gallery` command toggles gallery
- [ ] Gallery auto-scans on first open

---

## Spec H4.6: "Load" Button → `gen_load_world`

**Goal:** Clicking "Load" on a gallery card loads that world into the current scene.

### Implementation

1. On "Load" click, send `gen_load_world` command via the GenBridge
2. Pass the world skill path from the gallery entry
3. Close the gallery overlay after loading
4. Show loading indicator while world loads

### Acceptance Criteria

- [ ] Clicking "Load" loads the selected world into the scene
- [ ] Gallery closes after loading
- [ ] Current scene is replaced (not merged) with the loaded world
