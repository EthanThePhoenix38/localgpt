# RFC: Publishing, Templates & Gallery — Gap Closure

**Status:** Draft
**Author:** Yi
**Date:** 2026-03-22
**Target crates:** `localgpt-gen`, web frontend (new)
**Depends on:** Phase 1 (complete), Phase 2 (complete)

---

## 1. Summary

Phase 3 has a **solid export foundation** (RON save/load, glTF/GLB, self-contained HTML/Three.js) and a **working local gallery** (egui overlay with search/filter/load). The distribution layer — web platform, templates, publish pipeline, onboarding — is the remaining work.

### 1.1 What's Already Done

| System | Status | Notes |
|--------|--------|-------|
| `gen_save_world` | Functional | RON manifest + assets + screenshots + history |
| `gen_load_world` | Functional | Full state restore, gallery integration |
| `gen_export_world` | Functional | glTF/GLB with PBR materials, hierarchy |
| `gen_export_html` | Functional | Self-contained Three.js viewer with audio, behaviors, OG tags |
| Gallery scanner | Functional | Scans skills/, returns metadata + thumbnails |
| Gallery UI | Functional | egui overlay, G key, filter/search/load |
| Terrain tools | Functional | gen_add_terrain, gen_add_water, gen_add_path, gen_add_foliage |
| `gen_fork_world` | Functional | Copy world with attribution (new) |

### 1.2 What's Missing

| System | Status | Effort | Notes |
|--------|--------|--------|-------|
| ZIP packaging | Not started | 1d | Bundle world dir into distributable .zip |
| Web viewer | Not started | 3-5d | Next.js + Three Fiber frontend loading world format |
| Publish pipeline | Not started | 3d | API endpoint + static hosting (S3/Cloudflare) |
| Web gallery | Not started | 5d | Browse/discover worlds from browser |
| 10 starter templates | Not started | 5d | Curated world skills bundled with distribution |
| Onboarding flow | Not started | 2d | Theme selection → generate → customize → publish |

---

## 2. Architecture

### 2.1 Current (local-only, works today)

```
LocalGPT Gen (Bevy desktop)
  ├── gen_save_world → skills/{name}/world.ron + assets/
  ├── gen_load_world → restore from skills/{name}/
  ├── gen_export_html → export/index.html (self-contained Three.js)
  ├── gen_export_world → export/scene.glb (standard 3D format)
  ├── gen_fork_world → copy + attribution
  └── Gallery UI (G key) → browse local skills/
```

### 2.2 Target (web distribution)

```
LocalGPT Gen (Bevy desktop)
  ↓ gen_publish_world
Publish API (stateless)
  ├── POST /worlds → upload HTML + manifest → S3/R2
  ├── GET /worlds/{id} → redirect to CDN
  └── GET /gallery → list published worlds
  ↓
Static CDN (Cloudflare R2 / S3)
  ├── /{id}/index.html (self-contained Three.js viewer)
  ├── /{id}/manifest.json (metadata for gallery)
  └── /{id}/thumbnail.png (preview image)
  ↓
Web Gallery (static site or Next.js)
  ├── Grid view with thumbnails
  ├── Search/filter by tags
  ├── "Open in browser" → loads Three.js viewer
  └── "Fork in LocalGPT" → deep link to gen_load_world
```

### 2.3 Key Insight: HTML Export IS the Web Viewer

`gen_export_html` already produces a **self-contained Three.js viewer** with:
- 3D rendering with PBR materials
- Animated behaviors (orbit, spin, bob, etc.)
- Procedural audio via Web Audio API
- OrbitControls for interactive viewing
- Open Graph meta tags for social sharing
- Responsive iframe embedding

This means we don't need a separate web viewer — **the exported HTML IS the viewer**. Publishing reduces to: upload the HTML file + thumbnail to static hosting, generate a URL.

---

## 3. Implementation Plan

### 3.1 ZIP Packaging (1 day)

Add a `gen_package_world` tool that bundles the world directory into a `.zip`:

```
world-name.zip
├── world.ron           # Manifest
├── SKILL.md            # Description
├── index.html          # Self-contained viewer (from gen_export_html)
├── scene.glb           # Standard 3D format
├── thumbnail.png       # Preview image
└── assets/             # Referenced meshes
```

Implementation: Use the `zip` crate (already in Rust ecosystem). Combine `gen_save_world` + `gen_export_html` + `gen_export_world` outputs into a single archive.

### 3.2 Publish Pipeline (3 days)

Simplest viable approach: **Cloudflare R2 (S3-compatible) + Workers**.

1. `gen_publish_world` MCP tool:
   - Run `gen_export_html` internally
   - Upload `index.html` + `thumbnail.png` + `manifest.json` to R2
   - Return shareable URL: `https://worlds.localgpt.app/{id}/`

2. Cloudflare Worker (edge function):
   - Serves files from R2
   - Adds CORS headers for iframe embedding
   - Returns gallery JSON for `/api/gallery`

3. No database needed — R2 metadata serves as the gallery index.

### 3.3 Starter Templates (5 days)

Create 5 initial templates as world skill directories:

| # | Template | Theme | Key Features |
|---|----------|-------|-------------|
| 1 | Willowmere Village | Medieval fantasy | Buildings, paths, NPCs, ambient forest |
| 2 | Space Station | Sci-fi | Corridors, doors, hum emitters, zero-G zone |
| 3 | Zen Garden | Peaceful | Water, foliage, stone paths, wind ambient |
| 4 | Desert Oasis | Adventure | Terrain, water pool, palm foliage, cave |
| 5 | Crystal Cave | Fantasy | Glowing materials, echo ambient, collectibles |

Each template is generated via MCP tool calls, tested with player walkthrough, then saved as a skill. Distributed in `workspace/templates/`.

### 3.4 Onboarding Flow (2 days)

First-run experience in the gen mode:

1. Detect empty `skills/` directory → show welcome screen
2. Present template picker (5 cards with thumbnails)
3. "Use template" → `gen_fork_world` + `gen_load_world`
4. "Surprise me" → agent generates random world from prompt
5. After generation: "Publish to web?" → `gen_publish_world`

Implementation: egui overlay system, similar to gallery UI.

### 3.5 Web Gallery (5 days, separate project)

Static Next.js site at `worlds.localgpt.app`:

- Grid view with world cards (thumbnail, title, entity count)
- Click → opens HTML viewer in iframe
- Search/filter by tags
- "Fork in LocalGPT" button → copies manifest URL for local import
- Built from R2 gallery API

This is a **separate project** — not part of the Bevy crate.

---

## 4. Phase 3 → Phase 4 Transition Criteria

Phase 3 is **complete** when:

1. At least 3 starter templates exist and load correctly
2. `gen_fork_world` works (copy + attribution) ✅ Done
3. `gen_export_html` produces embeddable viewers ✅ Done
4. Terrain tools verified in-engine (terrain, water, path, foliage)
5. A world can go from generation → save → export → viewable URL

### Acceptable Deferrals

- Full web gallery platform (can launch with just HTML export + manual upload)
- Publish API (nice-to-have, not blocking Phase 4)
- Onboarding flow (can be added incrementally)

---

## 5. Estimated Effort

| Item | Estimate | Priority | Blocks |
|------|----------|----------|--------|
| ZIP packaging | 1d | Medium | Publish pipeline |
| Publish pipeline (R2 + Worker) | 3d | Medium | Web gallery |
| 5 starter templates | 5d | High | Onboarding |
| Onboarding flow | 2d | Medium | Templates |
| Web gallery (Next.js) | 5d | Low | Publish pipeline |
| In-engine terrain testing | 1d | High | Nothing |
| **Total** | **17d** | | |
| **MVP (templates + terrain test)** | **6d** | | |
