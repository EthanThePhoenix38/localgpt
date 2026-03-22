# LocalGPT Ecosystem Integration — Checklist Tracker

**Quick-reference companion to `LocalGPT_Rust_Ecosystem_Integration_Spec.md`**

Last updated: February 13, 2026

---

## Cargo.toml Additions (All Tiers)

```toml
# Tier 1 — Week 1
schemars = "0.8"
eventsource-stream = "0.2"

# Tier 2 — Weeks 2-3
genai = "0.5"
tiktoken-rs = "0.6"

# Tier 3 — Weeks 3-5
text-splitter = { version = "0.16", features = ["tiktoken-rs", "markdown"] }
async-stream = "0.3"
```

---

## Tier 1 — Drop-In Improvements (Week 1)

### 3.1 `schemars` for Tool Schema Generation
- [ ] Add `schemars = "0.8"` to Cargo.toml
- [ ] Define `BashToolParams` with `#[derive(JsonSchema)]`
- [ ] Define `FileReadParams` with `#[derive(JsonSchema)]`
- [ ] Define `FileWriteParams` with `#[derive(JsonSchema)]`
- [ ] Define `FileEditParams` with `#[derive(JsonSchema)]`
- [ ] Define `MemorySearchParams` with `#[derive(JsonSchema)]`
- [ ] Define `MemoryAppendParams` with `#[derive(JsonSchema)]`
- [ ] Define `WebFetchParams` with `#[derive(JsonSchema)]`
- [ ] Remove all hand-written JSON schema strings
- [ ] Add schema validation tests in CI
- [ ] Verify tool calling works with OpenAI, Anthropic, Ollama

### 3.2 `eventsource-stream` for SSE Parsing
- [ ] Add `eventsource-stream = "0.2"` to Cargo.toml
- [ ] Refactor OpenAI provider streaming to use `.eventsource()`
- [ ] Refactor Anthropic provider streaming to use `.eventsource()`
- [ ] Refactor Ollama provider streaming (if SSE-based)
- [ ] Update HTTP server SSE relay endpoint
- [ ] Test multi-line `data:` fields
- [ ] Test `[DONE]` sentinel termination
- [ ] Test malformed event resilience

### 3.3 Mock LLM Provider for Testing
- [ ] Create `src/agent/providers/mock.rs`
- [ ] Implement `LlmProvider` trait for `MockProvider`
- [ ] Support deterministic fixture-based responses
- [ ] Support tool-call simulation
- [ ] Support streaming simulation
- [ ] Create `tests/fixtures/` directory with test payloads
- [ ] Wire `mock` as a config provider option
- [ ] Convert existing integration tests to use MockProvider
- [ ] Verify CI runs without API keys

---

## Tier 2 — Strategic Integrations (Weeks 2-3)

### 4.1 `genai` Multi-Provider Abstraction
- [ ] Add `genai = "0.5"` to Cargo.toml
- [ ] Create `src/agent/providers/genai_adapter.rs`
- [ ] Implement `LlmProvider` trait wrapping genai `Client`
- [ ] Message format conversion (LocalGPT ↔ genai)
- [ ] Streaming support through genai
- [ ] Migrate OpenAI provider to genai backend
- [ ] Test Gemini provider
- [ ] Test Groq provider
- [ ] Test DeepSeek provider
- [ ] Keep direct Anthropic provider (tool_use edge cases)
- [ ] Keep direct Ollama provider (local-first users)
- [ ] Update config.toml docs with new provider list
- [ ] Verify binary size increase < 300KB

### 4.2 Tuple-Delimited Parsing Fallback
- [ ] Create `src/agent/parsing.rs`
- [ ] Implement `parse_tuple_output()` function
- [ ] Implement 3-stage parse pipeline: JSON → markdown extraction → tuple fallback
- [ ] Add `ParseMetrics` for monitoring parse paths
- [ ] Log which parse path succeeded per tool call
- [ ] Unit tests: valid JSON input
- [ ] Unit tests: JSON in markdown code blocks
- [ ] Unit tests: tuple-delimited input
- [ ] Unit tests: garbage input (graceful failure)
- [ ] Unit tests: partial recovery (49 of 50 records)

### 4.3 `tiktoken-rs` Token Counting
- [ ] **DECISION:** Accept ~8.5MB binary increase? (27MB → ~35MB)
- [ ] Add `tiktoken-rs = "0.6"` to Cargo.toml
- [ ] Create `src/agent/tokens.rs`
- [ ] Implement `TokenCounter::for_model()`
- [ ] Implement `count()`, `count_messages()`, `remaining_capacity()`
- [ ] Implement `should_compact()` threshold logic
- [ ] Integrate into session management (pre-call compaction check)
- [ ] Integrate into context compiler (budget allocation)
- [ ] Fallback to cl100k_base for unknown models
- [ ] Model max-token lookup table (GPT-4o, Claude 3/4, etc.)
- [ ] Test accuracy within 5% of provider-reported usage

---

## Tier 3 — Feature Parity (Weeks 3-5)

### 5.1 Model Tiering
- [ ] Add `[llm.tiers]` config section
- [ ] Create `src/agent/tiering.rs`
- [ ] Define `ModelTier` enum: Primary, Background, Internal
- [ ] Create `TieredProviderPool`
- [ ] Tag heartbeat operations as `Background`
- [ ] Tag compaction operations as `Internal`
- [ ] Tag user chat as `Primary`
- [ ] Graceful fallback when tier not configured
- [ ] Measure cost reduction (target: >50% fewer expensive-model tokens)

### 5.2 `text-splitter` for Document Chunking
- [ ] Add `text-splitter` with tiktoken-rs + markdown features
- [ ] Create `src/memory/chunker.rs`
- [ ] Implement markdown-aware chunking
- [ ] Integrate into memory indexing pipeline
- [ ] Verify chunks respect sentence boundaries
- [ ] Verify markdown headers preserved as boundaries
- [ ] Verify code blocks not split mid-block

### 5.3 `async-stream` Refactor
- [ ] Add `async-stream = "0.3"` to Cargo.toml
- [ ] Refactor WebSocket stream handler with `stream!` macro
- [ ] Refactor SSE endpoint with `stream!` macro
- [ ] Remove manual `Pin`/`Poll` implementations
- [ ] Verify error propagation in streams

---

## Tier 4 — Evaluations (Ongoing)

### 6.1 Workspace Modularization
- [ ] Measure current incremental compile times
- [ ] If >30s: prototype workspace split (localgpt-memory, localgpt-sandbox, localgpt-llm)
- [ ] If <15s: defer, re-evaluate quarterly

### 6.2 Embedded Inference via `mistral.rs`
- [ ] Prototype `--features local-inference` flag
- [ ] Benchmark vs Ollama for Llama 3.2 3B on CPU
- [ ] Measure binary size impact
- [ ] Decision: ship or defer

### 6.3 `rig-core` Tool Trait Pattern
- [ ] After §3.1 complete: evaluate rig's `Tool` trait design
- [ ] Prototype LocalGPT tool trait migration
- [ ] Decision: adopt pattern or keep current

### 6.4 Graph-RAG (EdgeQuake) for Memory
- [ ] Evaluate EdgeQuake as a library dependency
- [ ] Prototype entity extraction from MEMORY.md
- [ ] Measure retrieval quality improvement
- [ ] Decision: integrate or defer

---

## Success Metrics

| Metric | Target | How to Measure |
|--------|--------|---------------|
| Provider coverage | ≥14 | Count of working providers in `localgpt config providers` |
| CI without API keys | 100% | MockProvider used in all tests |
| Tool-call parse failures | <2% | ParseMetrics logging over 1000 interactions |
| Token count accuracy | ±5% | Compare tiktoken-rs count vs provider-reported usage |
| Binary size | <40MB | `ls -la target/release/localgpt` |
| Test suite speed | <30s | `time cargo test` |
| License compliance | 0 copyleft | `cargo deny check licenses` |

---

## New Files Created

```
src/agent/
├── providers/
│   ├── genai_adapter.rs    # §4.1 — genai multi-provider wrapper
│   └── mock.rs             # §3.3 — deterministic test provider
├── parsing.rs              # §4.2 — JSON + tuple fallback parser
├── tokens.rs               # §4.3 — tiktoken-rs token counter
└── tiering.rs              # §5.1 — model tier routing

src/memory/
└── chunker.rs              # §5.2 — text-splitter integration

tests/fixtures/
├── simple_response.json
├── tool_call_bash.json
├── tool_call_file_write.json
├── multi_turn_conversation.json
└── streaming_chunks.jsonl
```
