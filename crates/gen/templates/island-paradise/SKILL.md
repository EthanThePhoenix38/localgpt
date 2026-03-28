---
name: "island-paradise"
description: "A tropical island with sandy beach, palm trees, lagoon, waterfall, hut, campfire, and pier"
user-invocable: true
metadata:
  type: "world"
  genre: "tropical"
  difficulty: "starter"
  entity_count: 35
  features:
    - "parametric shapes"
    - "PBR materials"
    - "transparent water (alpha blend)"
    - "point lights"
    - "directional light"
    - "ambient audio (ocean + stream)"
    - "spatial audio (fire, water)"
    - "bob behaviors (palm trees)"
    - "pulse behaviors (campfire)"
    - "path_follow behaviors (fish)"
    - "avatar (first person)"
    - "guided tour"
useWhen:
  - contains: "island"
  - contains: "tropical"
  - contains: "beach"
  - contains: "paradise"
---
# Island Paradise

A tropical island with sandy beach, palm trees, a turquoise lagoon, and a
waterfall. Built entirely from parametric primitives demonstrating water surfaces
with alpha blending, ambient ocean and stream audio, fire emitters, swaying
palm trees, and a guided walking tour.

## Scene Layout

- **Sandy beach** surrounding the island at water level
- **Central hill** rising to a peak with a waterfall on one side
- **Palm trees** (4) along the beach with gentle bob animation
- **Lagoon** with transparent water plane
- **Waterfall** with stream audio emitter at the cliff face
- **Beach hut** with thatched pyramid roof
- **Campfire** with fire audio emitter, pulse glow, and point light
- **Pier** extending into the ocean from the south beach
- **Rocks** scattered around the waterfall and shore
- **Fish** following a looping path through the lagoon

## Audio

- Ambient ocean waves (global)
- Ambient stream (global, lower volume)
- Spatial fire crackle at campfire
- Spatial water turbulence at waterfall

## How to Load

```json
gen_load_world({ "path": "island-paradise" })
```

To customize, fork it first:

```json
gen_fork_world({ "source": "island-paradise", "new_name": "my-island" })
```
