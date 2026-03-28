---
name: "dungeon-crawler"
description: "A classic stone dungeon with interconnected rooms, corridors, treasure chests, pillars, torches, a gate, and stairs"
user-invocable: true
metadata:
  type: "world"
  genre: "dungeon"
  difficulty: "starter"
  entity_count: 40
  features:
    - "parametric shapes"
    - "PBR materials"
    - "point lights (torches)"
    - "spot lights (treasure)"
    - "ambient audio (cave)"
    - "spatial audio (fire emitters)"
    - "pulse behaviors (torch glow)"
    - "bounce behaviors (loose stones)"
    - "bob behaviors (treasure glow)"
    - "path_follow behaviors (patrol)"
    - "avatar (first person)"
    - "guided tour"
useWhen:
  - contains: "dungeon"
  - contains: "crawler"
  - contains: "underground"
---
# Dungeon Crawler

A classic stone dungeon built from parametric primitives. Multiple interconnected
rooms connected by narrow corridors, with torches on walls, treasure chests,
stone pillars, a portcullis gate, and descending stairs. The scene demonstrates
cave ambience, fire audio emitters, pulse glow on torches, bouncing loose stones,
and a guided exploration tour.

## Scene Layout

- **Entry chamber** (south) with descending stairs from the surface
- **Main hall** (center) with tall pillars and a vaulted feel
- **Treasure room** (east) with chests, a spot-lit golden orb, and guarded gate
- **Corridor** connecting entry to main hall and branching east to treasure room
- **Torches** mounted on walls with fire audio and pulse glow
- **Loose stones** on the floor with bounce behaviors
- **Patrol orb** following a path through the main hall

## Audio

- Ambient cave drips and deep resonance (global)
- Spatial fire crackle on all torch emitters

## Behaviors

- Torch flames pulse with breathing scale effect
- Loose stones bounce periodically
- Treasure orb bobs with a golden glow
- Patrol orb follows a path through the main hall

## How to Load

```json
gen_load_world({ "path": "dungeon-crawler" })
```

To customize, fork it first:

```json
gen_fork_world({ "source": "dungeon-crawler", "new_name": "my-dungeon" })
```
