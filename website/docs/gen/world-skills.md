---
sidebar_position: 14.4
---

# World Skills

Save and load complete worlds as reusable skills. Worlds are stored as skill directories containing all scene data.

## World Format

A saved world consists of:

```
~/.localgpt/workspace/skills/my-world/
├── SKILL.md          # Skill description for LLM context
├── world.toml        # World manifest (environment, camera, avatar)
├── scene.glb         # glTF binary with all entities
├── behaviors.toml    # Behavior definitions
├── audio.toml        # Audio emitter and ambience config
└── tours.toml        # Guided tour definitions
```

### world.toml

The world manifest configures environment, camera, and avatar settings:

```toml
[environment]
background_color = [0.1, 0.1, 0.2]
ambient_light = [0.2, 0.2, 0.3]

[camera]
position = [0, 5, 10]
look_at = [0, 0, 0]
fov = 60

[avatar]
# User presence in the world
spawn_position = [0, 1, 5]
pov_mode = "first_person"  # or "third_person"
movement_speed = 5.0
height = 1.8
# Optional: 3D model for third-person view
# model = "avatar.glb"
```

### tours.toml

Define guided tours with waypoints:

```toml
[[tour]]
name = "overview"
description = "A quick tour of the main areas"

[[tour.waypoint]]
position = [0, 3, 10]
look_at = [0, 0, 0]
description = "Welcome to the scene overview"
pause_seconds = 3
movement = "fly"  # "walk", "fly", or "teleport"

[[tour.waypoint]]
position = [10, 2, 0]
look_at = [0, 1, 0]
description = "Here's the main structure"
pause_seconds = 5
movement = "fly"
```

## Saving Worlds

```json
gen_save_world({
  "name": "forest-clearing",
  "description": "A peaceful forest clearing with stream and campfire"
})
```

This saves the current scene to `~/.localgpt/workspace/skills/forest-clearing/`.

## Loading Worlds

```json
gen_load_world({
  "path": "forest-clearing"
})
```

By default, loading a world clears the existing scene first. To preserve existing entities:

```json
gen_load_world({
  "path": "forest-clearing",
  "clear": false
})
```

You can also load by full path:

```json
gen_load_world({
  "path": "/path/to/world-skill-directory"
})
```

## Clearing Scenes

To clear all entities, behaviors, and audio without loading a new world:

```json
gen_clear_scene({
  "keep_camera": true,
  "keep_lights": true
})
```

## Deferred Loading

glTF scenes load asynchronously. When loading a world:

1. The glTF file is parsed and entities spawn over several frames
2. Behaviors and audio emitters are applied after entities spawn
3. The response confirms when loading is complete

This ensures smooth loading even for complex scenes.

## Showcase

- **[localgpt-app/workspace](https://github.com/localgpt-app/workspace)** — "World as skill" examples: complete explorable worlds saved as reusable, shareable skills
