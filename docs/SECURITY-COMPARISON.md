# LocalGPT vs OpenClaw Security Comparison

Track LocalGPT's security feature parity with OpenClaw. Goal: **exceed OpenClaw in all aspects**.

## Progress Summary

| Category | LocalGPT | OpenClaw | Status |
|----------|----------|----------|--------|
| Prompt Injection | 12 markers, 10 patterns | 10+ patterns | **PARITY** |
| Content Delimiters | 3 types | Similar | **PARITY** |
| Input Validation | JSON only | Zod schemas | **BEHIND** |
| Sandbox/Isolation | None | Docker-ready | **BEHIND** |
| Tool Policies | require_approval only | Full allowlist/denylist | **BEHIND** |
| SSRF Protection | None | Full (DNS pinning, redirect validation) | **BEHIND** |
| Execution Approval | Config exists, not enforced | Pattern-based, socket server | **BEHIND** |
| Authentication | None | Device identity, OAuth, webhook sigs | **BEHIND** |
| TLS/Encryption | None | Self-signed cert generation | **BEHIND** |
| Rate Limiting | None | Telegram throttler | **BEHIND** |
| Audit System | Basic logging | 50+ security checks | **BEHIND** |
| Secret Detection | Env var expansion | detect-secrets CI | **BEHIND** |

**Legend**: AHEAD / PARITY / BEHIND / NOT APPLICABLE

---

## 1. Prompt Injection Defenses

### Marker Stripping

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| System tags | `<system>`, `</system>`, `<<SYS>>`, `<</SYS>>` | Similar |
| OpenAI format | `<\|system\|>`, `<\|im_start\|>system`, `<\|im_end\|>` | Similar |
| Llama format | `[INST]`, `[/INST]`, `<s>`, `</s>` | Similar |
| Replacement | `[FILTERED]` | `[[MARKER_SANITIZED]]` |
| Location | `src/agent/sanitize.rs` | `src/security/external-content.ts` |

**Status**: **PARITY**

### Suspicious Pattern Detection

| Pattern | LocalGPT | OpenClaw |
|---------|----------|----------|
| "ignore previous instructions" | Yes | Yes |
| "disregard previous" | Yes | Yes |
| "forget instructions" | Yes | - |
| "you are now a" | Yes | - |
| "new instructions" | Yes | Yes |
| "system override" | Yes | Yes |
| "act as" / "pretend to be" | Yes | - |
| "from now on" | Yes | - |
| "bypass safety" | Yes | - |
| "elevated=true" | - | Yes |
| "rm -rf" / "delete all" | - | Yes |
| Unicode fullwidth folding | - | Yes |

**Status**: **PARITY** (different focus areas)

### Content Delimiters

| Delimiter Type | LocalGPT | OpenClaw |
|----------------|----------|----------|
| Tool output | `<tool_output>` | XML wrappers |
| Memory content | `<memory_context>` | - |
| External content | `<external_content>` | Wrapped with warnings |
| Source attribution | Yes (comment) | Yes |
| System prompt guidance | Yes | Yes |

**Status**: **PARITY**

---

## 2. Execution Sandboxing

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Docker support | None | Full sandbox support |
| seccomp/AppArmor | None | Mentioned in docs |
| Non-root execution | N/A | Recommended |
| Read-only filesystem | N/A | Recommended |
| Capability dropping | N/A | `--cap-drop=ALL` |
| Container isolation | None | Per-agent sandboxes |

**Status**: **BEHIND** - Critical gap

### TODO for LocalGPT:
- [ ] Add optional Docker execution mode for bash tool
- [ ] Implement seccomp profiles for restricted syscalls
- [ ] Add container-based isolation for untrusted code
- [ ] Support read-only workspace mounts

---

## 3. Tool Policies & Access Control

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Tool profiles | None | minimal/coding/messaging/full |
| Tool groups | None | group:memory, group:web, etc. |
| Allowlist | `require_approval` (not enforced) | Pattern-based allowlist |
| Denylist | None | Explicit deny patterns |
| Wildcard matching | None | Glob-style patterns |
| Per-agent overrides | None | Agent-specific policies |
| Default deny | None | Sandbox defaults deny UI/channel tools |

**Status**: **BEHIND** - Major gap

### OpenClaw Tool Groups:
```
group:memory, group:web, group:fs, group:runtime
group:sessions, group:ui, group:automation, group:messaging
group:plugins
```

### TODO for LocalGPT:
- [ ] Define tool groups (memory, web, fs, system)
- [ ] Implement pattern-based allowlist/denylist
- [ ] Add tool profiles (minimal, standard, full)
- [ ] Enforce `require_approval` config
- [ ] Add per-agent tool policy overrides

---

## 4. SSRF Protection

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Private IP blocking | None | Full (10.x, 172.16-31.x, 192.168.x, etc.) |
| IPv6 private ranges | None | fe80:, fec0:, fc, fd |
| IPv4-mapped IPv6 | None | Detected and blocked |
| Blocked hostnames | None | localhost, metadata.google.internal |
| Blocked TLDs | None | .localhost, .local, .internal |
| DNS pinning | None | Resolves and validates all IPs |
| Redirect validation | None | Follows and validates targets |
| Policy config | None | `allowPrivateNetwork` flag |

**Status**: **BEHIND** - Security risk for web_fetch

### TODO for LocalGPT:
- [ ] Implement private IP detection for IPv4/IPv6
- [ ] Block metadata endpoints (cloud provider SSRF)
- [ ] Add DNS resolution validation before fetch
- [ ] Validate redirect targets
- [ ] Add URL allowlist/blocklist config

---

## 5. Authentication & Identity

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Device identity | None | Ed25519 keypair |
| Device signing | None | Signature generation/verification |
| Auth tokens | None | Role/scope-based tokens |
| OAuth support | None | Google, GitHub, Qwen |
| Webhook signatures | None | Twilio, Plivo, LINE |
| API authentication | None | Token-based gateway auth |
| File permissions | None | 0o600 on sensitive files |

**Status**: **BEHIND** - Required for multi-user

### TODO for LocalGPT:
- [ ] Add API key authentication for HTTP endpoints
- [ ] Implement device identity (Ed25519)
- [ ] Add token-based session authentication
- [ ] Set secure file permissions on config/keys
- [ ] Add OAuth support for model providers

---

## 6. TLS & Encryption

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| HTTPS support | None | Self-signed cert generation |
| Certificate fingerprinting | None | SHA-256 fingerprinting |
| Custom CA support | None | Yes |
| Key file permissions | None | 0o600 enforcement |
| SQLite encryption | None | Mentioned in TODO |
| Config encryption | None | Mentioned in TODO |

**Status**: **BEHIND**

### TODO for LocalGPT:
- [ ] Add optional TLS for HTTP server
- [ ] Implement self-signed cert generation
- [ ] Add SQLCipher support for encrypted DB
- [ ] Encrypt sensitive config values

---

## 7. Audit & Security Scanning

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Security audit tool | None | 50+ checks in audit.ts |
| Bind address validation | None | loopback vs internet-facing |
| Auth token validation | None | Length and strength checks |
| Filesystem permission audit | None | World/group writable checks |
| Symlink detection | None | State dir and config checks |
| Channel security audit | N/A | Discord/Slack/Telegram policies |
| Allowlist size warnings | None | >25 entries flagged |
| Wildcard detection | None | Open policy warnings |

**Status**: **BEHIND** - OpenClaw has comprehensive audit system

### TODO for LocalGPT:
- [ ] Create `localgpt audit` command
- [ ] Check bind address security
- [ ] Validate file permissions
- [ ] Warn on overly permissive configs
- [ ] Add structured audit logging for SIEM

---

## 8. Rate Limiting

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Tool call rate limiting | None | Per-channel throttling |
| API rate limiting | None | Telegram throttler |
| Session limits | 100 max | Not explicit |
| Request throttling | None | grammy-throttler |

**Status**: **BEHIND**

### TODO for LocalGPT:
- [ ] Add per-tool rate limits
- [ ] Implement token bucket or sliding window
- [ ] Add configurable limits per session
- [ ] Add API endpoint rate limiting

---

## 9. Input Validation

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Schema validation | JSON parsing only | Zod schemas |
| Type checking | Runtime JSON | Compile-time TypeScript |
| Duration parsing | Custom (s/m/h/d) | Similar |
| Time parsing | HH:MM format | Similar |
| Path validation | shellexpand only | Plus sanitization |
| URL validation | None | SSRF checks |

**Status**: **BEHIND** - LocalGPT lacks schema validation

### TODO for LocalGPT:
- [ ] Add JSON schema validation for tool args
- [ ] Validate integer bounds
- [ ] Validate string lengths
- [ ] Add path canonicalization
- [ ] Implement URL validation

---

## 10. Secret Management

| Feature | LocalGPT | OpenClaw |
|---------|----------|----------|
| Env var expansion | Yes (`${VAR}`) | Similar |
| Secret detection CI | None | detect-secrets |
| Secrets baseline | None | .secrets.baseline |
| Pre-commit hooks | None | Secret scanning hooks |

**Status**: **BEHIND**

### TODO for LocalGPT:
- [ ] Add pre-commit secret detection
- [ ] Create secrets baseline
- [ ] Document secure credential handling

---

## 11. Features LocalGPT Should NOT Implement

These OpenClaw features are out of scope for LocalGPT (local-only design):

| Feature | Reason |
|---------|--------|
| Channel allowlists (Discord/Slack/Telegram) | No remote channels |
| Gateway security | No multi-user gateway |
| Tailscale integration | Local-only |
| Webhook signature verification | No incoming webhooks |
| Channel metadata sanitization | No channels |

---

## Priority Roadmap

### Phase 1: Critical Security (P0)
1. **Bash sandboxing** - Docker or seccomp isolation
2. **SSRF protection** - Private IP blocking, DNS validation
3. **API authentication** - At minimum, localhost + token

### Phase 2: Access Control (P1)
4. **Tool policies** - Full allowlist/denylist system
5. **Execution approval** - Enforce `require_approval`
6. **Path validation** - Prevent traversal attacks

### Phase 3: Enterprise Ready (P2)
7. **TLS support** - HTTPS for HTTP server
8. **Audit system** - Security audit command
9. **Rate limiting** - Per-tool and per-session limits

### Phase 4: Parity+ (P3)
10. **Device identity** - Ed25519 signing
11. **Secret detection** - CI integration
12. **Schema validation** - Full tool arg validation

---

## Metrics

Track progress toward OpenClaw parity:

| Metric | Current | Target |
|--------|---------|--------|
| Features at parity | 3 | 12 |
| Features ahead | 0 | 3+ |
| Critical gaps | 3 | 0 |
| High priority gaps | 5 | 0 |

---

*Last updated: 2026-02-04*
