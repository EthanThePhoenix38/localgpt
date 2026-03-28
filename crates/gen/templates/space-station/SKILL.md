---
name: "space-station"
description: "A modular orbital space station with corridors, control room, airlock, observation deck, and cargo bay"
user-invocable: true
metadata:
  type: "world"
  genre: "sci-fi"
  difficulty: "starter"
  entity_count: 36
  features:
    - "parametric shapes"
    - "PBR materials (metallic surfaces)"
    - "point lights"
    - "spot lights"
    - "ambient audio (hum)"
    - "spatial audio (hum emitters)"
    - "pulse behaviors (consoles)"
    - "spin behaviors (radar dish)"
    - "orbit behaviors (satellite)"
    - "avatar (first person)"
    - "guided tour (fly mode)"
useWhen:
  - contains: "space"
  - contains: "station"
  - contains: "sci-fi"
---
# Space Station

A modular orbital space station built from parametric primitives. The interior
features metallic corridors, a control room with pulsing consoles, an airlock,
an observation deck with a viewport, and a cargo bay.

## Scene Layout

- **Central corridor** connecting all modules along the Z axis
- **Control room** (north) with console banks, screens, and a rotating radar dish
- **Airlock** (east) with inner and outer doors
- **Observation deck** (south) with a transparent viewport overlooking space
- **Cargo bay** (west) with storage containers and cargo racks
- **Exterior** radar dish with spin behavior, orbiting communications satellite

## Audio

- Ambient low-frequency hum pervading the entire station
- Spatial hum on console banks (higher frequency, warmer tone)
- Spatial hum on the airlock (mechanical resonance)

## Behaviors

- Console indicator lights pulse slowly
- Radar dish spins on the Y axis
- Communications satellite orbits the station exterior

## How to Load

```json
gen_load_world({ "path": "space-station" })
```

To customize, fork it first:

```json
gen_fork_world({ "source": "space-station", "new_name": "my-station" })
```
