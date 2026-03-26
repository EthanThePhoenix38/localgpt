# RFC Addendum: Drift Detection and Reconciliation Between .md, .ron, and Live Scene

**Status:** Implemented (Archived 2026-03-25). See drift detection in `crates/gen/src/gen3d/`.
**Parent RFC:** RFC-Iterative-Multi-File-WorldGen
**Date:** 2026-03-21
**Dependencies:** Parent RFC Phase 2 (MCP tools: `gen_write_region`, `gen_load_region`, `gen_unload_region` must land first). Drift tools cannot begin implementation until those are available.

---

## The Three-Way Consistency Problem

The multi-file world format introduces three representations of the same world that can each be edited independently:

```
  .md files                .ron files              Live Bevy Scene
  (design intent)          (structured data)       (runtime state)
  ┌──────────┐             ┌──────────┐            ┌──────────┐
  │ "tavern   │             │ Cuboid(  │            │ Entity   │
  │  at [3,0,0]│            │  x:6,y:5,│            │ Transform│
  │  warm brown│            │  z:4)    │            │ {3,0,0}  │
  │  wood"     │             │ color:   │            │ Material │
  │            │             │ [.55,.35,│            │ Mesh     │
  └──────────┘             │  .2,1.0] │            └──────────┘
                            └──────────┘
```

**Five edit paths create drift:**

| # | Edit Path | Example | What drifts |
|---|-----------|---------|-------------|
| 1 | LLM writes .md, doesn't update .ron | LLM adds "add a chimney to the tavern" to the .md | .md ahead of .ron and scene |
| 2 | LLM calls MCP tools, scene changes | `gen_modify_entity("tavern", position: [5,0,0])` | Scene ahead of .md and .ron |
| 3 | User manually edits .ron | User opens village-center.ron, changes color | .ron ahead of .md and scene |
| 4 | User manually edits .md | User opens village-center.md, adds design notes | .md ahead of .ron and scene |
| 5 | LLM writes .ron, doesn't update .md | LLM generates entities but skips intent doc | .ron ahead of .md |

---

## Core Design: Content Fingerprints + Sync Metadata

### The Sync Manifest

Every world directory gets a `meta/.sync.ron` file that tracks the last-known-consistent state of each file pair:

```ron
SyncManifest(
    // When this manifest was last updated
    updated_at: "2026-03-21T14:30:00Z",

    // Per-domain sync records
    domains: {
        "layout/blockout": SyncRecord(
            md_hash: "x1y2z3w4",
            ron_hash: "p5q6r7s8",
            scene_hash: None,           // Layout has no direct scene entities
            md_mtime: "2026-03-21T14:20:00Z",
            ron_mtime: "2026-03-21T14:20:00Z",
            last_sync: "2026-03-21T14:20:00Z",
            sync_direction: MdToRon,
            status: Clean,
        ),
        "regions/village-center": SyncRecord(
            md_hash: "a1b2c3d4",        // SHA-256 prefix of .md content
            ron_hash: "e5f6g7h8",        // SHA-256 prefix of .ron content
            scene_hash: "i9j0k1l2",      // Hash of entity state when last synced
            md_mtime: "2026-03-21T14:25:00Z",
            ron_mtime: "2026-03-21T14:28:00Z",
            last_sync: "2026-03-21T14:28:00Z",
            sync_direction: RonToScene,  // Which direction last sync went
            status: Clean,               // Clean | MdAhead | RonAhead | SceneAhead | Conflict
        ),
        "regions/forest-edge": SyncRecord(
            // ...
            status: MdAhead,  // .md was edited after last sync
        ),
        "behaviors/water-effects": SyncRecord(
            // ...
            status: Conflict,  // Both .md and .ron changed since last sync
        ),
    },

    // Global: hash of root world.ron environment/camera/sky
    // world.md ↔ world.ron is tracked as the "root" domain
    root_md_hash: "t1u2v3w4",
    root_ron_hash: "m3n4o5p6",
    root_scene_hash: "q7r8s9t0",
)
```

### Hash Computation

Content hashes serve as the primary drift detection mechanism:

```rust
/// Compute a content fingerprint for a .ron region file.
/// Normalizes whitespace and comment stripping before hashing
/// so that formatting-only changes don't trigger false drift.
fn ron_content_hash(path: &Path) -> Result<String, Error> {
    let content = std::fs::read_to_string(path)?;
    // Parse and re-serialize to normalize formatting
    let parsed: ron::Value = ron::from_str(&content)?;
    let normalized = ron::ser::to_string(&parsed)?;
    let hash = sha256_prefix(&normalized, 8); // 8-char prefix
    Ok(hash)
}

/// Compute a content fingerprint for a .md file.
/// Extracts structured claims (entity names, positions, counts,
/// materials) from the markdown and hashes those — ignoring
/// prose that doesn't affect the .ron.
///
/// Returns None if the .md lacks a valid "## Entity Groups" section,
/// which causes the SyncRecord status to become Unknown rather than
/// silently hashing empty claims as Clean.
fn md_content_hash(path: &Path) -> Result<Option<String>, Error> {
    let content = std::fs::read_to_string(path)?;
    let claims = extract_structural_claims(&content)?; // Now returns Result
    if claims.is_empty() {
        return Ok(None); // No structural section → Unknown status
    }
    let hash = sha256_prefix(&serde_json::to_string(&claims)?, 8);
    Ok(Some(hash))
}

#[derive(Debug)]
enum ClaimExtractionError {
    /// The .md file has no "## Entity Groups" section.
    /// The LLM system prompt must enforce this convention.
    MissingEntityGroups,
    /// The section exists but contains unparseable content.
    MalformedEntityGroups(String),
}

/// Compute a fingerprint for the live scene state of a region.
/// Queries all entities with matching region_id, serializes their
/// transforms + materials + shapes, and hashes the result.
fn scene_region_hash(
    region_id: &str,
    registry: &NameRegistry,
    transforms: &Query<&Transform>,
    // ... other queries
) -> String {
    // Collect entities belonging to this region
    // Sort by name for deterministic ordering
    // Serialize to canonical JSON
    // Hash
}
```

---

## Drift Detection: The `gen_check_drift` Tool

A new MCP tool that compares all three representations and reports discrepancies:

### Tool Definition

```rust
ToolSchema {
    name: "gen_check_drift",
    description: "Compare .md files, .ron files, and the live Bevy scene \
        to detect inconsistencies. Returns a drift report showing which \
        domains are in sync, which have drifted, and what changed.",
    parameters: json!({
        "type": "object",
        "properties": {
            "domain": {
                "type": "string",
                "description": "Check a specific domain (e.g., 'regions/village-center'). \
                    Omit to check all domains."
            },
            "detail_level": {
                "type": "string",
                "enum": ["summary", "structural", "full"],
                "default": "structural",
                "description": "summary: just status per domain. \
                    structural: list entity-level differences. \
                    full: field-level diff."
            }
        }
    }),
}
```

### Drift Report Output

```json
{
  "overall_status": "drifted",
  "domains": [
    {
      "domain": "layout/blockout",
      "status": "clean",
      "md_hash": "x1y2z3w4",
      "ron_hash": "p5q6r7s8",
      "scene_hash": null,
      "last_sync": "2026-03-21T14:20:00Z"
    },
    {
      "domain": "regions/village-center",
      "status": "clean",
      "md_hash": "a1b2c3d4",
      "ron_hash": "e5f6g7h8",
      "scene_hash": "i9j0k1l2",
      "last_sync": "2026-03-21T14:28:00Z"
    },
    {
      "domain": "regions/forest-edge",
      "status": "md_ahead",
      "detail": "The .md describes 12 trees but the .ron only defines 8. \
          The .md mentions 'mushroom clusters at tree bases' which has \
          no corresponding entities in .ron.",
      "md_additions": [
        "4 additional trees mentioned in .md entity list",
        "mushroom cluster entities (new concept, not in .ron)"
      ],
      "suggestion": "Run gen_sync with source=md to generate missing .ron entities"
    },
    {
      "domain": "regions/lake-shore",
      "status": "scene_ahead",
      "detail": "Entity 'dock_post_3' was moved from [10,0,15] to [12,0,15] \
          in the live scene via gen_modify_entity. Neither .md nor .ron reflect this.",
      "scene_changes": [
        { "entity": "dock_post_3", "field": "position", "ron_value": [10,0,15], "scene_value": [12,0,15] },
        { "entity": "boat_small", "field": "rotation", "ron_value": [0,0,0,1], "scene_value": [0,0.38,0,0.92] }
      ],
      "suggestion": "Run gen_sync with source=scene to update .ron and .md"
    },
    {
      "domain": "behaviors/water-effects",
      "status": "conflict",
      "detail": "The .md says 'gentle_bob amplitude 0.15' but the .ron has \
          amplitude 0.25. The .ron was manually edited after the .md was \
          last generated.",
      "conflicts": [
        {
          "path": "behaviors.gentle_bob.amplitude",
          "md_value": 0.15,
          "ron_value": 0.25,
          "scene_value": 0.25
        }
      ],
      "suggestion": "Resolve conflict: pick .md value (0.15) or .ron value (0.25)"
    }
  ]
}
```

---

## Structural Claim Extraction from .md Files

The key challenge: `.md` files contain freeform prose mixed with structural facts. We need to extract the **structural claims** (things that should match the `.ron`) from the **design intent** (things that are `.md`-only context).

### What counts as a structural claim

| Claim Type | Example in .md | Corresponding .ron field |
|-----------|---------------|------------------------|
| Entity existence | "tavern — 2-story building" | `WorldEntity(name: "tavern")` |
| Entity position | "at `[3, 0, 0]`" | `transform.position: [3.0, 0.0, 0.0]` |
| Entity shape/size | "6x4x5m" | `shape: Cuboid(x:6, y:5, z:4)` |
| Entity material | "warm brown wood" | `material.color` (fuzzy match) |
| Entity count | "4x market stalls" | Count of entities matching "market_stall*" |
| Behavior reference | "with pulse behavior" | `behaviors: [Pulse(...)]` |
| Region bounds | "z: -5 to 10" | `bounds.center/size` |

### What is NOT a structural claim (md-only context)

- Design philosophy: "The heart of the village"
- Placement rationale: "Buildings face inward toward market square"
- Future intent: "Consider adding a well in Phase 2"
- Style guidance: "No neon colors"
- Regeneration notes

### Extraction Strategy

Rather than NLP-parsing freeform markdown (fragile), we use a **structured section convention** in `.md` files:

```markdown
# Village Center Region

## Design Intent              ← md-only, not compared
The heart of the village...

## Entity Groups              ← STRUCTURAL: compared against .ron

### Hero Structures (tier: hero)
- **tavern** — 2-story building at `[3, 0, 0]`, 6x4x5m, warm brown wood
- **blacksmith** — single story at `[-5, 0, 2]`, 4x3x3m, dark stone

### Medium Props (tier: medium)
- 4x market stalls along market square edges
- barrel clusters near tavern and blacksmith

## Placement Rules            ← md-only, not compared
Buildings face inward toward market square center...
```

The convention: **"Entity Groups" sections contain structural claims. Everything else is design intent.** The extractor parses this section using simple patterns:

```rust
struct StructuralClaim {
    entity_name: Option<String>,   // "tavern"
    count: Option<u32>,            // 4 (from "4x market stalls")
    position: Option<[f32; 3]>,    // [3, 0, 0] (from backtick-delimited)
    dimensions: Option<[f32; 3]>,  // [6, 4, 5] (from "6x4x5m")
    tier: Option<String>,          // "hero"
    material_hint: Option<String>, // "warm brown wood"
    behavior_hint: Option<String>, // "with pulse behavior"
}

fn extract_structural_claims(md_content: &str) -> Result<Vec<StructuralClaim>, ClaimExtractionError> {
    // 0. Look for "## Entity Groups" section — if absent, return
    //    Err(MissingEntityGroups) so the caller marks status as Unknown
    //    rather than silently treating the file as having zero claims.
    // 1. Find "## Entity Groups" section
    // 2. First, check for <!-- sync: {...} --> HTML comments (machine-readable,
    //    written by gen_sync). If present, parse claims from those — they are
    //    authoritative and avoid fuzzy prose parsing.
    // 3. Fallback: for each "- **name**" line, extract structured fields
    // 4. Parse backtick-delimited arrays as positions
    // 5. Parse "NxNxNm" as dimensions
    // 6. Parse "Nx <type>" as count + entity type
    // Return sorted by entity name for deterministic comparison
}
```

---

## Sync Metadata in .md Files

To prevent noise accumulation from repeated round-trip syncs (where literal values like `color [0.55, 0.35, 0.2]` clutter the human-readable prose), `.md` files use **HTML comment blocks** to store machine-readable sync data separately from design prose:

```markdown
### Hero Structures (tier: hero)
- **tavern** — 2-story building, warm brown wood
  <!-- sync: {"position":[3,0,0],"dimensions":[6,4,5],"color":[0.55,0.35,0.2,1.0]} -->
- **blacksmith** — single story, dark stone with chimney
  <!-- sync: {"position":[-5,0,2],"dimensions":[4,3,3],"color":[0.3,0.3,0.35,1.0]} -->
```

**Rules:**
1. `<!-- sync: {...} -->` comments are written by `gen_sync` when syncing `scene → md` or `ron → md`.
2. `extract_structural_claims` reads from these comments when present — they are authoritative and bypass fuzzy prose parsing.
3. If no sync comments exist (e.g., hand-written `.md`), the extractor falls back to parsing prose patterns (`[x,y,z]` in backticks, `NxNxNm` dimensions).
4. The human-readable prose ("warm brown wood") is preserved as-is and never overwritten by sync. Only the `<!-- sync: -->` block is updated.
5. Sync comments use compact JSON (no pretty-printing) to minimize visual noise.

This separation ensures `.md` files remain readable after many sync cycles while providing exact values for drift comparison.

---

## Reconciliation: The `gen_sync` Tool

Once drift is detected, the user (or LLM) reconciles using `gen_sync`:

### Tool Definition

```rust
ToolSchema {
    name: "gen_sync",
    description: "Reconcile drift between .md, .ron, and the live scene. \
        Specify which representation is the source of truth for this sync. \
        Preview mode shows what would change without applying.",
    parameters: json!({
        "type": "object",
        "properties": {
            "domain": {
                "type": "string",
                "description": "Domain to sync (e.g., 'regions/village-center'). \
                    Required."
            },
            "source": {
                "type": "string",
                "enum": ["md", "ron", "scene"],
                "description": "Which representation to treat as source of truth. \
                    md: regenerate .ron from .md, then update scene. \
                    ron: update scene from .ron, update .md to match. \
                    scene: snapshot scene to .ron, update .md to match."
            },
            "preview": {
                "type": "boolean",
                "default": true,
                "description": "If true, show what would change without applying. \
                    The Terraform 'plan' step."
            },
            "resolve_conflicts": {
                "type": "object",
                "description": "For conflict status: per-field resolution choices. \
                    Keys are conflict paths, values are 'md' or 'ron' or 'scene'. \
                    Fields listed here use their specified source. All other \
                    fields fall back to the top-level 'source' parameter.",
                "additionalProperties": { "type": "string" }
            }
        },
        "required": ["domain", "source"]
    }),
}
```

### Sync Directions

#### Direction 1: `.md` → `.ron` → Scene (source=md)

The LLM's design intent drives everything. Used when the LLM has updated the `.md` with new design decisions and wants to propagate them.

```
User or LLM edits .md
        │
        ▼
┌─────────────────────────────┐
│  gen_sync(source=md,         │
│           preview=true)      │
│                              │
│  1. Extract structural claims│
│     from .md                 │
│  2. Diff against .ron         │
│  3. Report:                  │
│     + 4 new entities         │
│     ~ 2 entities modified    │
│     - 1 entity removed       │
│                              │
│  "Apply these changes?"      │
└──────────────┬──────────────┘
               │ user confirms
               ▼
┌─────────────────────────────┐
│  gen_sync(source=md,         │
│           preview=false)     │
│                              │
│  1. LLM generates updated    │
│     .ron from .md claims     │
│  2. Diff new .ron vs old .ron│
│  3. Apply minimal changes    │
│     to live scene            │
│  4. Update meta/.sync.ron hashes  │
└─────────────────────────────┘
```

**Key detail:** The `.md` → `.ron` direction requires the LLM because `.md` contains semantic descriptions ("warm brown wood") that must be translated to concrete values (`color: [0.55, 0.35, 0.2, 1.0]`). The engine alone can't do this. So `gen_sync(source=md)` internally triggers an LLM generation pass for the affected entities.

#### Direction 2: `.ron` → Scene + `.md` (source=ron)

The `.ron` file is the source of truth. Used when the user has manually edited `.ron` and wants everything else to match.

```
User edits .ron
        │
        ▼
┌─────────────────────────────┐
│  gen_sync(source=ron,        │
│           preview=true)      │
│                              │
│  1. Parse new .ron            │
│  2. Diff against live scene  │
│  3. Diff against .md claims  │
│  4. Report:                  │
│     Scene: 3 entities moved  │
│     .md: entity count stale  │
│                              │
│  "Apply these changes?"      │
└──────────────┬──────────────┘
               │ user confirms
               ▼
┌─────────────────────────────┐
│  gen_sync(source=ron,        │
│           preview=false)     │
│                              │
│  1. Reload scene from .ron   │
│     (gen_unload_region +     │
│      gen_load_region)        │
│  2. LLM updates .md to       │
│     reflect .ron changes     │
│     (preserves design intent │
│      sections, updates       │
│      entity groups)          │
│  3. Update meta/.sync.ron hashes  │
└─────────────────────────────┘
```

#### Direction 3: Scene → `.ron` + `.md` (source=scene)

The live Bevy scene is the source of truth. Used after manual MCP tool edits (moving entities, tweaking materials) that should be persisted back.

```
User edits via MCP tools
        │
        ▼
┌─────────────────────────────┐
│  gen_sync(source=scene,      │
│           preview=true)      │
│                              │
│  1. Snapshot scene entities  │
│     for this region          │
│  2. Diff against .ron         │
│  3. Diff against .md claims  │
│  4. Report:                  │
│     .ron: 2 positions stale  │
│     .md: position text stale │
└──────────────┬──────────────┘
               │ user confirms
               ▼
┌─────────────────────────────┐
│  gen_sync(source=scene,      │
│           preview=false)     │
│                              │
│  1. Update .ron entities to  │
│     match scene state        │
│  2. LLM updates .md entity   │
│     groups to match          │
│  3. Update meta/.sync.ron hashes  │
└─────────────────────────────┘
```

### Conflict Resolution

When both `.md` and `.ron` have changed since last sync (status: `Conflict`), the tool requires explicit per-field resolution:

```json
// gen_check_drift returns:
{
  "domain": "behaviors/water-effects",
  "status": "conflict",
  "conflicts": [
    { "path": "gentle_bob.amplitude", "md_value": 0.15, "ron_value": 0.25 }
  ]
}

// User or LLM resolves:
gen_sync({
  "domain": "behaviors/water-effects",
  "source": "ron",
  "resolve_conflicts": {
    "gentle_bob.amplitude": "ron"    // Keep 0.25 from .ron
  }
})
```

**Mixed resolution example** — pick amplitude from `.ron` but position from `.md`:

```json
gen_sync({
  "domain": "behaviors/water-effects",
  "source": "md",
  "resolve_conflicts": {
    "gentle_bob.amplitude": "ron"
  }
})
// Result: amplitude=0.25 (from ron), all other fields from md (the source default)
```

If conflicts are unresolved, `gen_sync` refuses to proceed and lists them.

---

## The Flexible Workflow: User Personas

Different users interact with the system differently. The drift/sync tools support all of them:

### Persona 1: "Prompt-first creator" (LLM-heavy)

Primarily works through conversation. Rarely touches files directly.

**Workflow:**
1. Tells LLM "add a well to the village center"
2. LLM updates `regions/village-center.md` with well description
3. LLM calls `gen_sync(source=md, domain="regions/village-center")` to propagate
4. Engine generates `.ron` entities, spawns them in scene
5. LLM calls `gen_screenshot` to verify
6. If good → done. If not → LLM revises `.md` and re-syncs.

**Drift pattern:** `.md` is always ahead. Sync direction is always `md → ron → scene`.

### Persona 2: "Technical artist" (file editor)

Opens `.ron` files in a text editor. Tweaks positions, materials, colors by hand.

**Workflow:**
1. Opens `regions/village-center.ron` in editor
2. Adjusts tavern position from `[3,0,0]` to `[4,0,0]`
3. Saves file
4. File watcher detects change (or user calls `gen_check_drift`)
5. Calls `gen_sync(source=ron, domain="regions/village-center")`
6. Scene updates to match. `.md` gets tavern position updated.

**Drift pattern:** `.ron` is always ahead. Sync direction is always `ron → scene + md`.

### Persona 3: "Scene sculptor" (MCP tool user)

Uses MCP tools interactively to move, resize, recolor entities in the live scene.

**Workflow:**
1. Calls `gen_modify_entity("tavern", position: [5,0,0])`
2. Calls `gen_modify_entity("tavern", color: [0.6, 0.4, 0.25, 1.0])`
3. Happy with the result
4. Calls `gen_sync(source=scene, domain="regions/village-center")`
5. `.ron` and `.md` updated to match live scene.

**Drift pattern:** Scene is always ahead. Sync direction is always `scene → ron + md`.

### Persona 4: "Hybrid creator" (mixed editing)

Switches between all three modes. Most complex case.

**Workflow:**
1. Edits `.md` to add design notes about a new area
2. Uses MCP tools to place a few test entities
3. Manually tweaks `.ron` to fix a material
4. Calls `gen_check_drift` to see the full picture
5. Resolves each domain: some from scene, some from `.md`, some from `.ron`
6. Calls `gen_sync` per-domain with appropriate source

**Drift pattern:** Unpredictable. The `gen_check_drift` + per-domain `gen_sync` workflow handles this.

---

## Auto-Sync vs Manual Sync

### Option A: Manual sync (recommended for v1)

User explicitly calls `gen_check_drift` and `gen_sync`. No surprises.

Pros: Predictable, no accidental overwrites, works with any editor.
Cons: User must remember to sync.

### Option B: File-watcher auto-sync (future)

Uses `notify` crate to watch the skill directory. On file change:
1. Compute new hash
2. If hash differs from `meta/.sync.ron` → mark domain as drifted
3. Optionally auto-sync if user has configured a default source

Pros: Seamless editing experience.
Cons: Risk of overwriting concurrent edits, complexity.

### Option C: Pre-save drift check (recommended for v1)

When `gen_save_world` is called, automatically run `gen_check_drift` first. If any domain has unresolved drift, warn and ask for resolution before saving.

```
> gen_save_world(name: "medieval-village")

⚠ Drift detected before save:
  regions/forest-edge: scene_ahead (2 entities moved)
  behaviors/water-effects: conflict (amplitude: md=0.15, ron=0.25)

Resolve drift before saving, or use force=true to save scene state as-is.
```

### Recommendation

**v1: Manual sync + pre-save check.** The user calls `gen_check_drift` when they want a status report, `gen_sync` when they want to reconcile, and `gen_save_world` warns if there's unresolved drift. This is the Terraform model — explicit `plan` and `apply` steps.

**v2: File watcher notifications.** The engine watches the skill directory with `notify` and updates `meta/.sync.ron` hashes on file changes, but doesn't auto-sync. It surfaces drift status in the egui UI as a badge/indicator.

**v3: Smart auto-sync.** Based on user-configured default source per domain (e.g., "for regions, I edit .ron; for behaviors, I edit .md"), auto-sync on file save with undo support.

---

## Implementation Plan

### New Data Types

```rust
// crates/gen/src/gen3d/sync.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManifest {
    pub updated_at: String,
    pub domains: HashMap<String, SyncRecord>,
    pub root_ron_hash: Option<String>,
    pub root_scene_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub md_hash: String,
    pub ron_hash: String,
    pub scene_hash: Option<String>,  // None if scene not loaded
    pub md_mtime: String,
    pub ron_mtime: String,
    pub last_sync: String,
    pub sync_direction: SyncDirection,
    pub status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Clean,
    MdAhead,
    RonAhead,
    SceneAhead,
    Conflict,
    Unknown,  // Files exist but no previous sync record
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncDirection {
    MdToRon,
    RonToScene,
    SceneToRon,
    MdToRonToScene,  // Full forward pass
    SceneToRonToMd,  // Full backward pass
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub overall_status: SyncStatus,
    pub domains: Vec<DomainDrift>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDrift {
    pub domain: String,
    pub status: SyncStatus,
    pub detail: Option<String>,
    pub structural_diffs: Vec<StructuralDiff>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralDiff {
    pub entity: Option<String>,
    pub diff_type: DiffType,
    pub field: Option<String>,
    pub md_value: Option<serde_json::Value>,
    pub ron_value: Option<serde_json::Value>,
    pub scene_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffType {
    Added,      // Exists in source but not target
    Removed,    // Exists in target but not source
    Modified,   // Exists in both but values differ
}
```

### New MCP Tools

| Tool | Phase | Effort |
|------|-------|--------|
| `gen_check_drift` | P0 | 3 days — hash computation, diff engine, report formatting |
| `gen_sync` (preview mode) | P0 | 2 days — reuse diff engine, format plan output |
| `gen_sync` (apply, source=ron) | P0 | 2 days — reload region, update .md structural sections |
| `gen_sync` (apply, source=scene) | P0 | 2 days — snapshot scene to .ron, update .md |
| `gen_sync` (apply, source=md) | P0 | 3 days — requires LLM call to generate .ron from .md claims. **Elevated from P1:** the "prompt-first creator" persona (primary use case) depends entirely on this direction. Requires prompt engineering for semantic-to-concrete translation and fallback handling when the LLM is unavailable. |
| Conflict resolution UI | P1 | 2 days — per-field resolution in tool args |
| Pre-save drift warning | P1 | 1 day — hook into gen_save_world |
| File watcher integration | P2 | 3 days — notify crate, hash update, status resource |

**Total: ~21 days additional on top of parent RFC** (increased from 18 due to `source=md` elevation to P0).

**Combined effort with parent RFC: ~10.5 weeks.** Drift tools depend on parent Phase 2 (MCP tools), so implementation is sequential — drift work begins after parent Phase 2 delivers `gen_write_region` and `gen_load_region`.

### v1 World First-Load Behavior

When a world directory has no `meta/.sync.ron` manifest (e.g., a pre-existing v1 world or a freshly-created world before first sync):

1. All domains start with `status: Unknown` — drift detection cannot make claims about consistency without a baseline.
2. The first `gen_check_drift` call computes hashes from the current state of all `.md`, `.ron`, and scene representations.
3. It writes a new `meta/.sync.ron` with these hashes as the baseline. All domains become `Clean`.
4. **No sync is attempted** — only baseline establishment. The user must explicitly call `gen_sync` if they want to reconcile.
5. If only `.ron` files exist (no `.md` counterparts, as in v1 worlds), the `.md`-related fields in `SyncRecord` are set to `None` and those domains are marked `Clean` (there is no `.md` to drift against).

### Generation Log Integration

`gen_sync` operations are recorded in `meta/generation-log.jsonl` with `phase: "sync"`:

```jsonl
{"seq":42,"tool":"gen_check_drift","args":{"detail_level":"structural"},"result_hash":"abc123","phase":"sync"}
{"seq":43,"tool":"gen_sync","args":{"domain":"regions/village-center","source":"scene","preview":false},"result_hash":"def456","phase":"sync"}
```

This ensures replay can reproduce the full state history, not just initial generation. Sync operations that modify `.ron` files are semantically generation actions and must be tracked for deterministic reproduction.

### Integration with Parent RFC

The `gen_sync` tools compose naturally with the iterative generation loop:

```
Phase 0: Plan
  └─ gen_write_world_plan → creates .md + .ron + meta/.sync.ron (all Clean)

Phase N: Generate Region
  └─ gen_write_region → writes .md + .ron (Clean)
  └─ gen_load_region → spawns scene (Clean)
  └─ LLM evaluates, tweaks via MCP tools → scene drifts (SceneAhead)
  └─ gen_sync(source=scene) → .ron + .md updated (Clean)
  └─ Next iteration...

Final: Save
  └─ gen_check_drift → verify all Clean
  └─ gen_save_world → writes everything to disk including meta/
```

---

## Acceptance Criteria

1. **All five edit paths detected:** `gen_check_drift` correctly identifies drift from each of the five edit paths in the drift table (LLM edits .md, LLM calls MCP tools, user edits .ron, user edits .md, LLM writes .ron).
2. **Preview accuracy:** `gen_sync(preview=true)` output matches the actual changes applied by `gen_sync(preview=false)` — no surprises between plan and apply.
3. **Mixed conflict resolution:** `gen_sync` with `resolve_conflicts` correctly applies per-field source overrides while using the top-level `source` as default for unmentioned fields.
4. **v1 world baseline:** First `gen_check_drift` on a v1 world (no `meta/.sync.ron`) creates a baseline manifest with all domains `Clean` and does not modify any `.md` or `.ron` files.
5. **Round-trip preservation:** A full round-trip sync (`md → ron → scene → ron → md`) preserves all structural claims. The `<!-- sync: -->` comments update but the human-readable prose is unchanged.
6. **Silent failure prevention:** A `.md` file missing the `## Entity Groups` section causes `gen_check_drift` to report `Unknown` status for that domain — never `Clean`.
7. **Generation log completeness:** All `gen_sync` and `gen_check_drift` calls appear in `meta/generation-log.jsonl` with `phase: "sync"`.

---

## Open Questions

1. **Fuzzy matching for .md material descriptions.** "warm brown wood" should match `color: [0.55, 0.35, 0.2, 1.0]` but how fuzzy is too fuzzy? Recommendation: don't try to reverse-map colors to English. Instead, when syncing `scene → md`, write the literal values in the .md: "color `[0.55, 0.35, 0.2]` (warm brown wood)". The prose is preserved as a parenthetical.

2. **Entity identity across representations.** Entities are matched by `name` across .md, .ron, and scene. What happens if an entity is renamed in one representation? Recommendation: treat rename as delete + add. The old name disappears, the new one appears. `gen_check_drift` reports this as "-old_name, +new_name" and asks the user to confirm.

3. **Partial region sync.** Can you sync just one entity within a region, or must the whole region sync at once? Recommendation: v1 syncs whole regions. v2 adds entity-level granularity with `gen_sync(domain="regions/village-center", entity="tavern")`.

4. **Sync across renames/moves.** If a user moves entities between regions (e.g., moves the well from village-center to market-square), the drift report shows a removal in one region and an addition in another. Recommendation: accept this as the expected behavior. Cross-region moves are structural changes that the user should intentionally make in the .md, then sync forward.

5. **Performance of scene hashing.** For large worlds (500+ entities), computing scene hashes on every `gen_check_drift` call could be slow. Recommendation: use a custom `RegionDirtyFlags` Bevy resource (not raw `Changed<T>`, which resets every frame). A dedicated system runs each frame, latching dirty flags on `Changed<Transform> | Changed<Handle<StandardMaterial>> | Changed<Visibility>` for entities with a `RegionMember` component. Flags are only cleared after `gen_check_drift` or `gen_sync` successfully computes hashes. This avoids the common Bevy footgun where `Changed<T>` is only valid for two frames.

   ```rust
   #[derive(Resource, Default)]
   pub struct RegionDirtyFlags {
       pub dirty: HashSet<String>,  // region_id values
   }

   fn track_region_dirty(
       mut flags: ResMut<RegionDirtyFlags>,
       changed: Query<&RegionMember, Or<(Changed<Transform>, Changed<Handle<StandardMaterial>>, Changed<Visibility>)>>,
   ) {
       for member in &changed {
           flags.dirty.insert(member.region_id.clone());
       }
   }
   // flags.dirty.remove(&region_id) called only after hash computation in gen_check_drift/gen_sync
   ```
