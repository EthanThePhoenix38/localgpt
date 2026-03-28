---
name: "zen-garden"
description: "A peaceful Japanese zen garden with raked sand, rock arrangements, bridge, pagoda, cherry trees, stone lanterns, and water feature"
user-invocable: true
metadata:
  type: "world"
  genre: "zen"
  difficulty: "starter"
  entity_count: 38
  features:
    - "parametric shapes"
    - "PBR materials"
    - "point lights (lanterns)"
    - "directional light"
    - "ambient audio (stream)"
    - "spatial audio (water emitter)"
    - "bob behaviors (cherry trees)"
    - "orbit behaviors (floating petals)"
    - "spin behaviors (water wheel)"
    - "look_at behaviors (lantern)"
    - "avatar (third person)"
    - "guided tour (walk mode)"
useWhen:
  - contains: "zen"
  - contains: "garden"
  - contains: "japanese"
  - contains: "peaceful"
---
# Zen Garden

A tranquil Japanese zen garden built from parametric primitives. The scene
features raked sand patterns, carefully arranged rock formations, an arched
bridge over a stream, a traditional pagoda, cherry trees with swaying canopies,
stone lanterns, and floating cherry blossom petals.

## Scene Layout

- **Raked sand garden** covering the central area
- **Rock arrangement** (3 stones in classic asymmetric grouping)
- **Stream** flowing along the north edge with a water audio emitter
- **Arched bridge** crossing the stream
- **Pagoda** in the northwest corner with tiered roofs
- **Cherry trees** (3) with bob animation simulating gentle wind
- **Stone lanterns** (2) with warm point lights, one tracking the avatar
- **Floating petals** orbiting the cherry trees
- **Water wheel** spinning near the stream
- **Bamboo fence** bordering the east side

## Audio

- Ambient stream (global, gentle flow)
- Ambient wind (global, very light)
- Spatial water emitter at the stream crossing

## Behaviors

- Cherry tree canopies bob gently (wind sway)
- Floating petals orbit around cherry trees
- Water wheel spins near the stream
- East lantern uses look_at to follow the avatar spawn point

## How to Load

```json
gen_load_world({ "path": "zen-garden" })
```

To customize, fork it first:

```json
gen_fork_world({ "source": "zen-garden", "new_name": "my-garden" })
```
