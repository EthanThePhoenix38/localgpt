# World packaging formats for AI-driven 3D creation

The landscape of 3D world packaging formats reveals a clear convergence: **the most AI-friendly, git-compatible, and future-proof formats are text-based, declarative, and use JSON or Markdown with YAML frontmatter as their core serialization**. For LocalGPT Gen, the strongest design influences come from OpenClaw's SKILL.md pattern (folder-as-package with Markdown+YAML manifests), glTF's extensible JSON scene graph with emerging interactivity and audio extensions, ComfyUI's workflow-as-recipe JSON embedding, and A-Frame's HTML-attribute ECS model. This report maps every relevant format and distills the patterns that matter most.

---

## OpenClaw's SKILL.md pattern sets the standard for AI-native packaging

OpenClaw (118k GitHub stars, npm package) has built the most mature folder-as-package format for AI agent skills. Every skill is **a directory whose name becomes its identifier**, with a single required file: `SKILL.md` — a Markdown document with YAML frontmatter. This format, following the AgentSkills.io spec, is worth studying closely for LocalGPT Gen's world manifest design.

The standard skill folder layout is:

```
skill-name/
├── SKILL.md              # Required — metadata + instructions
├── scripts/              # Executable helper code
├── references/           # Documentation, examples
├── assets/               # Templates, images, files
├── install.sh            # Optional install script
└── config.json           # Optional config schema
```

The SKILL.md frontmatter declares everything: `name`, `description`, `version` (semver), and a `metadata` JSON object encoding requirements (`requires.env`, `requires.bins`, `requires.config`), platform constraints (`os`), installer specs, and UI hints. The Markdown body contains natural-language instructions — the actual "recipe" for an AI agent to follow. **Dependencies are declared in frontmatter metadata, not in a separate package.json**, keeping the format self-contained.

Three critical design decisions make this format excellent for AI tooling. First, the **text-only constraint**: OpenClaw's publish pipeline only accepts text-based file extensions (defined in a `textFiles.ts` allowlist), ensuring every skill is diffable, grep-able, and version-controllable. Second, **runtime path references** use `{baseDir}` tokens in instructions, enabling portable asset references without hardcoded paths. Third, **three-tier loading precedence** — workspace skills override managed skills override bundled skills — lets users customize without forking. The ClawHub registry (5,700+ skills) uses content-hash comparison for update detection, with a `.clawhub/lock.json` lockfile tracking installed versions.

For LocalGPT Gen, this pattern suggests: a `WORLD.md` or `world.yaml` manifest with YAML/JSON metadata, natural-language generation instructions in the body, and a folder structure for assets, scripts, and references — all text-based where possible, with binary assets (models, audio) referenced by path rather than embedded.

---

## Interactive 3D formats split into binary engines and web-native text

The ten formats surveyed fall into two clear camps. **Binary-first engine formats** (Unity AssetBundle, Godot .pck, VRChat VRCA, Roblox RBXL) are optimized for runtime performance but hostile to version control and AI generation. **Text-first web formats** (A-Frame HTML, PlayCanvas JSON+JS, Decentraland scene.json+TypeScript, glTF with extensions) are human-readable, git-friendly, and highly suitable for AI generation. USD straddles both worlds with its ASCII (.usda) and binary crate (.usdc) variants.

**A-Frame** represents the simplest possible scene packaging: pure HTML with custom elements. An entire 3D scene is a DOM tree where `<a-entity>` elements carry behavior through HTML attributes (`geometry="primitive: sphere"`, `sound="src: #bgm; loop: true"`, `animation="property: rotation; to: 0 360 0"`). This is the most AI-generatable format surveyed — LLMs already excel at producing HTML. The ECS architecture maps cleanly: entities are elements, components are attributes, systems are registered JavaScript functions.

**Decentraland** offers the best-documented manifest-based approach. Its `scene.json` declares parcels, spawn points, display metadata, and an entry point (`main: "bin/game.js"`). Scene code is TypeScript using an ECS SDK, compiled to JavaScript and run in isolated WebWorkers. Assets (glTF models, MP3 audio) sit in the project directory and are referenced by relative path. The full project structure — `scene.json`, `src/`, `models/`, `sounds/`, `images/` — is entirely text-based and git-friendly before compilation.

**glTF with KHR_interactivity** is the most significant emerging standard. Released as a draft in June 2024, this extension embeds **behavior graphs** (visual scripting nodes) directly in the glTF JSON. The structure defines `types`, `variables`, `events`, and `nodes` with flow sockets (execution order) and value sockets (data). Node types include lifecycle events (`onStart`, `onTick`), animation control, flow control, math/logic, and JSON Pointer-based access to any glTF property. The design is explicitly sandboxed — no arbitrary code execution — making it safe for user-generated content. Google Android XR, UnityGLTF, and Needle Engine are implementing support.

**USD's ASCII format (.usda)** deserves special mention for its composition architecture. USD supports non-destructive layering via **LIVRPS** (Local, Inherits, Variants, References, Payloads, Specializes) — stronger layers override weaker ones without modifying originals. This enables collaborative, incremental world-building where AI-generated content can be layered on top of hand-authored bases. The `.usdz` package format is an uncompressed ZIP (for zero-copy mmap access) containing a root USD file plus assets.

**Roblox** is notable for offering both binary (RBXL) and XML (RBXLX) variants of the same format. The binary format uses a chunk-based structure (INST, PROP, PRNT, META, SSTR, END chunks) with LZ4 compression, byte interleaving, and zigzag encoding. The XML variant is fully human-readable. **Luau scripts are embedded as string properties** within script instances — the entire behavior layer lives inside the scene graph, not in separate files.

| Format | Encoding | Human-readable | Git-friendly | Logic support | AI-gen suitability |
|--------|----------|:---:|:---:|---|:---:|
| Unity AssetBundle | Binary (UnityFS) | ✗ | ✗ | References only | Very low |
| Godot .pck | Binary container, text contents | Partial | Source: ✓ | GDScript text | High (source) |
| PlayCanvas | HTML + JSON + JS | ✓ | ✓ | JavaScript | High |
| A-Frame | HTML + JS | ✓ | ✓ | JS components | Very high |
| Decentraland | JSON + TypeScript + glTF | ✓ | ✓ | TypeScript ECS | High |
| Mozilla Hubs | glTF + extensions | Partial | Partial | Hubs extensions | Moderate |
| VRChat | Binary (encrypted UnityFS) | ✗ | ✗ | Udon bytecode | Very low |
| Roblox | Binary or XML | RBXLX: ✓ | RBXLX: ✓ | Luau embedded | Moderate-high |
| USD | Text (.usda) or binary (.usdc) | USDA: ✓ | USDA: ✓ | None native | High (USDA) |
| glTF + KHR_interactivity | JSON in glTF | ✓ | ✓ | Behavior graphs | High |

---

## Recording deterministic recipes: from node graphs to tool-call logs

For LocalGPT Gen to support reproducible world generation, the most relevant models are **ComfyUI's workflow JSON** (complete generation recipe embedded in output), **Houdini's procedural HDA** (parametric node graph as the artifact itself), and **Substance Designer's XML graphs** (deterministic material recipes). Each demonstrates a different philosophy of capturing "how something was made."

**ComfyUI** uses two JSON representations. The **workflow JSON** (LiteGraph format) captures the full visual editor state — nodes with IDs, types, positions, widget values, plus links as `[linkId, sourceNode, sourceSlot, targetNode, targetSlot, type]` tuples. The **API JSON** strips visual metadata, embedding connections directly in input values as `[sourceNodeId, outputIndex]` references. Crucially, ComfyUI **embeds the complete workflow JSON into PNG metadata** (tEXt chunks), so any generated image can be dragged back into the editor to recreate its exact generation pipeline. Given the same model checkpoint, seed, and parameters, output is deterministic. This is the closest existing model to "embed the recipe in the artifact."

**Houdini HDAs** represent the purest procedural-recipe format. An HDA packages an entire node graph — SOPs, parameters, embedded scripts, VEX code — into a single distributable file. Using the `hotl -x` command, HDAs expand to **plain-text directory structures** with `.init` files, `DialogScript` (parameter definitions), `contents.dir/` (node definitions), and embedded Python/VEX modules. Houdini is **deterministic by design**: same inputs + same parameters = identical output, always. The `namespace::name::version` naming convention supports non-breaking versioning. Houdini Engine enables headless execution in Unity, Unreal, or standalone — making HDAs viable as portable generation recipes.

**Substance Designer .sbs files** are XML containing the complete material graph: nodes with type identifiers, unique IDs, input/output connections (by UID), typed parameter values, and GUI layout data. Dependencies use an alias system (`sbs://` paths) for cross-machine portability. The pysbs Python API (Substance Automation Toolkit) enables programmatic graph construction and modification. Being XML, .sbs files are diffable, mergeable, and parseable by pipeline tools.

**Terraform's model** is relevant for a different reason: it demonstrates **declarative state management with deterministic diff-and-apply**. The state file (JSON) maps declared resources to real-world objects, and the plan computes the minimal diff between desired and actual state. The two-step plan-then-apply workflow, with the plan itself being a reviewable artifact, mirrors what LocalGPT Gen might need: generate a plan (tool calls), review it, then apply it to produce a world.

**Blender** provides a cautionary counterexample. The .blend file stores **final state, not operation history** — the undo stack exists only in RAM and is lost on close. Reproducibility requires external Python scripts using `bpy.ops.*` calls. This is viable (Blender's Python API is comprehensive) but demonstrates that state-only formats lose the recipe.

**Game replay systems** reveal two fundamental approaches. **Input recording** (StarCraft, Warcraft III) stores only initial state plus timestamped inputs, requiring bit-perfect engine determinism — fragile to any code change, but producing tiny files (kilobytes for full games). **State snapshot recording** (Unreal's DemoNetDriver) periodically serializes full world state, enabling seeking and scrubbing without strict determinism, at the cost of much larger files. For LocalGPT Gen, the **tool-call recording approach** is analogous to input recording: store the sequence of AI operations (create terrain, place object, set material) rather than the final state, enabling replay and modification.

---

## Spatial audio converges on Web Audio API parameters

Every spatial audio system surveyed — glTF extensions, Three.js, Resonance Audio, Decentraland, Mozilla Hubs — converges on the same core parameter set derived from the **Web Audio API PannerNode**: `distanceModel` (linear/inverse/exponential), `refDistance`, `maxDistance`, `rolloffFactor`, `coneInnerAngle`, `coneOuterAngle`, `coneOuterGain`, `gain`, `loop`, and `autoPlay`. This consensus makes defining an audio schema for LocalGPT Gen straightforward.

The **KHR_audio** glTF extension (evolving from Microsoft's MSFT_audio_emitter through the Open Metaverse group's OMI_audio_emitter) introduces the cleanest three-tier architecture: **audio data** (file references via URI or bufferView), **sources** (playback configuration: gain, loop, autoPlay), and **emitters** (spatial configuration: type, distance model, cone). Emitters attach to glTF nodes (positional) or scenes (global/ambient). This separation enables reuse — one audio clip can feed multiple sources with different playback settings, and one source can drive multiple emitters at different positions.

A newer **KHR_audio_graph** proposal (PR #2421 on the Khronos glTF repo) extends this into a full audio routing graph with typed nodes (source, gain, filter, emitter), enabling complex audio pipelines. It's designed to integrate with KHR_interactivity for behavior-driven audio control.

**Mozilla Hubs** adds a unique concept not found in other formats: **Audio Zones** — 3D box volumes that modify audio properties based on source/listener position. An `inOut` zone attenuates audio from inside sources for outside listeners; an `outIn` zone does the reverse. This enables room isolation and private conversation areas. Hubs also separates **avatar speech** and **media audio** parameters at the scene level, with independent distance models and rolloff for each.

**Resonance Audio** (Google, open source) contributes the most sophisticated room acoustics model. It uses **Higher-Order Ambisonics** to project all sources into a global soundfield, then applies HRTFs once — enabling hundreds of simultaneous spatialized sources even on mobile. Its room definition uses string-based material presets (`"brick-bare"`, `"curtain-heavy"`, `"marble"`, `"glass-thin"`, `"grass"`) applied to six room surfaces, plus room dimensions. These presets are highly AI-generatable — an LLM can describe a room's acoustic character through material names without understanding acoustics math. Resonance Audio has no standalone scene file format (it's a runtime API), but its room-materials model could be serialized as JSON.

For LocalGPT Gen, the recommended audio schema combines KHR_audio's three-tier structure (data → source → emitter) with Resonance Audio's room acoustics model (dimensions + material presets) and Hubs' zone concept. Audio files should be referenced by URI (not embedded), enabling git-friendly text manifests with separate binary audio assets.

---

## Encoding object behaviors without shipping a scripting engine

The most promising approaches for encoding smart object behaviors in a packaged world format — ranked by AI-generatability and declarative power — are **state machines (XState JSON)**, **behavior trees (BehaviorTree.CPP XML)**, **ECS component composition (A-Frame HTML attributes)**, and **glTF behavior graphs (KHR_interactivity)**. Each represents a different point on the declarative-imperative spectrum.

**XState-style state machines** are the most natural fit for interactive objects with distinct states. A door, a treasure chest, an NPC conversation — all map cleanly to states, transitions, guards, and actions in pure JSON:

```json
{
  "id": "door", "initial": "closed",
  "states": {
    "closed": { "on": { "INTERACT": { "target": "open", "guard": "hasKey" }}},
    "open": { "entry": "playOpenAnimation", "on": { "INTERACT": "closed" }}
  }
}
```

This format is extremely AI-friendly (structured JSON with a well-defined schema), git-friendly (pure text), and requires no scripting engine — just a lightweight state machine interpreter. The Stately visual editor can import/export these as JSON, and the actor model supports multiple machines communicating via events.

**Behavior trees** (BehaviorTree.CPP format) provide richer sequential and conditional logic in XML. The format defines Sequence, Fallback, Parallel, Decorator, Action, and Condition nodes with typed ports and a blackboard for data sharing. Subtrees enable modular composition (`<SubTree ID="patrol_route"/>`), and file includes support multi-file organization. The XML is highly structured and AI-generatable, with the `TreeNodeModel` section declaring node schemas for tooling validation.

**A-Frame's ECS-via-HTML-attributes** represents the most web-native approach. Components ARE behaviors, declared as HTML attributes with CSS-like property syntax (`animation="property: rotation; to: 0 360 0; dur: 3000; loop: true"`). Custom components register schemas defining property types and defaults, with lifecycle hooks (init, tick, update, remove). The HTML itself is the serialization format — no separate build step required. For AI generation, this is arguably the easiest format: LLMs produce high-quality HTML with near-zero training.

**glTF KHR_interactivity behavior graphs** offer the most portable solution — behaviors embedded directly in the 3D asset file, executable by any compliant viewer. The JSON structure defines nodes with flow sockets (execution order) and value sockets (data), covering lifecycle events, animation control, math/logic, and property access via JSON Pointers. The explicit sandboxing (no arbitrary code, bounded execution) makes this safe for user-generated content distribution.

For complex behaviors beyond what declarative formats can express, **WASM sandboxing** provides the escape hatch. WebAssembly modules run in isolated linear memory with deny-by-default capabilities, achieving ~85-95% native performance. Decentraland explored WASM via WASI (though their current SDK uses TypeScript in WebWorkers), and Hyperfy uses PhysX compiled to WASM for physics simulation. The packaging pattern is consistent: a JSON manifest references the WASM module alongside glTF assets, with an explicit import/export interface defining available APIs.

The strongest design for LocalGPT Gen would be a **layered behavior system**: declarative components (ECS attributes) for simple properties, state machines (JSON) for interactive objects, behavior trees (XML/JSON) for complex sequences, and optional WASM modules for unlimited scripting — with the format clearly indicating which layer each behavior uses.

---

## Conclusion: design principles for LocalGPT Gen's world format

Seven concrete principles emerge from this research. **First, folder-as-package with a Markdown/YAML manifest** (OpenClaw's model) is the most AI-native, git-friendly, and human-readable approach — a `WORLD.md` with YAML frontmatter for metadata and Markdown for generation instructions. **Second, separate scene graph from assets from behavior from audio** into distinct, independently versionable layers, following glTF's extensibility model and KHR_audio's three-tier architecture. **Third, embed the generation recipe alongside the result**, inspired by ComfyUI's workflow-in-PNG and Houdini's procedural-graph-as-artifact philosophy — every world should carry its complete tool-call history for reproducibility. **Fourth, use JSON for structured data and HTML-attribute-style declarations for components**, maximizing AI generatability across the entire format. **Fifth, adopt Web Audio API parameter names as the universal audio vocabulary**, since every platform already speaks this language. **Sixth, implement a layered behavior system** spanning declarative components through state machines through behavior trees to optional WASM, with each layer clearly delineated. **Seventh, keep binary assets external and referenced by path**, ensuring the manifest and scene description remain text-based, diffable, and mergeable — following OpenClaw's text-only-in-version-control discipline.

The formats that should most directly influence LocalGPT Gen's design are OpenClaw SKILL.md (manifest pattern), glTF + KHR_interactivity + KHR_audio (scene graph + behavior + audio in extensible JSON), ComfyUI workflow JSON (recipe embedding), A-Frame ECS (declarative component model), and USD composition arcs (non-destructive layering for collaborative editing). The binary-first engine formats (Unity, VRChat, Godot .pck) serve primarily as counterexamples of what to avoid for an AI-first tool.