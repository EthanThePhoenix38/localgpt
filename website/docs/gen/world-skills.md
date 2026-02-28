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
├── world.toml        # World manifest (environment, camera)
├── scene.glb         # glTF binary with all entities
├── behaviors.toml    # Behavior definitions
└── audio.toml        # Audio emitter and ambience config
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
