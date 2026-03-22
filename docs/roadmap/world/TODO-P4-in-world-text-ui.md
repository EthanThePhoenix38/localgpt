# Priority 4: In-World Text & UI

These 5 specs enable storytelling, wayfinding, and player feedback. Text and UI elements bridge the gap between a visual world and a communicative experience.

**Dependencies:** Bevy 0.18 `Text2d`, `bevy_mod_billboard` (camera-facing text), Priority 2 (ScoreBoard for HUD integration)

---

## Spec 4.1: `gen_add_sign` — World-Space Text Signs

**Goal:** Place readable text in the 3D world, either billboard-facing (always readable) or fixed orientation (part of the environment).

### MCP Tool Schema

```json
{
  "name": "gen_add_sign",
  "description": "Place a text sign in the 3D world",
  "parameters": {
    "position": { "type": "vec3", "required": true },
    "text": { "type": "string", "required": true },
    "font_size": { "type": "f32", "default": 24.0 },
    "color": { "type": "string", "default": "#ffffff" },
    "background_color": { "type": "string", "optional": true },
    "billboard": { "type": "bool", "default": true, "description": "Always face camera" },
    "max_width": { "type": "f32", "optional": true, "description": "Word wrap width in world units" },
    "rotation": { "type": "vec3", "default": [0, 0, 0], "description": "Only used when billboard=false" }
  }
}
```

### Implementation

1. **Billboard text:** When `billboard: true`, use Bevy's `Text2d` with a system that rotates the entity to face the camera each frame. The text entity is a child of a transform-only parent at `position`.

2. **Fixed text:** When `billboard: false`, use `Text2d` with the specified `rotation`. The text is fixed in world space — readable only from certain angles, like a real sign.

3. **Background panel:** If `background_color` is set, spawn a quad mesh behind the text with:
   - Width/height matching the text bounds + 0.2m padding
   - `StandardMaterial` with background color, slight transparency (alpha 0.85)
   - Z-offset -0.01 behind the text to prevent z-fighting

4. **Word wrapping:** If `max_width` is set, configure `Text2d` with `TextBounds { width: max_width, .. }`. Text wraps to fit within the specified width.

5. **Readability scaling:** Text size scales with distance for readability:
   - Below 5m: full size
   - 5m–20m: slight scale-up to remain legible
   - Beyond 20m: fade out (alpha → 0) to avoid visual clutter

### Acceptance Criteria

- [ ] Sign text renders at specified position in 3D world
- [ ] Billboard mode: text always faces camera
- [ ] Fixed mode: text at specified rotation
- [ ] Background panel appears with correct color when specified
- [ ] Word wrapping works with max_width parameter
- [ ] Text color matches specified parameter
- [ ] Font size controls text scale in world space
- [ ] Text fades at long distances

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/ui/mod.rs` — module root
- `localgpt/crates/localgpt-gen/src/ui/sign.rs` — sign entity, billboard system, background panel
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_sign.rs` — MCP tool handler

---

## Spec 4.2: `gen_add_hud` — Screen-Space UI Overlay

**Goal:** Display persistent screen-space UI elements (score, health, timer, custom text) that update in response to game state.

### MCP Tool Schema

```json
{
  "name": "gen_add_hud",
  "description": "Add a screen-space UI element",
  "parameters": {
    "element_type": { "type": "enum", "values": ["score", "health", "text", "timer"], "required": true },
    "position": { "type": "enum", "values": ["top-left", "top-right", "bottom-left", "bottom-right", "center-top", "center-bottom"], "default": "top-left" },
    "label": { "type": "string", "optional": true },
    "initial_value": { "type": "string", "default": "0" },
    "font_size": { "type": "f32", "default": 18.0 },
    "color": { "type": "string", "default": "#ffffff" },
    "id": { "type": "string", "optional": true, "description": "ID for programmatic updates" }
  }
}
```

### Implementation

1. **UI node hierarchy:** Use Bevy UI (`Node`, `Text`) positioned absolutely within a root UI node. Each HUD element gets a unique `HudElement { id, element_type }` component.

2. **Position mapping:** Map position enum to Bevy `Style` flexbox alignment:
   - `top-left`: `position_type: Absolute, left: 20px, top: 20px`
   - `top-right`: `position_type: Absolute, right: 20px, top: 20px`
   - etc.

3. **Element types:**
   - `score`: Displays `"{label}: {value}"`. Listens for `ScoreChanged` events (from collectible system) and auto-updates. Default label: "Score".
   - `health`: Displays a horizontal bar (colored rectangle) that shrinks as value decreases. Red when below 25%.
   - `text`: Static or dynamically-updated text display.
   - `timer`: Counts up from 0 or counts down from `initial_value` (parsed as seconds). Formats as `MM:SS`. Emits `TimerExpired` event when countdown reaches 0.

4. **Update API:** HUD elements can be updated via `gen_update_hud` (future tool) or reactively by listening to game events (ScoreChanged, health damage events).

5. **Styling:** Semi-transparent dark background behind text for readability (`rgba(0, 0, 0, 0.5)` padding box). Text uses the specified `color` and `font_size`.

### Acceptance Criteria

- [ ] HUD elements render at correct screen positions
- [ ] Score element auto-updates from ScoreBoard changes
- [ ] Health element shows a colored bar that reflects value
- [ ] Timer element counts up or down with MM:SS format
- [ ] Text element displays static text
- [ ] Label prefix displays when specified
- [ ] Semi-transparent background ensures readability
- [ ] Multiple HUD elements can coexist at different positions

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/ui/hud.rs` — HUD element spawning, positioning, updates
- `localgpt/crates/localgpt-gen/src/ui/hud_timer.rs` — timer countdown/countup logic
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_hud.rs` — MCP tool handler

---

## Spec 4.3: `gen_add_label` — Entity Nameplates

**Goal:** Attach a floating name label above any entity, always facing the camera. Used for NPCs, objects, locations.

### MCP Tool Schema

```json
{
  "name": "gen_add_label",
  "description": "Attach a floating name label above an entity",
  "parameters": {
    "entity_id": { "type": "string", "required": true },
    "text": { "type": "string", "required": true },
    "color": { "type": "string", "default": "#ffffff" },
    "background_color": { "type": "string", "default": "#00000088" },
    "offset_y": { "type": "f32", "default": 0.5, "description": "Height above entity's top" },
    "font_size": { "type": "f32", "default": 16.0 },
    "visible_distance": { "type": "f32", "default": 15.0 }
  }
}
```

### Implementation

1. **Label entity:** Spawn as a child of the target entity. Position at `(0, entity_height + offset_y, 0)` where `entity_height` is computed from the entity's mesh or collider bounds.

2. **Billboard behavior:** System rotates the label to face the camera every frame (same as sign billboard logic — share the system).

3. **Distance fade:** Compute distance from camera to label. Alpha transitions:
   - Distance < `visible_distance * 0.8`: full opacity
   - Distance `0.8–1.0 × visible_distance`: linear fade to 0
   - Distance > `visible_distance`: hidden

4. **Background:** Small rounded-rectangle quad behind text with `background_color` (supports alpha in hex: `#RRGGBBAA`).

5. **Follow parent:** As a child entity, the label automatically moves with its parent. No additional tracking system needed.

### Acceptance Criteria

- [ ] Label floats above the target entity
- [ ] Label always faces the camera (billboard)
- [ ] Label follows entity as it moves
- [ ] Label fades out beyond visible_distance
- [ ] Background color renders behind text
- [ ] Height offset positions label above entity top
- [ ] Multiple labels on different entities work independently

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/ui/label.rs` — label spawning, height calculation, distance fade
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_label.rs` — MCP tool handler

---

## Spec 4.4: `gen_add_tooltip` — Interaction Prompts

**Goal:** Show contextual text when the player looks at or approaches an entity. Used for "Press E to open", item descriptions, and hints.

### MCP Tool Schema

```json
{
  "name": "gen_add_tooltip",
  "description": "Show contextual text when player is near or looking at an entity",
  "parameters": {
    "entity_id": { "type": "string", "required": true },
    "text": { "type": "string", "required": true },
    "trigger": { "type": "enum", "values": ["proximity", "look_at"], "default": "proximity" },
    "range": { "type": "f32", "default": 3.0 },
    "style": { "type": "enum", "values": ["floating", "screen_center", "screen_bottom"], "default": "floating" },
    "color": { "type": "string", "default": "#ffffff" },
    "duration": { "type": "f32", "optional": true, "description": "Auto-dismiss after seconds, null = persistent while in range" }
  }
}
```

### Implementation

1. **Trigger detection:**
   - `proximity`: Check distance between player and entity each frame. Show tooltip when within `range`.
   - `look_at`: Raycast from camera center. If ray hits entity's collider within `range`, show tooltip.

2. **Display styles:**
   - `floating`: Billboard text above the entity (reuse sign/label rendering). Appears with fade-in (0.2s).
   - `screen_center`: UI text centered on screen (for important prompts). Dark semi-transparent backdrop.
   - `screen_bottom`: UI text at bottom-center of screen (subtitle-style).

3. **Tooltip state:** `TooltipState` component tracks `is_visible`, `fade_alpha`, `display_timer`. Only one `screen_center` or `screen_bottom` tooltip visible at a time (latest wins). Multiple `floating` tooltips can coexist.

4. **Auto-dismiss:** If `duration` is set, tooltip disappears after `duration` seconds even if player is still in range. Cooldown prevents re-showing for 2× duration.

5. **Input hint integration:** Text can include `{E}` placeholder which renders as a key icon/badge. E.g., `"Press {E} to interact"` shows "Press [E] to interact" with E in a rounded box.

### Acceptance Criteria

- [ ] Proximity tooltip appears when player enters range
- [ ] Look-at tooltip appears when player aims at entity
- [ ] Floating style shows text above entity
- [ ] Screen-center and screen-bottom styles show as UI overlay
- [ ] Tooltip fades in smoothly
- [ ] Auto-dismiss hides tooltip after duration
- [ ] {E} placeholder renders as key badge
- [ ] Tooltip disappears when player leaves range

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/ui/tooltip.rs` — trigger detection, display, fade, dismiss
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_tooltip.rs` — MCP tool handler

---

## Spec 4.5: `gen_add_notification` — Transient Messages

**Goal:** Display temporary notification messages (achievements, pickups, events) that appear and auto-dismiss with animation.

### MCP Tool Schema

```json
{
  "name": "gen_add_notification",
  "description": "Show a temporary notification message on screen",
  "parameters": {
    "text": { "type": "string", "required": true },
    "style": { "type": "enum", "values": ["toast", "banner", "achievement"], "default": "toast" },
    "position": { "type": "enum", "values": ["top", "center", "bottom"], "default": "top" },
    "duration": { "type": "f32", "default": 3.0 },
    "color": { "type": "string", "default": "#ffffff" },
    "icon": { "type": "string", "optional": true, "description": "Icon name: star, coin, key, heart, warning" },
    "sound": { "type": "string", "optional": true }
  }
}
```

### Implementation

1. **Notification styles:**
   - `toast`: Small rounded-rectangle card that slides in from the right, pauses, slides out. Max width 300px.
   - `banner`: Full-width bar that slides down from top (or up from bottom). Used for major events.
   - `achievement`: Centered card with icon on left, title text in bold, with gold border and sparkle effect. Slides in + scale animation.

2. **Animation sequence:**
   - Enter: slide in over 0.3s with ease-out
   - Hold: stay visible for `duration` seconds
   - Exit: fade out + slide away over 0.3s with ease-in

3. **Stacking:** Multiple simultaneous toasts stack vertically (newest on top, older ones shift down). Maximum 4 visible toasts; older ones are dismissed early.

4. **Icons:** Simple built-in icon set rendered as unicode or small colored shapes:
   - `star`: gold star shape
   - `coin`: gold circle
   - `key`: key shape
   - `heart`: red heart
   - `warning`: yellow triangle with exclamation

5. **Sound:** If `sound` is specified, play the audio asset when notification appears.

6. **Programmatic use:** Other systems (collectible pickup, achievement, timer expiry) can spawn notifications directly via `NotificationEvent { text, style, ... }`.

### Acceptance Criteria

- [ ] Toast notification slides in from right side
- [ ] Banner notification slides from top/bottom edge
- [ ] Achievement notification shows centered with icon and gold border
- [ ] Notification auto-dismisses after duration
- [ ] Enter/exit animations are smooth
- [ ] Multiple toasts stack vertically
- [ ] Icons render next to notification text
- [ ] Sound plays when notification appears
- [ ] NotificationEvent can be emitted by other systems

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/ui/notification.rs` — styles, animation, stacking, icons
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_notification.rs` — MCP tool handler
