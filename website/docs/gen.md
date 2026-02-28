---
sidebar_position: 14
---

# LocalGPT Gen

**LocalGPT Gen** is a built-in world generation mode. You type natural language, and the AI builds explorable worlds — geometry, materials, lighting, behaviors, audio, and camera. All inside the same single Rust binary, powered by [Bevy](https://bevyengine.org/).

## Demo Videos

<iframe width="100%" height="400" src="https://www.youtube.com/embed/n18qnSDmBK0" title="LocalGPT Gen Demo" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

<br/>

<iframe width="100%" height="400" src="https://www.youtube.com/embed/cMCGW7eMUNE" title="LocalGPT Gen Demo" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

## Installation

```bash
# Install the standalone Gen binary
cargo install localgpt-gen

# Or from a source checkout
cargo install --path crates/gen
```

## Usage

```bash
# Interactive mode — type prompts in the terminal
localgpt-gen

# Start with an initial prompt
localgpt-gen "create a heart outline with spheres and cubes"

# Load an existing glTF/GLB scene
localgpt-gen --scene ./scene.glb

# Verbose logging
localgpt-gen --verbose

# Combine options
localgpt-gen -v -s ./scene.glb "add warm lighting"

# Custom agent ID (default: "gen")
localgpt-gen --agent my-gen-agent
```

The agent receives your prompt and iteratively builds a world — spawning shapes, adjusting materials, positioning the camera, and taking screenshots to course-correct. Type `/quit` or `/exit` in the terminal to close.

## Gen Tools

The gen agent has access to 24 specialized tools:

### Scene Query

| Tool | Description |
|------|-------------|
| `gen_scene_info` | Get complete scene hierarchy |
| `gen_screenshot` | Capture viewport screenshot |
| `gen_entity_info` | Get detailed info about a named entity |

### Entity Creation

| Tool | Description |
|------|-------------|
| `gen_spawn_primitive` | Spawn geometric primitives (sphere, cube, cylinder, torus, etc.) |
| `gen_spawn_mesh` | Spawn custom mesh geometry |
| `gen_load_gltf` | Load entities from a glTF/GLB file |

### Entity Modification

| Tool | Description |
|------|-------------|
| `gen_modify_entity` | Modify entity transform, material, or visibility |
| `gen_delete_entity` | Remove an entity and its children |

### Camera & Environment

| Tool | Description |
|------|-------------|
| `gen_set_camera` | Position and orient the camera |
| `gen_set_light` | Configure scene lighting |
| `gen_set_environment` | Set background color and ambient light |

### Export

| Tool | Description |
|------|-------------|
| `gen_export_screenshot` | Export high-res image to file |
| `gen_export_gltf` | Export scene as glTF/GLB file |

### Behaviors

Data-driven animations that stack on entities — no scripting required.

| Tool | Description |
|------|-------------|
| `gen_add_behavior` | Add a behavior (orbit, spin, bob, look_at, pulse, path_follow, bounce) |
| `gen_remove_behavior` | Remove a behavior from an entity |
| `gen_list_behaviors` | List all behaviors on an entity |
| `gen_pause_behaviors` | Pause or resume all behaviors globally |

### Audio

Procedural environmental audio with spatial emitters.

| Tool | Description |
|------|-------------|
| `gen_set_ambience` | Set ambient soundscape (wind, rain, forest, ocean, cave, stream) |
| `gen_audio_emitter` | Attach a sound emitter to an entity |
| `gen_modify_audio` | Modify an existing audio emitter |
| `gen_audio_info` | Get audio system status |

### World Skills

Save and load complete worlds as reusable skills.

| Tool | Description |
|------|-------------|
| `gen_save_world` | Save scene, behaviors, and audio to a skill directory |
| `gen_load_world` | Load a saved world (clears existing scene by default) |
| `gen_clear_scene` | Clear all entities, behaviors, and audio |

## Architecture

Bevy requires ownership of the main thread (macOS windowing/GPU requirement), so LocalGPT Gen uses a split-thread architecture:

- **Main thread** — Bevy engine runs the render loop and processes scene commands
- **Background thread** — Tokio runtime runs the agent loop, making LLM calls and issuing tool commands
- **Communication** — Async mpsc channels bridge the two threads

```
┌─────────────────────┐     mpsc channels     ┌─────────────────────┐
│    Main Thread      │◄─────────────────────►│  Background Thread  │
│                     │                        │                     │
│  Bevy Engine        │   ToolRequest ──►      │  Tokio Runtime      │
│  - Rendering        │   ◄── ToolResult       │  - Agent Loop       │
│  - Scene Graph      │                        │  - LLM API Calls    │
│  - Window/GPU       │                        │  - Tool Execution   │
└─────────────────────┘                        └─────────────────────┘
```

## Current Limitations

- Visual output depends on the LLM's spatial reasoning ability
- Requires a GPU-capable display for rendering

## Share Your Creations

Created something awesome with LocalGPT Gen? We'd love to see it! Join the community on [Discord](https://discord.gg/yMQ8tfxG) and share your world generation results — showcase your creative prompts, stunning scenes, and experimental ideas with fellow LocalGPT users.
