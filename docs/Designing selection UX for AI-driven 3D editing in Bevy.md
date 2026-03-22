# Designing selection UX for AI-driven 3D editing in Bevy

**The optimal "select then prompt" system for LocalGPT Gen should combine a hybrid ECS selection model (marker component + resource), Bevy's built-in picking pipeline, and a JSON-based LLM function-calling protocol that serializes selected objects with spatial context.** This architecture mirrors patterns proven across professional 3D tools and emerging AI editors, while leveraging Bevy 0.18's native picking infrastructure. The research synthesizes selection UX from Blender, Unity, Unreal, Maya, and Cinema 4D alongside AI-driven tools like Spline AI, Figma AI, and research projects like SceneGPT and LLMER to deliver concrete implementation recommendations.

---

## Professional 3D tools converge on shared selection fundamentals

Every major 3D application — Blender, Unity, Unreal, Maya, Cinema 4D — implements a remarkably consistent core: **click to select (replacing previous selection), modifier key for additive selection, and rectangular marquee for area selection**. Orange/yellow outlines dominate as the universal selection color. Beyond this baseline, three patterns matter most for LocalGPT Gen.

**The active object distinction** separates the last-selected object from the rest of the selection. Blender highlights the active object in yellow while other selected objects show orange outlines. Maya and Cinema 4D implement similar concepts. This is critical for AI prompting because the active object becomes the implicit referent for pronouns like "this" and "it," while "these" maps to the full selection set.

**Hierarchical selection propagation** varies significantly. Unity's `SelectionBase` attribute automatically promotes clicks on prefab children to the prefab root — a first click selects the group, a double-click enters it. Unreal's locked actor groups behave similarly: clicking any group member selects the whole group, with green brackets indicating locked state and red brackets for unlocked. Blender deliberately avoids auto-propagation — selecting a parent does not select its children, requiring explicit `Select Hierarchy` commands. **For an AI editor, Unity's two-tier model (click selects group, double-click enters) is the strongest pattern** because it maps cleanly to AI operation scope: group-level operations vs. component-level operations.

**Semantic selection** — selecting by attribute rather than spatial interaction — appears across all professional tools. Blender offers Select by Material, Select by Type, Select All by Trait, and Select Pattern (name wildcards). Unreal provides Select Matching, Select All With Same Material, and Select Relevant Lights. Maya has selection masks that filter selectable types with fine granularity. These capabilities become essential in an AI editor where a user might say "select all the wooden furniture" — the system needs semantic tags on objects to support fuzzy/semantic selection.

| Modifier | Function | Standard across |
|----------|----------|----------------|
| **Shift+Click** | Add to / toggle selection | Blender, Unity, Cinema 4D, Maya |
| **Ctrl+Click** | Add/remove (viewport) | Unreal, Maya (remove), Cinema 4D |
| **B / drag** | Box/marquee select | All tools |
| **A** | Select all / deselect all | All tools |
| **Ctrl+I** | Invert selection | Blender, Maya, Unreal |

---

## Spatial and VR selection techniques inform 3D interaction design

VR creative tools solve a harder version of the selection problem — no mouse precision, no keyboard shortcuts, full 3D spatial context. Gravity Sketch's approach stands out: an **adjustable-radius grab sphere** on the dominant controller hand, sized via thumbstick, with objects highlighting when they enter the sphere's volume. This proximity-based model translates directly to a potential "spatial selection mode" in LocalGPT Gen where users could define a selection volume rather than clicking individual objects.

Academic VR research offers two particularly relevant techniques. **IntenSelect** uses a cone-shaped selection volume with time-dependent scoring — objects accumulate selection scores based on dwell time within the cone, with the highest-scoring object indicated by a ray bending toward it. This handles cluttered scenes where precise targeting is difficult. **SQUAD** (Sphere-casting refined by QUAD-menu) starts with a broad sphere selection and progressively narrows via refinement menus, achieving much higher accuracy than basic ray casting for small targets.

Apple Vision Pro introduced the most elegant selection model for spatial computing: **eye tracking for targeting combined with pinch gestures for confirmation**. Every interactive element shows hover state on gaze, providing continuous feedback about what would be selected. This gaze-hover pattern should inform LocalGPT Gen's cursor-hover behavior — **always show what would be selected before the click**, with clear visual highlighting on hover that distinguishes from the post-selection highlight.

For LocalGPT Gen's desktop interface, the key takeaway is implementing hover preview (highlight on mouseover before click), supporting volume selection tools (box and potentially sphere/lasso), and ensuring the selection visualization clearly communicates three states: unhovered, hovered (potential selection), and selected.

---

## AI "select then prompt" workflows are crystallizing around a common pattern

Across Spline AI, Figma AI, Adobe Substance 3D, Meshy, and Masterpiece Studio, a consistent UX flow has emerged for AI-augmented editing:

**Step 1: Select.** User selects one or more objects using standard selection tools. **Step 2: Prompt.** A text input field appears (contextual panel, chat interface, or inline prompt bar) where the user describes the desired change. **Step 3: Preview.** The AI generates one or more result variations — Spline shows instant texture application, Figma provides live preview with style controls, Meshy generates **4 variations** for user choice. **Step 4: Confirm.** User accepts, iterates with a refined prompt, or rejects.

Spline AI's texture workflow is the closest analog to LocalGPT Gen's target: select an object in the 3D viewport, open the material panel, click "Generate with AI," type a description ("weathered stone"), and the texture applies directly. Figma AI extends this to multi-element selection — selecting parts of a design and prompting changes only to selected elements. This **scoped AI operation** is precisely what LocalGPT Gen needs.

The critical UX decision is **where the prompt input lives**. Three viable approaches exist. An **inline prompt bar** (like Cursor's Cmd+K) appears near the selection, minimizing context switching. A **persistent chat panel** (like Copilot Chat) maintains conversation history for follow-up prompts. A **contextual popover** appears on selection, directly attached to the selected objects. **The recommendation for LocalGPT Gen is a hybrid: persistent chat panel for complex multi-step operations, with an inline Cmd+K prompt bar for quick single operations.** The inline bar should show the selection context ("3 objects selected: Wall_01, Wall_02, Floor") so the user knows exactly what "these" refers to.

Autodesk's generative design in Fusion 360 offers a different but instructive pattern: users don't prompt with natural language but instead define **preserved geometry** (regions that must remain), **obstacle geometry** (keep-out zones), and **constraints** (loads, materials, manufacturing methods). The AI then generates multiple optimized alternatives. For LocalGPT Gen, this suggests supporting constraint-style prompts alongside natural language: "make this wall taller but keep the door opening" combines a directive with a constraint.

---

## Bevy 0.18 provides a solid foundation for the selection system

Bevy's picking system was **fully upstreamed into core** starting with Bevy 0.15, and the legacy `bevy_mod_picking` crate was archived in March 2025. In Bevy 0.18, `bevy::picking` is included in `DefaultPlugins` and provides a multi-stage pipeline: input gathering → backend raycasting → hover map computation → high-level event emission with entity hierarchy bubbling.

**`MeshPickingPlugin`** is the critical addition for 3D selection. It performs **triangle-level ray-mesh intersection** — not bounding-box approximation — on all meshes with `RenderAssetUsages::MAIN_WORLD`. The system produces `Pointer<Click>`, `Pointer<Over>`, `Pointer<Out>`, and drag events that bubble up the entity hierarchy. This hierarchy bubbling is directly useful for implementing Unity-style group selection: place a `Pickable` component on the parent entity, observe `Click` events that bubble up from children, and intercept them at the group level.

For the selection state itself, the **recommended hybrid approach** combines a `Selected` marker component with a `SelectionState` resource:

```rust
#[derive(Component)]
#[component(storage = "SparseSet")]
struct Selected;

#[derive(Resource, Default)]
struct SelectionState {
    entities: Vec<Entity>,
    primary: Option<Entity>,  // Active object (last selected)
}
```

Using `SparseSet` storage for the `Selected` component is an important optimization — it eliminates archetype table migration when selection changes frequently, trading slightly slower iteration for much faster add/remove. The `SelectionState` resource provides ordered access, primary/active object tracking, and easy serialization for AI context. Keeping both in sync via a dedicated system ensures you get the best of both worlds: efficient ECS queries via `With<Selected>` filters and convenient centralized access via the resource.

**Selection change detection** leverages Bevy's observer system introduced in 0.17+:

```rust
commands.spawn((Mesh3d(mesh), MeshMaterial3d(material)))
    .observe(|click: On<Pointer<Click>>, mut commands: Commands| {
        commands.entity(click.entity).insert(Selected);
    });
```

For visual feedback, **`bevy_mod_outline`** (v0.10.3) remains the primary crate for outline rendering, using vertex extrusion with a separate render pass. It supports configurable outline color and width, stencil-based occlusion, skinned meshes, and even includes a picking example with shift-click selection. An important caveat: the latest published version targets Bevy 0.16, so check for a 0.18-compatible release. Bevy's built-in `Gizmos` system provides a lightweight alternative for bounding box wireframes:

```rust
fn draw_selection_bounds(mut gizmos: Gizmos, query: Query<(&Transform, &Aabb), With<Selected>>) {
    for (transform, aabb) in &query {
        gizmos.aabb_3d(*aabb, *transform, Color::srgb(1.0, 1.0, 0.0));
    }
}
```

**Performance at scale** is the primary concern. The default `MeshPickingPlugin` performs brute-force triangle intersection every frame — acceptable for hundreds of entities but problematic for thousands. Three mitigation strategies exist: (1) set `MeshPickingSettings::require_markers = true` and only add `Pickable` to interactable entities, (2) use the community `bevy_picking_bvh_backend` crate which provides BVH-accelerated raycasting with orders-of-magnitude speedup, or (3) use a physics engine backend (Avian or Rapier) which provides built-in spatial acceleration structures.

The complete crate stack for LocalGPT Gen's selection system:

- **bevy 0.18** (`bevy_picking` + `MeshPickingPlugin`) — core picking
- **bevy_mod_outline** — selection outlines (check for 0.18 compatibility)
- **bevy_egui** (0.39.1, confirmed 0.18 compatible) — UI panels, prompt input, selection info
- **transform-gizmo-bevy** — translate/rotate/scale gizmos on selected objects
- **bevy_picking_bvh_backend** — BVH acceleration for large scenes

---

## Serializing selection context for LLM consumption

The research identifies **SceneGPT's JSON scene graph format** as the dominant pattern for feeding 3D context to LLMs. GPT-4 and similar models are specifically trained to reason about JSON, making it far more reliable than alternative formats. The key is translating Bevy's internal representation to a human-readable, LLM-friendly schema.

The recommended context structure includes three layers: **selected objects** (full detail), **nearby objects** (abbreviated), and **scene summary** (one-line overview). For selected objects, include name, semantic tags, mesh type, transform (position/rotation in Euler degrees/scale), bounding box, material properties (base color, metallic, roughness), and parent-child relationships. For nearby objects, include only name, relationship to selection ("below, adjacent"), distance, and position. Crucially, **use Euler angles instead of quaternions** (LLMs understand "rotated 45 degrees on Y axis" but struggle with quaternion components) and **truncate coordinates to 2 decimal places** to minimize token usage.

**LLMER** (2025) demonstrated that JSON-based protocols for scene manipulation are significantly more reliable than code generation approaches. When an LLM generates Python or Rust code to modify a scene, hallucination-induced syntax errors cause crashes. When it generates structured JSON function calls, the receiving system can validate the schema before execution, gracefully reject malformed calls, and provide clear error messages back to the LLM.

The recommended function-calling architecture uses a **two-tier API**:

**Low-level atomic operations**: `set_transform`, `set_material_property`, `spawn_entity`, `despawn_entity`, `reparent`. These are composable primitives the LLM can combine for any operation.

**High-level semantic operations**: `arrange_in_pattern` (circle, grid, line), `align_objects` (along axis), `duplicate_and_place`, `apply_style` (weathered, polished, rustic). These encode common intent patterns that would otherwise require multiple low-level calls.

Following 3D-GPT's multi-agent architecture, the LLM should first attempt to match user intent to high-level operations, falling back to composing low-level operations when no high-level match exists. The system prompt should explicitly bind pronouns: "The user has selected: [Wall_01, Wall_02]. References like 'this', 'it', 'these' refer to these selected objects."

**Bevy Remote Protocol (BRP)** provides existing JSON-RPC 2.0 infrastructure in bevy 0.18 that can serve as the communication layer. BRP already supports `world.get_components`, `world.query`, and `world.insert_components` — LocalGPT Gen's AI pipeline can build directly on this foundation rather than creating a custom protocol.

---

## Preview, confirmation, and undo complete the interaction loop

The final piece of the "select then prompt" workflow is how AI-proposed changes are presented, reviewed, and committed. VS Code's Copilot Edits model provides the strongest precedent: changes appear **in-place** with a review flow, users can **accept or discard each change individually**, and undo treats the entire AI operation as a single group.

For 3D, this translates to a **ghost preview system**: when the AI proposes changes, render the results as semi-transparent overlays. Green wireframe for additions, red for deletions, yellow for modifications showing both old and new positions. A change list panel shows a textual summary ("Wall_01: scale Y 5.0 → 7.0, material → brick_weathered") with per-change accept/reject checkboxes. This mirrors how Figma AI shows live previews with style adjustment controls, allowing the user to iterate before committing.

Undo/redo requires a **command pattern** implementation that snapshots pre-state for all modified entities:

```rust
struct AiOperationRecord {
    prompt: String,
    changes: Vec<EntityChange>,
}

enum EntityChange {
    Modified { entity: Entity, component: String, before: Value, after: Value },
    Spawned { entity: Entity, components: HashMap<String, Value> },
    Despawned { entity: Entity, snapshot: HashMap<String, Value> },
}
```

All changes from a single AI prompt should be grouped into one undo record. Bevy's `Reflect` system and `ReflectSerializer` enable snapshotting arbitrary component state to JSON, making the before/after recording straightforward.

**Conversation memory** is essential for iterative editing. Following LLMER's approach, maintain the last **10 interactions** in the prompt context for pronoun resolution. This enables exchanges like: "Make this wall taller" → [applied] → "Actually a bit shorter" → [the AI knows "this" still refers to the wall and "shorter" is relative to the just-applied height].

---

## Conclusion: the complete recommended architecture

LocalGPT Gen's selection system should implement a **four-layer architecture**:

1. **Picking layer**: Bevy's built-in `MeshPickingPlugin` with BVH backend for performance, hover highlighting via material emission changes, and click/shift-click/ctrl-click handling through `Pointer<Click>` observers with keyboard state checks.

2. **Selection state layer**: Hybrid `Selected` marker component (SparseSet storage) plus `SelectionState` resource. Support active object (last selected), ordered selection list, and selection changed events via component observers. Implement Unity-style two-tier group selection — first click selects the group parent, double-click enters to select children.

3. **Context serialization layer**: Build on Bevy Remote Protocol's JSON-RPC foundation. Serialize selected objects to a SceneGPT-style JSON schema with name, tags, transform (Euler angles), bounding box, material properties, and parent-child relationships. Compute spatial relationships (above/below/adjacent) for nearby objects via k-nearest-neighbor lookup on bounding box centers.

4. **AI interaction layer**: Two-tier function-calling schema (high-level semantic + low-level atomic operations), ghost preview rendering for proposed changes, per-change accept/reject UI via `bevy_egui`, batch undo grouping, and conversation memory buffer for iterative refinement.

The most novel insight from this research is that **the selection system IS the AI's context window**. Every UX decision about what gets selected, how groups propagate, and what metadata attaches to objects directly determines what the LLM can reason about. Investing in rich semantic tagging, spatial relationship extraction, and clear selection visualization pays dividends not just in UX quality but in AI operation accuracy. The tools that get this right — Figma AI's scoped operations on selected elements, Spline's direct object-to-prompt pipeline — succeed precisely because their selection system doubles as their AI context system.