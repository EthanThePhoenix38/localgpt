# World Template Authoring Guide

**Status:** Draft
**Date:** 2026-03-22
**Purpose:** Step-by-step instructions for creating starter world templates

---

## 1. Template Format

Each template is a world skill directory:

```
templates/{name}/
├── SKILL.md              # Template metadata + description
├── world.ron             # Full scene manifest (entities, materials, audio, etc.)
├── screenshots/          # Preview images
│   ├── thumbnail.png     # 1200x630 gallery card
│   └── preview_*.png     # 3-5 additional angles
└── assets/meshes/        # Any imported glTF/GLB assets
```

Templates are stored in `{workspace}/templates/` and loaded via `gen_fork_world` (copies to `skills/` with new name) or `gen_load_world` (loads directly).

---

## 2. Template Creation Workflow

### Step 1: Plan the scene

Define before building:
- **Theme**: medieval, sci-fi, nature, fantasy, etc.
- **Key features**: 3-5 notable elements (buildings, terrain, water, NPCs)
- **Interaction points**: doors, collectibles, teleporters, triggers
- **Ambient audio**: which soundscapes (forest, cave, ocean, etc.)
- **Player spawn**: position, facing direction

### Step 2: Build via MCP tools

Start gen mode and build the world using tool calls:

```bash
cargo run -p localgpt-gen
```

Core building sequence:
1. `gen_add_terrain(size=[200,200], height_scale=15, noise_type="perlin", material="grass")`
2. `gen_add_water(height=2.0, size=[50,50], color="#2389da")`
3. `gen_spawn_primitive(shape="cuboid", ...)` — buildings, walls, structures
4. `gen_add_path(points=[[...]], width=3.0, texture="stone")`
5. `gen_add_foliage(radius=30, density=50, type="tree")`
6. `gen_set_environment(sky_color=..., fog_density=...)` — atmosphere
7. `gen_set_light(...)` — lighting
8. `gen_set_ambience(layers=[{sound:"forest", volume:0.6}])` — audio

### Step 3: Add interactions

```
gen_add_door(entity_id="door1", open_angle=90, auto_close_delay=3.0)
gen_add_collectible(entity_id="coin1", value=10, category="gold", pickup_effect="sparkle")
gen_add_trigger(entity_id="zone1", trigger_type="area", radius=5.0, action="show_text", text="Welcome!")
gen_add_teleporter(position=[0,0,5], destination=[50,0,50], effect="fade", label="Portal")
```

### Step 4: Test with player

```
gen_spawn_player(position=[0, 5, 0])
```

Walk through the entire world:
- [ ] All buildings have collision (can't walk through)
- [ ] Doors open/close properly
- [ ] Collectibles award score
- [ ] Teleporters move player to destination
- [ ] Audio plays and fades with distance
- [ ] Scale feels human (doorways ~2m, ceilings ~3m)
- [ ] At least 2 distinct areas connected by navigation

### Step 5: Capture screenshots

```
gen_screenshot(camera_angle="isometric", width=1200, height=630)
gen_screenshot(camera_angle="front")
gen_screenshot(camera_angle="top_down")
```

Save the best as `thumbnail.png` (1200x630 for social sharing / gallery cards).

### Step 6: Save as template

```
gen_save_world(name="template_zen_garden", description="A peaceful Japanese-inspired garden with water features, stone paths, and wind ambient audio.")
```

### Step 7: Verify round-trip

```
gen_clear_scene()
gen_load_world(path="template_zen_garden")
gen_spawn_player(position=[0, 5, 0])
```

Walk through again — everything should be identical to the original.

---

## 3. Template Catalog

### Priority 1 (ship first)

| # | Name | Theme | Key Features | SEO Keywords |
|---|------|-------|-------------|--------------|
| 1 | **Willowmere Village** | Medieval fantasy | Stone buildings, market square, torch lights, forest ambient | ai world generator, 3d village, fantasy world |
| 2 | **Zen Garden** | Peaceful nature | Water pond, stone paths, bamboo foliage, wind ambient | zen garden 3d, peaceful world, meditation space |
| 3 | **Space Station** | Sci-fi | Metal corridors, sliding doors, hum emitters, blue lighting | space station 3d, sci-fi world, space game |

### Priority 2

| # | Name | Theme | Key Features |
|---|------|-------|-------------|
| 4 | **Desert Oasis** | Adventure | Terrain hills, water pool, palm foliage, cave entrance |
| 5 | **Crystal Cave** | Fantasy underground | Glowing materials, echo ambient, collectible crystals |
| 6 | **Cyberpunk Alley** | Neon urban | Tall buildings, neon materials, rain ambient, hum emitters |
| 7 | **Treehouse Village** | Whimsical | Elevated platforms, rope paths, forest ambient, bird sounds |
| 8 | **Underwater Reef** | Ocean | Blue-green water, coral structures, ocean ambient, bubbles |

### Priority 3

| # | Name | Theme |
|---|------|-------|
| 9 | Haunted Mansion | Horror — flickering lights, creaking doors, cave ambient |
| 10 | Floating Islands | Fantasy — sky platforms, teleporter bridges, wind ambient |

---

## 4. Quality Checklist

For each template before shipping:

### Structural
- [ ] At least 10 entities (not counting terrain/foliage instances)
- [ ] At least 2 distinct "areas" connected by walkable paths
- [ ] Player can walk through entire space without getting stuck
- [ ] No entities floating in air or clipping through terrain
- [ ] Scale feels human (doorways ~2m, ceilings ~3m, furniture proportional)

### Interactive
- [ ] At least 3 interactive objects (doors, collectibles, triggers)
- [ ] At least 1 collectible with score tracking
- [ ] HUD shows score or relevant info
- [ ] Click prompt ("Press E") appears near interactables

### Atmosphere
- [ ] Ambient audio plays automatically (at least 1 layer)
- [ ] Lighting appropriate to theme (warm for cozy, cool for sci-fi)
- [ ] At least 1 audio emitter on a relevant entity (campfire, waterfall, etc.)

### Distribution
- [ ] Thumbnail captured at 1200x630 (compelling at 300x200 gallery size)
- [ ] 3-5 preview screenshots from different angles
- [ ] Description is clear and enticing (2-3 sentences)
- [ ] Tags cover theme, mood, and key features
- [ ] `gen_load_world` round-trip produces identical scene

### Technical
- [ ] `gen_export_html` produces working self-contained viewer
- [ ] `gen_export_world` produces valid glTF/GLB
- [ ] Total entity count under 500 (for performance)
- [ ] No missing textures or broken material references

---

## 5. Template Metadata Format

In `SKILL.md`:

```markdown
---
name: Zen Garden
description: A peaceful Japanese-inspired garden with water features, stone paths, and ambient wind.
tags: [zen, garden, peaceful, nature, water, meditation]
difficulty: beginner
category: nature
thumbnail: screenshots/thumbnail.png
entity_count: 45
---

# Zen Garden

A tranquil garden space featuring a central water pond surrounded by stone paths,
bamboo groves, and decorative rocks. Wind rustles through the foliage while
water gently flows. Perfect for meditation or as a starting point for your own
peaceful world.

## Features
- Procedural terrain with gentle hills
- Central water pond with wave animation
- Stone walking paths
- Bamboo and tree foliage scatter
- Wind + stream ambient audio
- 5 collectible zen stones
- Score HUD tracking collected stones
```

---

## 6. MCP Tool Call Recipes

### Recipe: Basic outdoor scene

```
gen_add_terrain(size=[100,100], height_scale=10, material="grass")
gen_set_environment(sky_color=[0.5,0.7,1.0], fog_density=0.01)
gen_set_light(type="directional", intensity=10000, color=[1.0,0.95,0.9], shadows=true)
gen_set_ambience(layers=[{sound:"forest",volume:0.5},{sound:"wind",volume:0.3}])
```

### Recipe: Indoor room

```
gen_spawn_primitive(shape="cuboid", name="floor", size=[10,0.2,10], position=[0,0,0], color="#808080")
gen_spawn_primitive(shape="cuboid", name="wall_n", size=[10,3,0.2], position=[0,1.5,-5], color="#a0a0a0")
gen_spawn_primitive(shape="cuboid", name="wall_s", size=[10,3,0.2], position=[0,1.5,5], color="#a0a0a0")
gen_spawn_primitive(shape="cuboid", name="wall_e", size=[0.2,3,10], position=[5,1.5,0], color="#a0a0a0")
gen_spawn_primitive(shape="cuboid", name="wall_w", size=[0.2,3,10], position=[-5,1.5,0], color="#a0a0a0")
gen_spawn_primitive(shape="cuboid", name="ceiling", size=[10,0.2,10], position=[0,3,0], color="#909090")
gen_set_light(type="point", position=[0,2.5,0], intensity=5000, color=[1.0,0.9,0.8])
gen_set_ambience(layers=[{sound:"cave",volume:0.3}])
```

### Recipe: Water feature

```
gen_add_water(height=1.0, size=[20,20], color="#2389da", wave_amplitude=0.3)
gen_audio_emitter(position=[0,1,0], sound="water", volume=0.6, radius=15)
```
