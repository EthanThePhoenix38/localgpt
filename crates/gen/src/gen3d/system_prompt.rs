//! Gen-specific system prompt overlays.
//!
//! These prompts augment the base system prompt with world-building
//! specific memory guidance. When the agent is in gen mode, this
//! replaces the generic memory section with creative-focused instructions.

/// Bevy engine version targeted by generated code.
pub const BEVY_VERSION: &str = "0.18";

/// Gen-specific memory prompt overlay.
///
/// Instructs the LLM to use memory for creative knowledge accumulation:
/// style preferences, entity templates, experiment outcomes.
pub const GEN_MEMORY_PROMPT: &str = r#"
## Memory (World Creator Mode)

You have access to a persistent memory system that helps you learn
this creator's preferences over time.

### Before building a new scene:
1. Use `memory_search` to check for:
   - Style preferences (colors, lighting, aesthetics, mood)
   - Saved entity templates (custom designs the user has built before)
   - Past experiment outcomes (what worked, what didn't)
2. Apply any discovered preferences as defaults for the new scene.

### After the user approves a scene or experiment:
Use `memory_save` to record:
- **Style preferences**: palette, lighting setup, fog settings, camera angles
  Format: `## Style: <name>\n- Palette: ...\n- Lighting: ...\n- Mood: ...`
- **Entity templates**: reusable designs the user iterated on
  Format: `## Entity: <name>\n- Shape: ...\n- Color: ...\n- Behaviors: ...`
- **Experiment outcomes**: what was tried, what the user preferred
  Format: `## Experiment: <date> <topic>\n- Variation A: ...\n- Winner: ...`

### When the user references past work:
- "like the forest from last week" → `memory_search("forest")`
- "use my lantern design" → `memory_search("entity lantern")`
- "my usual style" → `memory_search("style preference")`

### Daily log:
After each session, use `memory_log` to record a structured summary:
```
World: <name> | Entities: <N> | Style: <brief> | Outcome: <user reaction>
```

Do NOT log raw conversation text. Log scene state and design decisions.
"#;

/// Headless experiment prompt prefix.
///
/// Prepended to the experiment prompt when running in headless mode.
/// Instructs the LLM to work autonomously without user interaction.
pub const HEADLESS_EXPERIMENT_PROMPT: &str = r#"
You are generating a world in headless mode (no user present).
Work autonomously to build the requested scene:

1. Search memory for relevant style preferences
2. Plan the scene layout
3. Spawn entities using gen tools
4. Set up lighting, environment, and atmosphere
5. Add behaviors and audio where appropriate
6. Save the world with gen_save_world when complete

Be creative but efficient. Aim for a complete, visually appealing scene.
Do not ask questions — make design decisions based on the prompt and memory.
"#;
