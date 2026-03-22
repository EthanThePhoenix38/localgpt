# Priority 3: Terrain & Landscape

These 5 specs enable landscape-scale worlds. Terrain, water, paths, vegetation, and sky/weather transform LocalGPT Gen from room-scale scenes to open-world environments.

**Dependencies:** Priority 1 (avatar + collision for walking on terrain), Avian v0.5 (terrain colliders), bevy_generative or custom noise (procedural generation)

---

## Spec 3.1: `gen_add_terrain` — Procedural Terrain

**Goal:** Generate a terrain surface from noise parameters, with automatic collision and material.

### MCP Tool Schema

```json
{
  "name": "gen_add_terrain",
  "description": "Create a terrain surface from procedural noise",
  "parameters": {
    "size": { "type": "vec2", "default": [100, 100] },
    "resolution": { "type": "i32", "default": 128, "description": "Vertices per side" },
    "height_scale": { "type": "f32", "default": 20.0 },
    "noise_type": { "type": "enum", "values": ["perlin", "simplex", "flat"], "default": "perlin" },
    "noise_octaves": { "type": "i32", "default": 4 },
    "noise_frequency": { "type": "f32", "default": 0.02 },
    "seed": { "type": "i64", "optional": true, "description": "Random seed, auto-generated if omitted" },
    "material": { "type": "enum", "values": ["grass", "sand", "snow", "rock", "custom"], "default": "grass" },
    "position": { "type": "vec3", "default": [0, 0, 0] }
  }
}
```

### Implementation

1. **Mesh generation:** Create a grid mesh of `resolution × resolution` vertices spanning `size`. For each vertex, sample noise at `(x * noise_frequency, z * noise_frequency)` with `noise_octaves` octaves of fractal Brownian motion (fBm). Multiply noise value by `height_scale` to get Y position.

2. **Noise library:** Use the `noise` crate (pure Rust). `perlin` → `Perlin::new(seed)`, `simplex` → `SuperSimplex::new(seed)`, `flat` → constant 0.0 height.

3. **Material presets:**
   - `grass`: green-brown gradient based on slope (steep = rock, flat = grass), using Bevy's `StandardMaterial`
   - `sand`: warm tan color, slight roughness variation
   - `snow`: white with blue-tinted shadows, high roughness
   - `rock`: gray with normal map variation based on noise
   - `custom`: plain white, user can modify via `gen_set_material`

4. **Collision:** Generate an Avian `Collider::trimesh` from the terrain mesh vertices and indices. This enables the player to walk on the terrain.

5. **Normals:** Compute smooth vertex normals from the height map gradient for correct lighting. Tangents computed for potential normal mapping.

6. **Chunking (future):** For MVP, generate as a single mesh. Document that terrains larger than 500×500 at resolution 256+ should be chunked in future versions.

### Acceptance Criteria

- [ ] Terrain mesh generates with correct dimensions and height variation
- [ ] Perlin and simplex noise produce distinct terrain shapes
- [ ] Flat mode creates a level plane
- [ ] Seed parameter produces deterministic terrain
- [ ] Material presets apply appropriate colors and textures
- [ ] Player can walk on terrain surface (trimesh collider)
- [ ] Terrain normals produce correct lighting
- [ ] Resolution parameter controls mesh detail level

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/terrain/mod.rs` — module root
- `localgpt/crates/localgpt-gen/src/terrain/heightmap.rs` — noise sampling, mesh generation
- `localgpt/crates/localgpt-gen/src/terrain/materials.rs` — preset materials
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_terrain.rs` — MCP tool handler
- `Cargo.toml` — add `noise` crate dependency

---

## Spec 3.2: `gen_add_water` — Water Planes

**Goal:** Add a water surface at a specified height with animated waves, transparency, and reflections.

### MCP Tool Schema

```json
{
  "name": "gen_add_water",
  "description": "Add a water plane with waves and transparency",
  "parameters": {
    "height": { "type": "f32", "default": 0.0 },
    "size": { "type": "vec2", "default": [100, 100] },
    "color": { "type": "string", "default": "#2389da" },
    "opacity": { "type": "f32", "default": 0.7 },
    "wave_speed": { "type": "f32", "default": 1.0 },
    "wave_height": { "type": "f32", "default": 0.3 },
    "position": { "type": "vec3", "optional": true, "description": "XZ center, Y from height param" }
  }
}
```

### Implementation

1. **Water mesh:** Subdivided plane (32×32 segments) at the specified `height`. Using `AlphaMode::Blend` for transparency with the specified `opacity`.

2. **Wave animation:** Custom vertex shader (or system-driven vertex displacement) using two overlapping sine waves at different frequencies and directions:
   - Wave A: direction (1, 0), frequency 0.5, amplitude `wave_height * 0.6`
   - Wave B: direction (0.7, 0.7), frequency 0.8, amplitude `wave_height * 0.4`
   - Both modulated by `wave_speed` multiplier on time

3. **Material:** `StandardMaterial` with:
   - Base color from `color` parameter with alpha from `opacity`
   - Metallic: 0.0, Reflectance: 0.8 (water-like specular)
   - Roughness: 0.1 (smooth, reflective surface)
   - `AlphaMode::Blend`

4. **Interaction with terrain:** Water is purely visual — it does not block movement. If the player goes below water height, apply a blue tint overlay to the camera (underwater effect). This is a stretch goal; MVP just needs the visual plane.

5. **No collision:** Water plane has no collider. Player walks through it (swimming is a future feature).

### Acceptance Criteria

- [ ] Water plane renders at specified height with correct color
- [ ] Transparency works — terrain/objects below water are visible
- [ ] Wave animation moves the water surface
- [ ] wave_speed controls animation rate
- [ ] wave_height controls displacement amplitude
- [ ] Water reflects light with specular highlights
- [ ] Multiple water planes can coexist at different heights

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/terrain/water.rs` — mesh, material, wave animation
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_water.rs` — MCP tool handler

---

## Spec 3.3: `gen_add_path` — Walkable Paths

**Goal:** Create a visible path (road, trail, walkway) between waypoints, conforming to terrain surface.

### MCP Tool Schema

```json
{
  "name": "gen_add_path",
  "description": "Create a walkable path between waypoints",
  "parameters": {
    "points": { "type": "vec3[]", "required": true, "min_length": 2 },
    "width": { "type": "f32", "default": 2.0 },
    "material": { "type": "enum", "values": ["stone", "dirt", "wood", "cobblestone", "custom"], "default": "stone" },
    "curved": { "type": "bool", "default": true },
    "raised": { "type": "f32", "default": 0.02, "description": "Height above terrain to prevent z-fighting" },
    "border": { "type": "bool", "default": false, "description": "Add stone border edges" }
  }
}
```

### Implementation

1. **Curve generation:**
   - If `curved: true`: Catmull-Rom spline through the waypoints, sampled at 1m intervals
   - If `curved: false`: straight line segments between points

2. **Mesh generation:** For each segment between sample points:
   - Compute tangent direction (forward)
   - Compute perpendicular (right) on the XZ plane
   - Place two vertices at `point ± (right * width/2)`, raised by `raised` above the Y coordinate
   - Connect consecutive vertex pairs into a triangle strip
   - UV coordinates: U maps across width (0→1), V maps along length (accumulated distance)

3. **Material presets:**
   - `stone`: gray with subtle noise variation
   - `dirt`: brown, low roughness
   - `wood`: planks texture (repeating UV along length)
   - `cobblestone`: grid pattern
   - `custom`: plain, user modifiable

4. **Border:** If `border: true`, generate two narrow raised strips (width * 0.1) along each edge with a stone material, raised an additional 0.05m above the path surface.

5. **Terrain conformity:** If a terrain entity exists, raycast downward from each path point to find terrain height. Adjust path vertex Y to terrain height + `raised`. This drapes the path onto the terrain.

### Acceptance Criteria

- [ ] Path renders between all specified waypoints
- [ ] Curved mode smoothly interpolates between points
- [ ] Straight mode creates direct line segments
- [ ] Path width matches specified parameter
- [ ] Material presets apply correct appearance
- [ ] Path conforms to terrain height when terrain exists
- [ ] Border option adds raised edges
- [ ] Path is slightly raised to prevent z-fighting

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/terrain/path.rs` — spline, mesh generation, terrain conformity
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_path.rs` — MCP tool handler

---

## Spec 3.4: `gen_add_foliage` — Vegetation Scattering

**Goal:** Scatter vegetation (trees, bushes, grass clumps, flowers) across terrain using density and placement rules.

### MCP Tool Schema

```json
{
  "name": "gen_add_foliage",
  "description": "Scatter vegetation across terrain",
  "parameters": {
    "foliage_type": { "type": "enum", "values": ["tree", "bush", "grass", "flower", "rock"], "required": true },
    "area": { "type": "object", "properties": {
      "center": { "type": "vec3", "default": [0, 0, 0] },
      "radius": { "type": "f32", "default": 30.0 }
    }},
    "density": { "type": "f32", "default": 0.5, "description": "0.0-1.0, items per square meter scaled" },
    "scale_range": { "type": "vec2", "default": [0.8, 1.2], "description": "Min/max random scale multiplier" },
    "seed": { "type": "i64", "optional": true },
    "avoid_paths": { "type": "bool", "default": true },
    "avoid_water": { "type": "bool", "default": true },
    "max_slope": { "type": "f32", "default": 30.0, "description": "Max terrain slope in degrees for placement" }
  }
}
```

### Implementation

1. **Placement algorithm:** Poisson disk sampling within the circular area for natural-looking distribution. Density parameter controls minimum distance between samples (density 1.0 = 1 item per 2m², density 0.1 = 1 item per 20m²).

2. **Terrain sampling:** For each placement point, raycast downward to find terrain height. Skip points where:
   - No terrain exists below
   - Terrain slope exceeds `max_slope`
   - Point is below water height (if `avoid_water`)
   - Point is within 1.5× width of any path (if `avoid_paths`)

3. **Foliage meshes** (procedural primitives for MVP):
   - `tree`: brown cylinder trunk (0.2m radius, 2m tall) + green cone canopy (1.5m radius, 3m tall)
   - `bush`: green sphere (0.5–0.8m radius), slightly flattened
   - `grass`: cluster of 5–8 thin green quads oriented randomly
   - `flower`: thin green stem + colored sphere on top (random color from palette)
   - `rock`: deformed sphere (noise-displaced icosphere) with gray material

4. **Random variation:** Each instance gets random Y-axis rotation, random scale within `scale_range`, and slight color variation (±10% hue/saturation).

5. **Performance:** Use Bevy's instanced rendering. All instances of the same foliage type share one mesh and material, rendered as instances. Group into a single parent entity for easy management.

### Acceptance Criteria

- [ ] Foliage scatters within specified area at correct density
- [ ] Placement follows Poisson disk distribution (no clumping)
- [ ] Foliage sits on terrain surface (raycast height)
- [ ] Steep slopes are excluded
- [ ] Paths and water areas are avoided when flags set
- [ ] Random rotation and scale variation applied per instance
- [ ] Each foliage type has a distinct visual appearance
- [ ] Performance is acceptable with 500+ instances

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/terrain/foliage.rs` — placement, meshes, instancing
- `localgpt/crates/localgpt-gen/src/terrain/poisson.rs` — Poisson disk sampling utility
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_add_foliage.rs` — MCP tool handler

---

## Spec 3.5: `gen_set_sky` — Sky and Atmosphere

**Goal:** Configure the sky, sun direction, ambient lighting, and optional fog to set the mood and time of day for the world.

### MCP Tool Schema

```json
{
  "name": "gen_set_sky",
  "description": "Set sky appearance, sun direction, and atmosphere",
  "parameters": {
    "preset": { "type": "enum", "values": ["day", "sunset", "night", "overcast", "custom"], "default": "day" },
    "sun_altitude": { "type": "f32", "optional": true, "description": "Sun angle above horizon in degrees (0-90)" },
    "sun_azimuth": { "type": "f32", "optional": true, "description": "Sun compass direction in degrees (0=north)" },
    "sun_intensity": { "type": "f32", "optional": true },
    "ambient_color": { "type": "string", "optional": true },
    "ambient_intensity": { "type": "f32", "optional": true },
    "fog_enabled": { "type": "bool", "default": false },
    "fog_color": { "type": "string", "default": "#c8d0d8" },
    "fog_start": { "type": "f32", "default": 50.0 },
    "fog_end": { "type": "f32", "default": 200.0 }
  }
}
```

### Implementation

1. **Presets** (set sun/ambient/fog defaults):
   - `day`: sun_altitude=60, sun_intensity=1.0, ambient=#87CEEB at 0.3, no fog
   - `sunset`: sun_altitude=5, sun_azimuth=270, sun_intensity=0.8, warm amber ambient, light fog
   - `night`: sun_altitude=-10 (below horizon, moonlight), sun_intensity=0.1, dark blue ambient at 0.05
   - `overcast`: sun_altitude=45, sun_intensity=0.3, gray ambient at 0.5, fog enabled
   - `custom`: no defaults, all explicit parameters

2. **Directional light:** Update the scene's `DirectionalLight` entity with direction computed from `sun_altitude` and `sun_azimuth`. Enable shadows with cascade shadow maps.

3. **Ambient light:** Set `AmbientLight { color, brightness }` resource from `ambient_color` and `ambient_intensity`.

4. **Sky visual:** Use Bevy's `Skybox` with a procedural gradient texture:
   - Sample sky color from horizon to zenith based on sun altitude
   - Day: blue gradient. Sunset: orange-purple gradient. Night: dark blue with star dots. Overcast: uniform gray.
   - Generate as a cubemap texture at setup time.

5. **Fog:** Configure Bevy's `DistanceFog` with `color`, `falloff: Linear { start: fog_start, end: fog_end }`.

6. **Singleton:** Only one sky configuration exists. Calling `gen_set_sky` again replaces the previous settings.

### Acceptance Criteria

- [ ] Day preset shows bright blue sky with strong sunlight
- [ ] Sunset preset shows warm lighting with low sun angle
- [ ] Night preset shows dark environment with dim moonlight
- [ ] Overcast preset shows gray sky with soft diffuse lighting
- [ ] Custom parameters override preset defaults
- [ ] Sun direction matches altitude/azimuth parameters
- [ ] Fog renders with correct start/end distances
- [ ] Shadows from directional light work correctly
- [ ] Calling gen_set_sky again replaces previous settings

### Files to Create/Modify

- `localgpt/crates/localgpt-gen/src/terrain/sky.rs` — presets, directional light, ambient, fog, skybox
- `localgpt/crates/localgpt-gen/src/mcp/tools/gen_set_sky.rs` — MCP tool handler
