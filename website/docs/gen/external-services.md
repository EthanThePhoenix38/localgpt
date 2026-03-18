---
sidebar_position: 14.9
---

# External Services

Most of LocalGPT Gen runs entirely locally with no external dependencies. Three features require optional local services running on your machine — no cloud APIs, no remote calls.

## Overview

| Feature | Service | What It Does | GPU Required |
|---------|---------|--------------|-------------|
| [NPC Intelligence](#ollama--npc-brains) | Ollama | Autonomous NPC behavior via local LLMs | 2-7 GB |
| [Depth Preview](#comfyui--depth-preview) | ComfyUI | Styled 2D preview from depth maps | 2-8 GB |
| [3D Asset Generation](#model-server--3d-assets) | Model server | Text-to-3D mesh generation | 5-16 GB |

All services run on `localhost`. No API keys required.

## Quick Start

```bash
# Easiest: NPC brains (single binary, no Python)
brew install ollama   # or: curl -fsSL https://ollama.ai/install.sh | sh
ollama pull llama3.2:3b

# Moderate: depth-conditioned preview (Python)
git clone https://github.com/comfyanonymous/ComfyUI.git
cd ComfyUI && pip install -r requirements.txt
python main.py --port 7860

# Advanced: 3D asset generation (Python + large GPU)
# (Model server implementation pending)
```

## Ollama — NPC Brains

Attaches autonomous AI brains to NPCs. Each NPC runs a local language model that perceives the scene, reasons about goals, and outputs actions every few seconds.

### Install

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Start server (if not already running as service)
ollama serve

# Pull models
ollama pull llama3.2:3b       # NPC brain (text-only, 2 GB)
ollama pull llava1.6:7b       # NPC vision (optional, 5 GB)
```

### How It Works

```
gen_set_npc_brain { entity: "elder", personality: "wise sage" }
  → Spawns brain loop (ticks every 2 seconds):
    1. Perceive: gather nearby entities, player distance
    2. Build prompt: personality + goals + memories + perception
    3. Infer: POST to Ollama localhost:11434/api/generate
    4. Parse: extract action (speak, move_to, emote, etc.)
    5. Execute: trigger Bevy systems
```

### NPC Actions

| Action | Example | Result |
|--------|---------|--------|
| `speak("Hello!")` | Speech bubble, auto-dismiss |
| `move_to(10, 0, 5)` | Walk to position |
| `look_at("player")` | Smooth rotation |
| `emote(wave)` | Animation/particles |
| `interact("door")` | Trigger interaction system |
| `wander` | Random nearby movement |

### Supported Models

| Model | VRAM | Best For |
|-------|------|----------|
| `llama3.2:3b` | 2-3 GB | Default. Good action decisions. |
| `mistral:7b` | 5-7 GB | Faster, instruction-tuned. |
| `neural-chat:7b` | 5-6 GB | Optimized for dialogue. |
| `llava1.6:7b` | 5-7 GB | Vision (NPC visual observation). |
| `moondream2` | 2-3 GB | Lightweight vision alternative. |

### Tools

| Tool | Description |
|------|-------------|
| `gen_set_npc_brain` | Attach an AI brain to an NPC with personality and goals |
| `gen_npc_observe` | Render from NPC's viewpoint and describe what they see (requires vision model) |
| `gen_set_npc_memory` | Set NPC memory capacity and initial memories |

### Performance

- **Max concurrent brains:** 4 recommended
- **Distance culling:** Brain deactivates when NPC > 50m from player
- **Response latency:** ~200-500ms on GPU, ~2-5s on CPU
- **Tick rate:** Default 2.0 seconds (configurable per NPC)

:::info Status
The Rust infrastructure is complete (brain config, context builder, action parser, memory system, MCP tools). The HTTP client wiring to Ollama is pending.
:::

## ComfyUI — Depth Preview

Generates a styled 2D preview image from a scene's depth map. Lets the AI validate creative direction (colors, mood, style) before committing to full 3D generation.

### Install

```bash
git clone https://github.com/comfyanonymous/ComfyUI.git
cd ComfyUI
pip install -r requirements.txt
python main.py --listen 0.0.0.0 --port 7860

# Download ControlNet depth model
cd models/controlnet
wget https://huggingface.co/lllyasviel/control_v11f1p_sd15_depth/resolve/main/diffusion_pytorch_model.safetensors
```

### How It Works

```
gen_render_depth (fully implemented)
  → depth map PNG (grayscale)
    ↓
gen_preview_world (scaffolded — needs HTTP client)
  → POST depth map + prompt + style to ComfyUI
    ↓
  ← styled 2D preview PNG
```

### Style Presets

| Preset | Style |
|--------|-------|
| `realistic` | Photorealistic, high detail, natural lighting |
| `stylized` | Pixar-style 3D render, vibrant colors |
| `pixel_art` | 16-bit retro game style |
| `watercolor` | Soft edges, muted colors |
| `concept_art` | Painterly, atmospheric perspective |

### GPU Requirements

- 2-4 GB VRAM for SD 1.5 + ControlNet
- 6-8 GB VRAM for SDXL + ControlNet
- ~10-30 seconds per image at 512x512

### Tools

| Tool | Description |
|------|-------------|
| `gen_render_depth` | Render a depth map from the current scene (fully implemented) |
| `gen_preview_world` | Generate a styled 2D preview from a depth map (needs service) |

:::info Status
Depth map rendering is fully implemented. The HTTP client to ComfyUI is pending.
:::

## Model Server — 3D Assets

Generates 3D meshes from text prompts using local open-source models. The AI calls `gen_generate_asset`, gets a task ID, and the mesh auto-spawns when ready.

### How It Works

```
gen_generate_asset { prompt: "medieval sword", model: "triposg" }
  → Task queued, returns task_id + ETA
    ↓
Background: POST to model server at localhost:8741
  → GPU inference (30-180 seconds)
    ↓
Complete: .glb copied to skill assets, auto-spawned into scene
```

### Supported Models

| Model | VRAM | Speed (standard) | Output |
|-------|------|-------------------|--------|
| TripoSG | 8 GB | 30s | Mesh only |
| Hunyuan3D 2mini | 5-6 GB | 45s | Mesh + PBR textures |
| Hunyuan3D 2.1 | 10 GB | 60s | Full PBR |
| Step1X-3D | 16 GB | 90s | Mesh + PBR + LoRA |

### Tools

| Tool | Description |
|------|-------------|
| `gen_generate_asset` | Queue a 3D mesh generation task |
| `gen_asset_status` | Check generation progress |
| `gen_list_assets` | List all asset generation tasks |

:::info Status
The Rust infrastructure is complete (task manager, MCP tools, command handlers). The Python model server (`localgpt-model-server`) is pending implementation.
:::
