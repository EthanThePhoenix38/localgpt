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

## Features

- **[Tools](/docs/gen/tools)** — 32 specialized tools for scene creation, plus 25 MCP-only tools for characters, interactions, terrain, UI, and physics
- **[Behaviors](/docs/gen/behaviors)** — Data-driven animations (orbit, spin, bounce, etc.)
- **[Audio](/docs/gen/audio)** — Procedural environmental audio with spatial emitters
- **[World Skills](/docs/gen/world-skills)** — Save and load complete worlds as reusable skills
- **[MCP Server](/docs/gen/mcp-server)** — Use gen tools from Claude CLI, Gemini CLI, VS Code, Zed, and other MCP clients
- **HTML Export** — Export worlds as self-contained HTML with Three.js and Web Audio
- **Undo/Redo** — Full undo/redo support for all scene edits with persistence
- **Streaming Chat** — Real-time tool call display and streaming responses

## Templates

World templates are coming soon — ready-to-customize Fantasy, Sci-Fi, Horror, and Urban environments.

## Current Limitations

- Visual output depends on the LLM's spatial reasoning ability
- Requires a GPU-capable display for rendering

## Showcase

- **[proofof.video](https://proofof.video/)** — Video gallery comparing world generations across different models using the same or similar prompts
