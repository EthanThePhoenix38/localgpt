# AI Integration 3: MCP Multimodal Input for Image-Guided World Generation

**Closes the gap between text-only prompts and visual intent.** Extends worldgen tools to accept images (screenshots, concept art, photos, panoramas) as generation input, enabling "make a world that looks like this" workflows. Builds on existing multimodal embedding infrastructure and the MCP 2026 multimodal roadmap.

**Source:** MCP 2026 roadmap includes multimodal support (images, video, audio). World Labs World API already accepts images, 360° panoramas, and video as input for world generation. LocalGPT Gen should be ready when MCP multimodal support lands.

**Dependencies:** `gen_screenshot` (implemented), multimodal embeddings (`10-multimodal-embed` in GAPS.md, implemented), worldgen tools (WG1–WG7, implemented), MCP multimodal transport (2026 roadmap — spec can proceed with local file paths as interim transport)

**Priority within AI series:** 3 of 3 (after AI1 asset generation and AI2 NPC intelligence — this extends the generation input surface rather than output capabilities)

---

## Motivation

Text-only prompts are a bottleneck for expressing visual intent. Users know what they want a world to look like — they have screenshots, concept art, mood boards, reference photos — but translating visual ideas into text descriptions loses information. Every competitor that accepts image input (World Labs, Tencent ASI World, Roblox 4D) reports higher user satisfaction and output quality compared to text-only.

LocalGPT Gen already has the building blocks:
- `gen_screenshot` captures visual state (output direction)
- `gen_evaluate_scene` sends screenshots to VLMs for quality assessment (screenshot → LLM direction)
- `embed_multimodal()` processes images via Gemini embeddings (image understanding)
- Worldgen pipeline (WG1–WG7) decomposes generation into structured steps

The missing piece: using images as **input** to guide generation, not just as **output** for evaluation.

---

## Spec AI3.1: `gen_image_to_layout` — Reference Image to Blockout Plan

**Goal:** Accept a reference image and produce a blockout layout plan (`gen_plan_layout` format) that approximates the spatial structure visible in the image. This bridges the gap between a visual reference and the existing blockout-first pipeline.

### MCP Tool Schema

```json
{
  "name": "gen_image_to_layout",
  "description": "Analyze a reference image and generate a blockout layout plan matching its spatial structure",
  "parameters": {
    "image": { "type": "string", "required": true, "description": "Path to reference image (PNG/JPEG) or base64-encoded image data" },
    "prompt": { "type": "string", "optional": true, "description": "Additional text guidance to supplement the image (e.g., 'focus on the castle in the background')" },
    "scale": { "type": "enum", "values": ["small", "medium", "large"], "default": "medium", "description": "Target world scale: small (single room), medium (building/courtyard), large (landscape)" },
    "style": { "type": "enum", "values": ["match", "blockout", "stylized"], "default": "match", "description": "match: reproduce the image's style; blockout: gray-box only; stylized: interpret freely" }
  }
}
```

### Response

Returns a layout plan compatible with `gen_apply_blockout`:

```json
{
  "plan": {
    "regions": [
      {
        "name": "main_castle",
        "bounds": { "min": [-20, 0, -15], "max": [20, 25, 15] },
        "role": "structure",
        "tier": "hero",
        "description": "Central castle structure with two towers, derived from image foreground"
      }
    ],
    "ground_plane": "terrain",
    "estimated_entities": 45
  },
  "image_analysis": {
    "detected_elements": ["castle", "two towers", "courtyard", "surrounding wall", "forest background"],
    "spatial_depth": "three layers: foreground wall, midground castle, background forest",
    "dominant_palette": ["#4a5568", "#2d3748", "#48bb78", "#edf2f7"],
    "perspective": "three-quarter view, slightly elevated"
  }
}
```

### Implementation

1. **Image loading:** Accept file paths (local images) or base64 strings (MCP multimodal transport when available). Use the `image` crate (already a dependency) for decoding. Resize to max 1568px on longest edge (matching existing `7-image-resize` behavior) before sending to the LLM.

2. **VLM analysis:** Send the image to the configured LLM with a structured prompt requesting:
   - Spatial decomposition: what objects exist and their approximate relative positions
   - Depth layering: foreground, midground, background separation
   - Scale estimation: room-scale, building-scale, or landscape-scale
   - Color palette extraction: dominant colors for material assignment
   - Perspective identification: camera angle for matching the reference view

3. **Layout generation:** Map the VLM analysis to `gen_plan_layout` region format:
   - Each detected spatial element → a blockout region with bounds, role, and tier
   - Depth layers → Z-axis positioning
   - Scale parameter → multiplier on region bounds
   - Color palette → stored in region metadata for later material assignment

4. **Prompt augmentation:** When `prompt` is provided, prepend it to the VLM analysis prompt as priority guidance. This lets users steer interpretation ("ignore the sky, focus on the ruins").

### Acceptance Criteria

- [ ] Tool accepts PNG/JPEG file paths and base64 image data
- [ ] Returns a valid layout plan that `gen_apply_blockout` can consume
- [ ] Image analysis identifies at least major spatial elements (structures, terrain, vegetation)
- [ ] Scale parameter produces appropriately-sized regions (small ≈ 10m², medium ≈ 100m², large ≈ 1000m²)
- [ ] Text prompt supplements (not replaces) image analysis
- [ ] Works with the existing blockout → populate → evaluate pipeline

---

## Spec AI3.2: `gen_match_style` — Apply Visual Style from Reference Image

**Goal:** Given a reference image and an existing scene, adjust materials, lighting, and atmosphere to match the reference image's visual style. This is the "make my world look like this" tool — it operates on an already-built scene rather than generating from scratch.

### MCP Tool Schema

```json
{
  "name": "gen_match_style",
  "description": "Adjust scene materials, lighting, and atmosphere to match the visual style of a reference image",
  "parameters": {
    "image": { "type": "string", "required": true, "description": "Path to style reference image or base64 data" },
    "scope": { "type": "enum", "values": ["all", "lighting", "materials", "atmosphere"], "default": "all", "description": "Which aspects of the scene to adjust" },
    "intensity": { "type": "f32", "default": 0.8, "min": 0.0, "max": 1.0, "description": "How strongly to match the reference (0.0 = no change, 1.0 = exact match attempt)" },
    "entities": { "type": "array", "items": "string", "optional": true, "description": "Specific entity IDs to restyle (default: all entities)" }
  }
}
```

### Response

```json
{
  "changes": {
    "lighting": {
      "ambient_color": "#1a1a2e",
      "directional_color": "#e0c090",
      "directional_intensity": 2.5,
      "shadow_softness": 0.7
    },
    "materials_modified": 12,
    "atmosphere": {
      "fog_color": "#2d3748",
      "fog_density": 0.02,
      "sky_preset": "sunset"
    }
  },
  "style_notes": "Applied warm sunset palette from reference. Increased shadow softness and added fog for depth."
}
```

### Implementation

1. **Style extraction:** Send the reference image to the VLM requesting:
   - Lighting direction, color temperature, and intensity
   - Dominant material properties (roughness, metallicity patterns)
   - Atmospheric effects (fog, haze, time of day)
   - Color grading (warm/cool, saturated/muted, contrast level)

2. **Scene modification:** Apply extracted style via existing tools:
   - `gen_set_light` for directional/ambient lighting changes
   - `gen_set_environment` for sky and fog
   - `gen_modify_entity` / `gen_bulk_modify` for material property adjustments
   - `gen_set_ambience` for audio atmosphere matching (e.g., sunset → crickets)

3. **Intensity scaling:** The `intensity` parameter interpolates between current scene values and target values. At 0.5, each property moves halfway toward the reference.

4. **Scoped application:** `scope` parameter limits which systems are touched. A user might want only the lighting from a reference image while keeping their custom materials.

### Acceptance Criteria

- [ ] Extracts lighting, material, and atmosphere properties from reference images
- [ ] Applies changes through existing scene modification tools (no new rendering code)
- [ ] Intensity parameter produces visually proportional results
- [ ] Scope parameter correctly limits modifications to the requested systems
- [ ] Before/after screenshots show measurable style convergence toward the reference
- [ ] Existing entity geometry and positions are preserved (style-only changes)

---

## Spec AI3.3: `gen_reference_board` — Multi-Image Context for Generation

**Goal:** Allow users to attach multiple reference images as persistent context for a generation session. Instead of single-image tools, this establishes a "mood board" that influences all subsequent generation calls — blockout, population, material assignment, and evaluation.

### MCP Tool Schema

```json
{
  "name": "gen_reference_board",
  "description": "Manage reference images that influence world generation in the current session",
  "parameters": {
    "action": { "type": "enum", "values": ["add", "remove", "list", "clear"], "required": true },
    "image": { "type": "string", "optional": true, "description": "Path or base64 image data (required for 'add')" },
    "label": { "type": "string", "optional": true, "description": "Label for the reference (e.g., 'color palette', 'architecture style', 'vegetation reference')" },
    "weight": { "type": "f32", "default": 1.0, "min": 0.0, "max": 2.0, "description": "Influence weight (higher = more influence on generation)" },
    "ref_id": { "type": "string", "optional": true, "description": "Reference ID (required for 'remove')" }
  }
}
```

### Response (action: "add")

```json
{
  "ref_id": "ref_001",
  "label": "architecture style",
  "image_analysis": "Gothic architecture with pointed arches, flying buttresses, stone material, warm interior lighting",
  "active_references": 3
}
```

### Response (action: "list")

```json
{
  "references": [
    { "ref_id": "ref_001", "label": "architecture style", "weight": 1.0, "summary": "Gothic stone architecture" },
    { "ref_id": "ref_002", "label": "color palette", "weight": 1.5, "summary": "Autumnal warm tones, golden hour" },
    { "ref_id": "ref_003", "label": "vegetation", "weight": 0.8, "summary": "Dense deciduous forest, moss-covered" }
  ]
}
```

### Implementation

1. **Reference storage:** Store references in the world's session state (alongside undo stack and entity history). Each reference includes:
   - The image data (resized to 1568px max edge)
   - VLM-generated text description (cached at add time)
   - Multimodal embedding vector (via existing `embed_multimodal()`)
   - Label and weight

2. **Generation context injection:** When any worldgen tool runs (`gen_plan_layout`, `gen_apply_blockout`, `gen_populate_region`, `gen_evaluate_scene`), active references are included in the LLM prompt context:
   - For VLM-capable models: include reference images directly (up to 4 images, selected by weight)
   - For text-only models: include the cached text descriptions
   - Weight influences selection order and prompt emphasis

3. **Evaluation augmentation:** `gen_evaluate_scene` (WG4) gains an implicit comparison mode — when references are active, evaluation includes "does the scene match the reference board?" as a quality dimension alongside existing layout/style/completeness checks.

4. **Persistence:** References are saved with `gen_save_world` in the world's metadata. When a world is loaded, its reference board is restored. This enables iterative sessions where the user refines a world across multiple sittings with consistent visual targets.

### Acceptance Criteria

- [ ] Add/remove/list/clear operations work correctly
- [ ] References persist across `gen_save_world` / `gen_load_world` cycles
- [ ] Worldgen tools include reference context in LLM prompts when references are active
- [ ] Text-only LLM fallback works (uses cached descriptions instead of images)
- [ ] Weight parameter influences reference selection priority
- [ ] Maximum of 8 active references enforced (context window management)

---

## Spec AI3.4: `gen_panorama_to_world` — 360° Image to Explorable World

**Goal:** Accept a 360° panorama image (equirectangular) and generate an explorable 3D world that places the user inside the panorama's environment. This matches World Labs' flagship capability (panorama → explorable 3D) but outputs compilable Bevy code instead of a proprietary format.

### MCP Tool Schema

```json
{
  "name": "gen_panorama_to_world",
  "description": "Generate an explorable 3D world from a 360° panorama image",
  "parameters": {
    "image": { "type": "string", "required": true, "description": "Path to equirectangular panorama image or base64 data" },
    "prompt": { "type": "string", "optional": true, "description": "Additional guidance for world generation" },
    "depth_estimation": { "type": "bool", "default": true, "description": "Estimate depth from panorama to inform 3D placement" },
    "generate_beyond": { "type": "bool", "default": false, "description": "Generate areas not visible in the panorama (behind camera, occluded regions)" }
  }
}
```

### Response

```json
{
  "world_name": "panorama_20260324_001",
  "entities_generated": 28,
  "regions": ["foreground_structures", "midground_terrain", "background_landscape", "sky_dome"],
  "spawn_point": [0, 1.7, 0],
  "notes": "Generated from equirectangular panorama. Foreground: stone path with wooden fence. Midground: rolling hills with scattered trees. Background: mountain range. Sky dome textured from panorama upper hemisphere."
}
```

### Implementation

1. **Panorama analysis:** Send the equirectangular image to the VLM with a specialized prompt requesting:
   - 360° spatial decomposition (what's at each cardinal direction)
   - Depth layer identification (near, mid, far for each direction)
   - Ground plane estimation
   - Sky/atmosphere description

2. **Depth estimation (optional):** When `depth_estimation: true`:
   - Use `gen_render_depth` (WG7) in reverse — instead of rendering depth from a scene, estimate depth from the panorama
   - If an external depth estimation model is available (via AI1 model server), use monocular depth estimation on the panorama
   - Fallback: use VLM spatial reasoning to assign relative depth values

3. **World generation pipeline:**
   - **Sky dome:** Apply the panorama's upper hemisphere as a sky texture using `gen_set_sky`
   - **Ground plane:** Extract ground color/texture from panorama's lower hemisphere, create terrain via `gen_add_terrain`
   - **Spatial layout:** Map VLM's directional analysis to `gen_plan_layout` regions arranged in a ring around the spawn point
   - **Entity population:** `gen_apply_blockout` → `gen_populate_region` for each region, with the reference board (AI3.3) auto-populated with crops from the panorama at each direction
   - **Spawn point:** Place the player at the panorama's camera origin (0, eye_height, 0) facing the same direction as the panorama's center

4. **Beyond-panorama generation:** When `generate_beyond: true`, generate content for areas not visible in the original panorama (behind occluded objects, extending landscape beyond the horizon). Uses the reference board context to maintain style consistency.

### Acceptance Criteria

- [ ] Accepts standard equirectangular panorama images (2:1 aspect ratio)
- [ ] Places player at the panorama's viewpoint with matching orientation
- [ ] Sky dome reflects the panorama's sky
- [ ] Ground plane approximates the panorama's ground surface
- [ ] Major spatial elements (structures, terrain features) are represented as 3D geometry
- [ ] Generated world is navigable (player can walk around within the reconstructed environment)
- [ ] Output is a standard world save (`gen_save_world` compatible)

---

## Architecture Notes

### MCP Multimodal Transport

The MCP 2026 roadmap includes native multimodal support. Until that ships, image input uses **file paths** as the transport mechanism:

1. **Current (file paths):** User provides a local file path. The tool reads the image, resizes it, and includes it in the LLM prompt as base64.
2. **Future (MCP multimodal):** MCP transport delivers image bytes directly. The tool receives image data without file system access. The tool interface (`"image"` parameter) is designed to accept both paths and base64 so the transition is seamless.

### VLM Provider Requirements

These tools require a vision-capable LLM. The provider routing system (`agent/providers.rs`) already supports multimodal models:

| Provider | Vision Model | Notes |
|----------|-------------|-------|
| Anthropic API | Claude (Sonnet/Opus) | Native vision, preferred |
| OpenAI | GPT-4o / GPT-4.1 | Native vision |
| Ollama | LLaVA, Llama 3.2 Vision | Local, no API cost |
| Claude CLI | Claude (via CLI) | Vision via file path |
| GLM | GLM-4V | Vision support |

If the configured model lacks vision capabilities, tools return an error with a message suggesting a vision-capable model.

### Relationship to Existing Tools

```
                    Reference Images (AI3)
                           │
                    ┌──────┴──────┐
                    ▼             ▼
            gen_image_to_layout  gen_reference_board
                    │             │ (persistent context)
                    ▼             ▼
              gen_plan_layout ◄──── style/mood influence
                    │
                    ▼
             gen_apply_blockout
                    │
                    ▼
            gen_populate_region ◄── reference board materials
                    │
                    ▼
             gen_evaluate_scene ◄── reference board comparison
                    │
                    ▼
              gen_match_style ◄──── post-hoc style adjustment
                    │
                    ▼
             gen_screenshot ────► visual verification
```

The new tools slot into the existing pipeline as **input enhancers** — they provide richer context to existing generation tools rather than replacing them.

### State Externalization Consideration

Per code idea #11 (MCP state externalization), reference board state should be serializable from the start. The reference images, cached descriptions, and embeddings are stored in the world's RON manifest alongside existing world state. This ensures compatibility with future stateless MCP transport.

---

## Estimated Effort

| Spec | Estimate | Priority | Depends On |
|------|----------|----------|------------|
| AI3.1 `gen_image_to_layout` | 3d | High | WG1 (blockout pipeline) |
| AI3.2 `gen_match_style` | 2d | Medium | Existing scene tools |
| AI3.3 `gen_reference_board` | 3d | High | AI3.1, world save/load |
| AI3.4 `gen_panorama_to_world` | 5d | Low | AI3.1, AI3.3, WG7 (depth) |
| **Total** | **13d** | | |
| **MVP (AI3.1 + AI3.3)** | **6d** | | |

### Implementation Order

1. **AI3.1** first — standalone tool, validates VLM image analysis → layout pipeline
2. **AI3.3** second — reference board provides persistent context, multiplies AI3.1's value
3. **AI3.2** third — style matching operates on existing scenes, lower risk
4. **AI3.4** last — most complex, depends on all prior specs

---

## Open Questions

1. **Image count limits:** How many reference images can be included in a single LLM prompt before context window / quality degrades? Initial limit of 4 images per prompt, 8 per reference board — to be tuned based on testing.

2. **Embedding similarity search:** Should `gen_reference_board` support "find a reference similar to this entity" using multimodal embeddings? This would let the evaluation loop automatically match entities against the most relevant reference image. Deferred to a follow-up spec.

3. **Video input:** The MCP 2026 roadmap mentions video support. A natural extension would be `gen_video_to_world` — analyzing a walkthrough video to reconstruct a 3D environment. Deferred until MCP video transport is available and AI3.1–AI3.4 are validated.
