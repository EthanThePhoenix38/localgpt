# WorldGen 4: Screenshot-Based Evaluation Loop

**Uses existing infrastructure for significant quality gains.** Extends the existing `gen_screenshot` tool with context-aware highlighting and integrates it into a generate → screenshot → evaluate → adjust loop. This is the most directly transferable pattern from the WorldGen paper.

**Source:** WorldGen paper Stage IV (Per-Object Enhancement) — renders top-down view with target object highlighted in red, sends to VLM for context-aware enhancement.

**Dependencies:** Existing `gen_screenshot` tool, LLM vision capabilities, WG3 (tier tagging for selective operations)

**Priority within WorldGen series:** 4 of 7

---

## Spec WG4.1: `gen_screenshot` Highlight Mode

**Goal:** Extend the existing `gen_screenshot` tool to support highlighting a specific entity in a distinct color while showing the rest of the scene for context. This gives the LLM visual grounding for style consistency decisions.

### MCP Tool Extension

Add parameters to the existing `gen_screenshot` tool:

```json
{
  "name": "gen_screenshot",
  "description": "Capture a screenshot of the current scene",
  "parameters": {
    "... existing parameters ...": {},
    "highlight_entity": { "type": "string", "optional": true, "description": "Entity ID to highlight with distinct color" },
    "highlight_color": { "type": "string", "default": "#ff0000", "description": "Highlight color (default red)" },
    "camera_angle": { "type": "enum", "values": ["current", "top_down", "isometric", "front", "entity_focus"], "default": "current" },
    "include_annotations": { "type": "bool", "default": false, "description": "Overlay entity names and bounding boxes" }
  }
}
```

### Implementation

1. **Entity highlighting:** When `highlight_entity` is specified:
   - Store the entity's original `StandardMaterial.emissive` value
   - Override with `highlight_color` at high emissive intensity (3.0)
   - Capture the screenshot
   - Restore the original emissive value
   - This makes the target entity visually pop against the scene context

2. **Camera angles:**
   - `current`: use the current camera position and orientation
   - `top_down`: position camera directly above the scene center, looking down (useful for layout evaluation)
   - `isometric`: position camera at ~45 degree angle from above, matching WorldGen's reference image perspective
   - `front`: position camera at ground level facing the scene from the north
   - `entity_focus`: if `highlight_entity` is set, position camera to frame the highlighted entity with surrounding context visible (2x entity bounding sphere as framing distance)

3. **Annotations overlay:** When `include_annotations: true`:
   - Render entity names as 2D text labels at each entity's screen-space position
   - Draw bounding boxes around entities as colored wireframe rectangles
   - Color-code by tier: gold for hero, green for medium, gray for decorative
   - This helps the LLM identify what it's looking at in the screenshot

4. **Performance:** Screenshots should capture in a single frame. The highlight/restore cycle must complete synchronously within one render pass. Use Bevy's `RenderTarget::Image` to capture without affecting the display.

### Acceptance Criteria

- [ ] Highlighted entity renders with distinct emissive color in screenshot
- [ ] Original material is fully restored after screenshot capture
- [ ] top_down camera angle captures full scene from above
- [ ] isometric camera angle matches ~45 degree perspective
- [ ] entity_focus frames the highlighted entity with context
- [ ] Annotations overlay shows entity names and tier-colored bounding boxes
- [ ] Screenshot capture is synchronous (no frame delay artifacts)

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_screenshot.rs` — extend with highlight, camera angle, annotations

---

## Spec WG4.2: `gen_evaluate_scene` — Automated Quality Check

**Goal:** Capture a screenshot of the current scene (or a specific entity in context) and return a structured quality assessment. This tool enables the LLM to self-evaluate its generation work.

### MCP Tool Schema

```json
{
  "name": "gen_evaluate_scene",
  "description": "Evaluate scene quality by capturing and analyzing a screenshot",
  "parameters": {
    "focus_entity": { "type": "string", "optional": true, "description": "Entity to evaluate in scene context" },
    "check_style_consistency": { "type": "bool", "default": true },
    "check_spatial_layout": { "type": "bool", "default": true },
    "check_density": { "type": "bool", "default": true },
    "reference_prompt": { "type": "string", "optional": true, "description": "Original world description for comparison" }
  }
}
```

### Response Schema

```json
{
  "screenshot_path": "/tmp/gen_eval_001.png",
  "scores": {
    "style_consistency": 0.8,
    "spatial_coherence": 0.7,
    "density_balance": 0.9,
    "overall": 0.8
  },
  "issues": [
    {
      "type": "style_mismatch",
      "entity": "modern_lamp_post",
      "description": "Modern lamp post clashes with medieval village style",
      "suggestion": "Replace with torch or lantern on wooden pole"
    },
    {
      "type": "density_gap",
      "area": [10, 0, 15],
      "description": "Large empty area between buildings with no ground cover",
      "suggestion": "Add grass, cobblestone path, or market stalls"
    }
  ],
  "passes": true
}
```

### Implementation

1. **Screenshot capture:** Internally call `gen_screenshot` with:
   - `camera_angle: "isometric"` for overall scene evaluation
   - `camera_angle: "entity_focus"` + `highlight_entity` for per-entity evaluation
   - `include_annotations: true` to help identify entities

2. **Quality checks** (each is an analysis step the LLM performs on the screenshot):

   **Style consistency:** Does every entity match the world's intended style/biome? Flag entities whose visual style (material, shape, color palette) diverges from the scene's established aesthetic. Compare against `palette.primary_biome` from the BlockoutSpec.

   **Spatial coherence:** Are entities placed logically? Buildings on flat ground, trees not inside buildings, paths connecting destinations, no floating objects. Check for obvious spatial errors visible in the screenshot.

   **Density balance:** Are there large empty gaps that look unfinished? Are areas overly crowded? Compare actual density against the BlockoutSpec region density targets.

3. **Scoring:** Each check produces a 0.0-1.0 score. Overall score is weighted average (style: 0.4, spatial: 0.35, density: 0.25). Scene "passes" if overall >= 0.7.

4. **Issue list:** Each issue includes type, affected entity/area, description, and an actionable suggestion. Suggestions should be implementable via existing MCP tools.

5. **Reference comparison:** If `reference_prompt` is provided, evaluate how well the scene matches the original description. Flag missing elements ("prompt mentions a fountain but none exists") and unexpected additions.

### Acceptance Criteria

- [ ] Screenshot captured at appropriate angle for evaluation type
- [ ] Style consistency check identifies mismatched entities
- [ ] Spatial coherence check finds floating or overlapping objects
- [ ] Density check identifies gaps and overcrowding
- [ ] Overall score reflects scene quality
- [ ] Issues include actionable suggestions
- [ ] Reference prompt comparison flags missing/unexpected elements
- [ ] passes field reflects whether scene meets quality threshold

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/evaluate.rs` — evaluation orchestration, scoring
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_evaluate_scene.rs` — MCP tool handler

---

## Spec WG4.3: Iterative Refinement Loop

**Goal:** Define a generate → evaluate → adjust workflow that the LLM follows to iteratively improve scene quality. This is not a separate tool but a documented workflow pattern and optional automation mode.

### Implementation

1. **Manual loop (LLM-driven):** After population (WG1.3), the LLM:
   1. Calls `gen_evaluate_scene` to check quality
   2. Reviews the issues list
   3. For each issue, makes adjustments via existing tools (`gen_modify_entity`, `gen_spawn_primitive`, etc.)
   4. Calls `gen_evaluate_scene` again to verify fixes
   5. Repeats until `passes: true` or max iterations reached

2. **Auto-refine mode:** `gen_auto_refine` tool that runs the loop automatically:
   ```json
   {
     "name": "gen_auto_refine",
     "description": "Automatically iterate on scene quality until it passes evaluation",
     "parameters": {
       "max_iterations": { "type": "i32", "default": 3 },
       "target_score": { "type": "f32", "default": 0.7 },
       "fix_style": { "type": "bool", "default": true },
       "fix_density": { "type": "bool", "default": true },
       "fix_spatial": { "type": "bool", "default": true }
     }
   }
   ```

3. **Refinement actions (auto mode):**
   - Style mismatch → attempt to replace entity with biome-appropriate alternative
   - Density gap → scatter decorative elements in the empty area
   - Floating object → snap to terrain height
   - Overlapping objects → offset one by its bounding box width

4. **Iteration guard:** Maximum iterations prevents infinite loops. If score doesn't improve between iterations, stop early. Log each iteration's score for debugging.

5. **Quality gate integration:** Before saving a world (`gen_save_world`), optionally run evaluation as a quality gate. If the scene doesn't pass, warn the user but allow saving.

### Acceptance Criteria

- [ ] Manual evaluation loop workflow is documented
- [ ] gen_auto_refine runs evaluation and applies fixes iteratively
- [ ] Max iterations prevents infinite refinement loops
- [ ] Score improvement is tracked across iterations
- [ ] Early stop when score plateaus
- [ ] Quality gate warns on save if scene doesn't pass evaluation

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/worldgen/refine.rs` — auto-refine orchestration, fix strategies
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_auto_refine.rs` — MCP tool handler

---

## Summary

| Spec | Tool | What | Effort |
|------|------|------|--------|
| WG4.1 | `gen_screenshot` extension | Highlight mode, camera angles, annotations | Low-Medium |
| WG4.2 | `gen_evaluate_scene` | Automated quality scoring with issue detection | Medium |
| WG4.3 | `gen_auto_refine` | Iterative generate → evaluate → fix loop | Medium |

**Net effect:** The LLM can see what it built and self-correct. Style mismatches, spatial errors, and density imbalances are caught automatically instead of requiring manual user feedback. This closes the loop between generation and quality — the most impactful pattern from WorldGen's Stage IV.
