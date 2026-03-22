# Data Loss Analysis for Gen3D Save/Load

**Last Updated:** Session 5 (2026-03-01)

## Status Summary

| Issue | Severity | Status |
|-------|----------|--------|
| DirectionalLight shadow parameters | Low | Open (rarely customized) |
| StandardMaterial texture properties | N/A | Out of scope (no texture system) |
| StandardMaterial alpha_mode/unlit/double_sided/reflectance | ~~High~~ | **FIXED** (Session 4-5) |
| MeshAssetRef not captured | ~~Critical~~ | **FIXED** (Session 5, GltfSource) |
| SetLightCmd range/angle fields | ~~Medium~~ | **FIXED** (Session 4) |
| Visibility for non-shape entities | ~~Medium~~ | **FIXED** (Session 5) |
| EntitySummary missing light/audio/behavior info | ~~Low~~ | **FIXED** (Session 5) |

---

## Remaining Open Issues

### 1. Light Shadow Parameters (Low Priority)

Bevy's lights have shadow quality parameters not captured:
- `shadow_depth_bias` - prevents shadow acne
- `shadow_normal_bias` - reduces self-shadowing artifacts
- `shadow_map_near_bound` - near plane for shadow map

**Impact:** Minimal - these are rarely customized, default values work for most scenes.

**If needed:** Add `shadow_depth_bias: Option<f32>` and `shadow_normal_bias: Option<f32>` to `LightDef`.

---

### 2. Texture-Based Material Properties (Out of Scope)

Properties requiring texture references:
- `normal_map_texture`
- `occlusion_texture`
- `parallax_mapping_method` / `parallax_depth_scale`
- `depth_map`

**Impact:** Complex imported glTF materials lose texture bindings.

**Rationale:** The gen system doesn't manage texture assets. This would require:
1. Texture asset management (copy to world directory)
2. Relative path resolution
3. Texture loading on world load

This is a larger feature beyond the current scope.

---

## Fixed Issues

### Material Properties (Session 4-5)

`MaterialDef` now captures:
- `alpha_mode: Option<AlphaModeDef>` - Opaque, Mask(f32), Blend, Add, Multiply
- `unlit: Option<bool>` - disables lighting
- `double_sided: Option<bool>` - renders both faces
- `reflectance: Option<f32>` - specular reflectance

All 8 PBR properties round-trip through spawn, modify, save, load, undo, redo, and entity_info.

### glTF Source Tracking (Session 5)

`GltfSource` component tracks source file path on imported mesh entities.
Save captures `mesh_asset: MeshAssetRef`, load re-imports glTF when present.

### Light Range/Angles (Session 4)

`LightDef` and `SetLightCmd` include:
- `range: Option<f32>` - PointLight/SpotLight max range
- `outer_angle: Option<f32>` - SpotLight outer cone
- `inner_angle: Option<f32>` - SpotLight inner cone

### Visibility (Session 5)

`Visibility::Hidden` now applies to all entity types on load (lights, meshes, groups), not just shapes.

### EntitySummary Enrichment (Session 5)

`gen_scene_info` now includes:
- `light: Option<String>` - light type if entity has light
- `audio: Option<String>` - audio emitter sound type
- `behaviors: Option<usize>` - count of attached behaviors

---

## Historical Reference

Original issues identified in Session 3:

1. **~~Light Properties - Shadow depth bias~~** - Low priority, rarely customized
2. **~~Material Data - StandardMaterial properties~~** - Core properties now captured
3. **~~Mesh Asset Reference~~** - Fixed with GltfSource component
4. **~~SetLightCmd range/angle fields~~** - Fixed in Session 4
5. **~~Visibility for non-shape entities~~** - Fixed in Session 5
