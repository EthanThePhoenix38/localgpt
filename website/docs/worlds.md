---
sidebar_position: 9.2
---

# Explorable World Systems

A comparison of AI systems that generate or interact with 3D explorable worlds.

## High-Level Comparison

| System | Input | Output | Runtime | Primary Focus |
|--------|-------|--------|---------|---------------|
| [Genie 3](https://deepmind.google/discover/blog/genie-2-a-large-scale-foundation-world-model/) | Text, Image | Interactive 3D | Cloud | Game world generation |
| [SIMA 2](https://deepmind.google/discover/blog/sima-2-next-gen-game-ai/) | Gameplay video | Agent behavior | Cloud | Game-playing AI agent |
| [WorldGen](https://www.meta.com/blog/worldgen-3d-world-generation-reality-labs-generative-ai-research/) | Text | 3D Worlds (USD) | Cloud | Structured 3D world generation |
| [Marble](https://www.worldlabs.ai/) | Text, Image, Video, 3D | Gaussian Splats, 3D | Cloud | World model for 3D generation |
| [Artcraft](https://artcraft.ai/) | Text | Images, Video | Cloud | Creative IDE for AI media |
| [Intangible](https://www.intangible.ai/) | Text | 3D Scenes | Cloud | Camera-centric scene composition |
| **LocalGPT Gen** | Text | Interactive 3D (glTF) | Local | Open-source world building |

## Feature Comparison

| Feature | Genie 3 | SIMA 2 | WorldGen | Marble | Artcraft | Intangible | LocalGPT Gen |
|---------|---------|--------|----------|--------|----------|------------|--------------|
| Text-to-3D | ✓ | — | ✓ | ✓ | — | ✓ | ✓ |
| Image-to-3D | ✓ | — | — | ✓ | — | — | — |
| Interactive playback | ✓ | ✓ | — | — | — | ✓ | ✓ |
| Real-time simulation | ✓ | ✓ | — | — | — | — | ✓ |
| Structured generation | — | — | ✓ | — | — | — | ✓ |
| Local execution | — | — | — | — | — | — | ✓ |
| Open source | — | — | — | — | — | — | ✓ |
| Procedural audio | — | — | — | — | — | — | ✓ |
| glTF/USD export | — | — | ✓ | ✓ | — | ✓ | ✓ |
| Agent control | — | ✓ | — | — | — | — | ✓ |

## System Highlights

### Genie 3 (DeepMind)

Foundation world model that generates interactive 3D environments from a single text prompt or image. Designed for rapid game prototyping and synthetic data generation.

### SIMA 2 (DeepMind)

Gemini-powered agent that learns to play 3D games by watching gameplay video. Self-improving through experience, it reasons about game objectives and adapts to new environments.

### WorldGen (Meta Reality Labs)

Structured 3D world generation from Meta Research. Uses a multi-stage pipeline — LLM generates high-level layout parameters, then procedural systems handle actual geometry placement. Outputs USD scenes with terrain, structures, vegetation, and props. Key insight: LLMs should generate *parameters*, not geometry directly. LocalGPT Gen's [WorldGen pipeline](/docs/gen/tools) implements a similar blockout-first architecture locally.

### Marble (World Labs)

Multimodal world model that creates 3D scenes from text, images, video, or 3D layouts. Exports as Gaussian splats for high-fidelity rendering.

### Artcraft

IDE for AI-assisted creative work. Combines image generation, video creation, 3D compositing, character posing, and scene blocking in a unified interface.

### Intangible

Spatial intelligence platform focused on camera-centric 3D composition. Designed for creative industries needing precise camera control and scene layout.

### LocalGPT Gen

Open-source, local-first 3D world generation powered by Bevy. Features procedural audio synthesis, data-driven behaviors, and full glTF export. Runs entirely on your machine without cloud dependencies.

**Showcases:**
- [localgpt-gen-workspace](https://github.com/localgpt-app/localgpt-gen-workspace) — "World as skill" examples: complete explorable worlds saved as reusable, shareable skills
- [proofof.video](https://proofof.video/) — Video gallery comparing world generations across different models using the same or similar prompts

See the [Gen documentation](/docs/gen) for details on LocalGPT's world generation capabilities.
