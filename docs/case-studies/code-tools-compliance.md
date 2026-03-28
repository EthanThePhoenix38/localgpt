# Why Code-Output AI Tools Are Exempt: The Complete Compliance Case

**Last updated:** 2026-03-28

## Executive Summary

AI tools used in game development fall into two fundamentally different categories based on their output format, and regulators, platforms, and the industry have converged on treating them differently:

- **Code tools** produce source code (text files, scene definitions, configuration) that a human developer compiles, reviews, and modifies. The output is developer-facing. Examples: code assistants, LocalGPT Gen (RON scene definitions), debugging tools.
- **Content tools** produce binary assets (images, textures, 3D meshes, audio, video) that end up directly in the shipped product as consumer-facing content. Examples: image generators, AI texture tools, neural rendering systems.

This distinction now has formal backing at every level that matters for game developers: platform policy (Steam/Valve, January 2026), regulation (EU AI Act, Regulation 2024/1689), publisher policy (Capcom, February 2026), cultural movement (the "No Gen AI" seal), and industry sentiment (GDC 2026 developer surveys). Code-output tools face zero disclosure requirements, zero labeling obligations, zero award disqualification risk, and zero consumer backlash -- because their output never reaches consumers.

This document consolidates findings from 28 research runs (March 23-27, 2026) tracking compliance, branding, and industry developments relevant to code-output AI tools in game development.

---

## 1. Steam / Valve

### January 2026 Policy Rewrite

In January 2026, Valve rewrote its AI disclosure requirements for games published on Steam. The key change was an explicit carve-out for code tools. The updated disclosure form states:

> "Efficiency gains through the use of AI powered dev tools is not the focus of this section."

Under the rewritten rules:

- **AI tools used to help build games** (code assistants, debugging software, development automation) are explicitly exempt from disclosure requirements.
- **AI that creates content players see, hear, or interact with** (generated art, music, textures, voice acting) requires disclosure labeling on the game's Steam store page.

The policy draws the line at consumer visibility: if the AI output is compiled or transformed by the developer before it reaches the player, no disclosure is needed. If the AI output is the thing the player experiences, disclosure is mandatory.

### Scale of Impact

As of early 2026, approximately 8,000 Steam games released in 2025 disclosed generative AI usage -- a 700% year-over-year increase from roughly 1,000 disclosures in 2024. By 2026, one-third of new Steam releases carry AI content disclosure tags. 90% of surveyed developers support even stricter disclosure requirements. Every one of these disclosure obligations applies to AI-generated content, not to code tools.

### Valve's Own Practice: Spring Sale 2026

Valve's behavior as a company reinforces its policy distinction. For the Steam Spring Sale 2026 (March 19-26), Valve commissioned human artists -- Tiffany Diep and animator thanhuki -- to create all sale artwork. Valve's official Steam X account explicitly credited these artists by name. Tiffany Diep handles all of Steam's 2026 seasonal sale banners and point shop assets.

Community response was immediate and positive: "Thank you for using real artists and not slop" was representative of the engagement, with one Reddit post generating 17,000+ interactions.

This contrasts with GOG, which faced backlash in January 2026 for promoting its New Year's sale with AI-generated banner art and was forced to issue an apology.

Valve's dual stance is the clearest platform-level signal in the industry:
- **For content**: commission human artists, credit them publicly, celebrate their work.
- **For development tools**: exempt code tools from disclosure entirely.

The largest PC gaming distribution platform both enforces a content/code distinction in policy and models it in practice.

### Steam Next Fest and the Coming AI Content Filters

Steam Next Fest (February 23 - March 2, 2026) brought the content/code distinction into sharp relief. PCGamesN's manual analysis of the top 100 curated demos found 10% explicitly disclosed generative AI usage. With nearly 4,000 demos available, the event was widely described as flooded with "AI slop" -- low-effort games with AI-generated art and text. Kotaku headlined: "Steam Next Fest Players And Devs Navigate Flood Of GenAI Junk."

The community's primary demand: a filter to exclude AI-generated content from Steam browsing. No such filter exists today, but the pressure is intense and growing. When Steam adds content-based AI filtering -- which the Next Fest backlash makes likely -- games built with code tools will be invisible to those filters, because they contain no AI-generated content to filter.

An additional consequence of the Next Fest incident: legitimate indie developers with stylized art reported being falsely accused of using AI. Code tools eliminate this false-accusation risk entirely -- the AI contribution (code structure) is invisible to players.

---

## 2. EU AI Act (Regulation 2024/1689)

### Classification Framework

The EU AI Act (Regulation 2024/1689) establishes a risk-based classification system for AI systems. The classifications relevant to game development:

- **Minimal risk**: AI systems with negligible risk, subject only to transparency obligations. No conformity assessment required. No mandatory registration.
- **Limited risk**: AI systems with specific transparency obligations (Article 50). Must inform users they are interacting with AI.
- **High risk**: AI systems in sensitive domains (healthcare, law enforcement, critical infrastructure). Require conformity assessment, registration, monitoring.
- **Unacceptable risk**: Banned AI practices (social scoring, real-time biometric surveillance).

**AI-enabled video games are classified as minimal-risk AI systems with no specific mandates.** Open-source AI tools released under free licenses receive explicit exemptions from provider obligations. For a tool like LocalGPT Gen (Apache 2.0, local-first, code output), this means triple insulation from EU regulation: minimal-risk classification, open-source exemption, and code-output (not consumer-facing content).

### Article 50 Transparency Obligations

Article 50 of the EU AI Act addresses transparency for AI-generated content. The obligation targets AI systems that generate synthetic content -- text, images, audio, or video -- that could be mistaken for authentic content by a reasonable person. Source code is not content that could be mistaken for authentic human-created media. The obligation therefore does not apply to code-generation tools.

### Code of Practice: Two-Layered Marking

The European Commission is developing a Code of Practice on marking and labeling AI-generated content to operationalize Article 50. The timeline:

| Date | Milestone |
|------|-----------|
| December 17, 2025 | First draft published |
| Mid-March 2026 | Second draft published |
| March 30, 2026 | Feedback deadline on second draft |
| June 2026 | Finalization |
| August 2, 2026 | Enforcement begins |

The second draft (March 2026) introduces a **two-layered marking approach**:

1. **Secured metadata**: Machine-readable metadata embedded in AI-generated content files indicating AI provenance.
2. **Watermarking**: Visual or auditory watermarks in AI-generated content (images, video, audio) indicating AI origin.

The draft also includes illustrative examples of a potential EU icon for AI content labeling and optional fingerprinting techniques.

**What this means for content tools**: AI image generators, texture generators, audio generators, and video generators will need to embed both machine-readable metadata AND perceptible watermarks in their output by August 2, 2026. Developers using these tools will ship games containing marked content.

**What this means for code tools**: Nothing. Source code is not "synthetic content that could be mistaken for authentic content." Code-generation tools require zero marking, zero metadata embedding, and zero watermarking. RON scene definitions compiled by Bevy are developer artifacts, not consumer-facing synthetic media.

The contrast is stark: content tools face a two-layered marking obligation with enforcement four months away. Code tools face zero marking obligations.

### Penalties

Non-compliance with the EU AI Act carries penalties of up to 35 million EUR or 7% of global annual turnover, whichever is higher. While these penalties primarily target high-risk AI system violations, the content labeling requirements under Article 50 also carry enforcement mechanisms. Developers shipping games with unlabeled AI-generated content face regulatory risk. Developers using code tools face none.

---

## 3. The "No Gen AI" Seal Movement

### Origins and Spread

Indie studio Polygon Treehouse created a golden cog-shaped "No Gen AI" seal -- inspired by the Nintendo Seal of Quality -- that developers can freely add to their Steam store pages. Games including Rosewater, Astral Ascent, and Quarterstaff now display it. The seal operates on an honor system with no formal certification process.

The movement represents a cultural shift: visible rejection of AI-generated game content as a positive marketing differentiator, not just a defensive posture.

### Award Disqualifications

The cultural movement has teeth. The Indie Game Awards stripped Clair Obscur: Expedition 33 of its Game of the Year and Debut Game awards after discovering undisclosed generative AI usage, stating: "We have a hard stance against gen AI in videogames." Blue Prince received the Game of the Year award after explicitly confirming no AI was used in its development.

Award competitions now operate as a two-step evaluation: quality AND AI certification. The "No Gen AI" confirmation has become a competitive advantage for awards eligibility.

### Publisher Bans

In February 2026, Capcom officially stated: "We will not incorporate assets generated by AI into our game content." However, they intend to "actively utilize such technologies to enhance efficiency and boost productivity within game development processes" across graphics, sound, and programming.

Capcom's policy draws the identical line as Steam and the EU AI Act: AI-generated content in the shipped product is banned; AI development tools are embraced. This is corporate policy at one of the world's largest game publishers (Monster Hunter, Resident Evil, Street Fighter).

Following the DLSS 5 controversy (see Section 4), Capcom reiterated that no generative AI will be used for final in-game assets.

Larian Studios (Baldur's Gate 3) followed a similar trajectory. After initial experimentation, CEO Swen Vincke announced in a January 2026 Reddit AMA that Larian has "decided to refrain from using genAI tools during concept art development" so there is no doubt everything in Divinity is original artwork. The writing director confirmed AI-generated text scored "3/10 at best." Larian still allows internal AI tools for non-creative workflows.

### Why Code Output Is Compatible

The "No Gen AI" seal targets AI-generated content -- art, music, assets, text that appears in the final product. A game built with a code-output AI tool can legitimately display the seal because:

1. The AI output is source code (RON scene definitions, Rust structs), not consumer-facing content.
2. A human developer compiles, reviews, and potentially modifies the code.
3. The final creative artifacts (geometry, materials, lighting) are produced by the Bevy engine at compile/runtime, not by the AI.
4. The game contains no AI-generated art, music, textures, or voice acting.

The creative direction is human-driven. The AI assists with the structural scaffolding (scene layout, entity placement, behavior wiring) that the developer would otherwise write by hand.

---

## 4. Industry Signals

### GDC 2026 Developer Survey Data

The annual GDC State of the Industry survey (2,300 developers) reveals the code-vs-content acceptance gap in quantified terms:

| Metric | Value | Source |
|--------|-------|--------|
| Developers viewing gen AI as bad for industry | 52% (up from 18% in 2024, 30% in 2025) | GDC 2026 |
| Visual/technical artists opposing gen AI | 64% | GDC 2026 |
| Designers/narrative writers opposing gen AI | 63% | GDC 2026 |
| Programmers opposing gen AI | 59% | GDC 2026 |
| Developers using AI for code assistance | 47% | GDC 2026 |
| Corporate AI adoption rate | 52% | GDC 2026 |
| Developers using AI for research/brainstorming | 81% | GDC 2026 |
| Developers who lost jobs in past 12 months | 17% | GDC 2026 |

The data reveals a clear pattern: opposition to generative AI is highest among artists (64%) and lowest among programmers (59%), but 47% of developers actively use AI for code assistance even as majority opposition grows. The line is not anti-AI vs. pro-AI -- it is anti-AI-content vs. pro-AI-tools.

81% of developers use AI for research and brainstorming, and 47% use it for code -- but content creation (art, music, assets) remains deeply contested. Code tools sit on the accepted side of the divide.

### Q1 2026 Anti-AI-Content Incidents

Eight significant anti-AI-content incidents occurred in Q1 2026 alone, establishing a pattern across multiple mediums:

| # | Incident | Date | Medium | Outcome |
|---|----------|------|--------|---------|
| 1 | Steam Next Fest "AI slop" flood | Feb 2026 | Video games | Community demands content filters |
| 2 | Clair Obscur: Expedition 33 | Feb 2026 | Video games | Awards stripped by Indie Game Awards |
| 3 | GOG AI banner art | Jan 2026 | Storefront | Apology issued |
| 4 | Crimson Desert AI textures | Mar 2026 | Video games | Apology, patches, retroactive disclosure |
| 5 | Concordia: Special Edition | Feb/Mar 2026 | Board games | Review-bombed to 4.7, publisher promised human art |
| 6 | Larian gen AI abandonment | Jan 2026 | Video games | Full concept art ban, AI text rated "3/10" |
| 7 | NVIDIA DLSS 5 | Mar 2026 | GPU hardware | 84% YouTube dislikes, CEO backtrack |
| 8 | Lutris Claude code backlash | Mar 2026 | Open source | Attribution restored, notably smaller reaction |

Of these eight incidents, seven targeted AI-generated content (art, textures, rendering modifications). One (Lutris) targeted AI-assisted code -- and that incident was notably smaller, more divided, and centered on transparency rather than on the output being harmful. The pattern is consistent: consumer-facing AI content provokes intense backlash; developer-facing AI code assistance provokes limited and qualified concern.

### The Crimson Desert / Clair Obscur Pattern

Two high-profile game launches in March 2026 illustrate a failure mode specific to content tools: the "placeholder that leaked."

**Crimson Desert** (Pearl Abyss): Launched with AI-generated "placeholder" textures that were supposed to be replaced before release. Players found smudged faces, anatomical errors, and harmful stereotypes in AI-generated poster textures within 24 hours. Pearl Abyss is now conducting a comprehensive asset audit and patching every AI-generated image. The game's Steam page was retroactively updated to add AI disclosure.

**Clair Obscur: Expedition 33** (Sandfall Interactive): Used AI-generated content that was discovered post-launch, leading to award revocation by the Indie Game Awards.

Both incidents share the same failure mode: AI images used as development placeholders leaked into the production build because deadlines prevented replacement. This failure mode is structural to content-tool workflows -- temporary AI assets are generated early and are supposed to be replaced, but production pressure causes leaks.

Code-output workflows do not have this failure mode. The code IS the intended output, not a placeholder for something else. There is nothing to "replace before launch."

### NVIDIA DLSS 5

NVIDIA's DLSS 5 announcement at GTC 2026 extended the anti-AI-content backlash to GPU hardware vendors. The technology uses neural rendering to infer photorealistic character models, which gamers accused of applying "Instagram beauty filters" that override developers' art direction. The YouTube showcase received 84% dislikes. Concept artist Karla Ortiz called it "disrespectful to the intentional art direction of devs." Jensen Huang initially dismissed critics as "completely wrong," then backtracked on the Lex Fridman Podcast (#494), admitting: "I can see where they're coming from, because I don't love AI slop myself."

The DLSS 5 incident demonstrates that even AI technology that modifies visual output -- not just generates it -- triggers the same backlash. If NVIDIA cannot ship AI-modified visuals without controversy, no content tool is safe from this reaction. Code output, which never touches visual output, is structurally exempt.

---

## 5. LocalGPT Gen's Position

### Output Format

LocalGPT Gen's output is RON (Rusty Object Notation) scene definitions -- human-readable, human-editable text files that define world structure, entity placement, materials, behaviors, audio, and tours. These definitions are compiled by the Bevy game engine (Rust) into 3D geometry at build time.

A typical output looks like this:

```ron
WorldManifest(
    version: 2,
    meta: WorldMeta(
        name: "mountain-village",
        description: Some("A terraced village on a mountainside"),
        biome: Some("alpine"),
        compliance: Some(ComplianceMeta(
            steam_code_tool_exempt: true,
            eu_ai_act_risk_level: "minimal",
            no_gen_ai_compatible: true,
            generation_tool: "LocalGPT Gen v0.3.5",
            generation_method: "code-generation",
            human_modifiable: true,
        )),
    ),
    entities: [
        // ... parametric shape definitions (position, scale, material, behavior)
    ],
)
```

The output is:
- **Text**: not binary. Every value is readable.
- **Editable**: any developer can open the file and change dimensions, materials, positions, behaviors.
- **Compilable**: the Bevy engine compiles these definitions into GPU draw calls. The final pixels are produced by the engine, not by the AI.
- **Version-controllable**: lives in git, can be diffed, merged, code-reviewed.
- **Not consumer-facing**: players never see RON files. They see the Bevy-rendered 3D world.

### The ComplianceMeta Struct

As of v0.3.5 (March 2026), LocalGPT Gen embeds compliance classification directly in every exported world via the `ComplianceMeta` struct in the `localgpt-world-types` crate. Six fields record the compliance posture:

| Field | Default | Meaning |
|-------|---------|---------|
| `steam_code_tool_exempt` | `true` | Output is code, not pre-made binary assets. Exempt under Steam's AI disclosure policy. |
| `eu_ai_act_risk_level` | `"minimal"` | Minimal-risk classification under Regulation 2024/1689. Transparency only (Article 50), no conformity assessment. |
| `no_gen_ai_compatible` | `true` | Output is human-editable source code. Compatible with "No Gen AI" content provenance requirements. |
| `generation_tool` | `"LocalGPT Gen v{version}"` | Tool name and version for attribution. |
| `generation_method` | `"code-generation"` | LLM writes scene code compiled by the engine, not binary asset data. |
| `human_modifiable` | `true` | Output can be meaningfully edited by a human (RON text with parametric shapes). |

This metadata is:
- Serialized into every `.ron` world manifest file.
- Included as `<meta>` tags in HTML world exports.
- Machine-readable for automated compliance auditing.
- Backward-compatible via `serde(default)` -- existing worlds without the field get correct defaults.

The struct makes compliance classification a property of the output format itself, not a claim made after the fact.

### Contrast with Content Tools

| Property | Code tool (LocalGPT Gen) | Content tool (image/mesh/audio generator) |
|----------|--------------------------|-------------------------------------------|
| Output format | RON text (scene definitions) | Binary (PNG, GLTF, WAV, MP4) |
| Consumer-facing? | No (compiled by Bevy) | Yes (directly in shipped product) |
| Steam disclosure | Exempt (January 2026 rewrite) | Required |
| EU AI Act marking | None required | Two-layered (metadata + watermark), August 2026 |
| "No Gen AI" seal | Compatible | Incompatible |
| Award eligibility | No risk | Disqualification risk (Clair Obscur precedent) |
| AI content filter | Invisible (no AI content) | Filterable/flaggable |
| Placeholder leak risk | None (code is the final output) | High (Crimson Desert, Clair Obscur) |
| Bias/stereotype risk | None (Rust structs) | Present (Crimson Desert offensive textures) |
| Human editability | Full (text file) | Limited (binary format) |

---

## 6. Practical Guidance

### For Developers Using LocalGPT Gen

**What you need to do: nothing.** Compliance is built into the output format.

- **Steam**: No AI disclosure tag needed. Your game's store page does not require an AI content label. Steam's January 2026 rules explicitly exempt code tools.
- **EU AI Act**: No content marking needed. Your game ships no AI-generated synthetic content. The two-layered marking obligation (metadata + watermarks, enforcement August 2, 2026) does not apply.
- **"No Gen AI" seal**: Your game can display the seal. The AI output (RON scene definitions) is developer code, not consumer-facing content. The final creative artifacts are produced by the Bevy engine.
- **Awards**: No disqualification risk from AI code tool usage. Award bodies target AI-generated content in the shipped game.
- **AI content filters**: When Steam or other platforms add AI content filtering, your game will not be flagged because it contains no AI-generated content.
- **ComplianceMeta**: Every world exported by LocalGPT Gen already carries machine-readable compliance metadata confirming all of the above. No manual tagging required.

### For Developers Using Content AI Tools

If you are using AI tools that generate binary assets (images, textures, 3D meshes, audio, video) that appear in your shipped game:

- **Steam**: Add the AI content disclosure tag to your store page. Failure to disclose risks retroactive flagging (Crimson Desert precedent).
- **EU AI Act**: Prepare for two-layered marking (metadata + watermarks) by August 2, 2026. Monitor the Code of Practice finalization in June 2026.
- **"No Gen AI" seal**: You cannot display the seal if your game contains AI-generated consumer-facing content.
- **Awards**: Verify award competition rules regarding AI usage. The Indie Game Awards precedent (Clair Obscur) demonstrates that undisclosed AI content can lead to disqualification.
- **QA process**: Audit all AI-generated placeholders before release. The "placeholder that leaked" failure mode (Crimson Desert, Clair Obscur) has now occurred in two major launches in a single month.

### The Validation Chain

The code-tool compliance case is not a single data point. It is a convergence across five independent authorities:

1. **Platform policy**: Steam explicitly exempts code tools (January 2026).
2. **Platform practice**: Valve commissions human artists for its own marketing while exempting code tools in policy (Spring Sale 2026).
3. **Regulation**: EU AI Act classifies games as minimal-risk, exempts open-source tools, and targets only consumer-facing synthetic content (Regulation 2024/1689, Article 50).
4. **Publisher policy**: Capcom bans AI-generated assets but embraces AI development tools (February 2026). Larian abandons generative AI for creative content while retaining AI development tools (January 2026).
5. **Cultural movement**: The "No Gen AI" seal targets content, not code. Award bodies disqualify based on AI content, not AI tools.

Each of these authorities arrived at the same distinction independently. The consensus is structural, not coincidental.

---

## References

- Steam AI Disclosure Policy, January 2026 rewrite (Valve Corporation)
- EU AI Act, Regulation (EU) 2024/1689 of the European Parliament and of the Council
- EU Code of Practice on marking and labelling of AI-generated content, second draft, March 2026
- GDC 2026 State of the Industry Survey (Game Developers Conference)
- Capcom corporate statement on AI usage, February 2026
- Larian Studios Reddit AMA, January 2026
- Indie Game Awards disqualification of Clair Obscur: Expedition 33, February 2026
- PCGamesN analysis of Steam Next Fest AI disclosure, February 2026
- Pearl Abyss (Crimson Desert) AI art disclosure and patch statement, March 2026
- NVIDIA GTC 2026 DLSS 5 announcement; Jensen Huang on Lex Fridman Podcast #494
- "No Gen AI" seal by Polygon Treehouse
- LocalGPT Gen `ComplianceMeta` struct: `crates/world-types/src/world.rs`
