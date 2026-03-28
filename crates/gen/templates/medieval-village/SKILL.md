---
name: "medieval-village"
description: "A rustic medieval village with town square, timber-frame houses, market stalls, church, and surrounding forest"
user-invocable: true
metadata:
  type: "world"
  genre: "medieval"
  difficulty: "starter"
  entity_count: 38
  features:
    - "parametric shapes"
    - "PBR materials"
    - "point lights"
    - "directional light"
    - "ambient audio (forest + wind)"
    - "spatial audio (fire emitters)"
    - "bob behaviors"
    - "pulse behaviors"
    - "spin behaviors"
    - "avatar (first person)"
    - "guided tour"
useWhen:
  - contains: "medieval"
  - contains: "village"
  - contains: "fantasy"
---
# Medieval Village

A rustic medieval village built entirely from parametric primitives. The scene
demonstrates a range of Gen features: shapes, PBR materials, lighting, ambient
and spatial audio, behaviors, avatar configuration, and a guided tour.

## Scene Layout

- **Town square** at origin with a central stone well
- **Timber-frame houses** (4) arranged around the square
- **Market stalls** (3) with colored awnings along the east side
- **Church** with a tall steeple to the north
- **Fenced perimeter** with wooden posts and crossbars
- **Trees** scattered outside the fence with gentle bob animation
- **Torches** with fire audio emitters and pulse glow
- **Ground plane** representing a dirt village floor

## Audio

- Ambient forest soundscape with bird calls and light wind
- Spatial fire crackle on the two torch emitters
- Spatial hum on the church bell

## Behaviors

- Market awning flags bob gently in the breeze
- Torch flames pulse (scale breathing)
- Weathervane on the church spins slowly

## How to Load

```json
gen_load_world({ "path": "medieval-village" })
```

To customize, fork it first:

```json
gen_fork_world({ "source": "medieval-village", "new_name": "my-village" })
```

Then modify entities, add NPCs, doors, or teleporters as needed.
