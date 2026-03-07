---
sidebar_position: 9.1
slug: /claw
---

# Feature Parity Matrix — Claw Ecosystem

> **⚠️ AI-Generated Documentation:** This document was generated and is maintained by AI assistants. While efforts are made to ensure accuracy, many details must be outdated or incorrect as those projects are moving very fast. Please verify with the source repositories for the most current information.
>
> **Last updated:** 2026-03-07

This document tracks feature parity across ten implementations of the personal AI assistant architecture. OpenClaw (TypeScript) is the reference implementation; IronClaw, LocalGPT, Moltis, and ZeroClaw are Rust implementations; Nanobot and CoPaw are Python implementations; PicoClaw is Go; NullClaw is Zig; MimiClaw is C (ESP32).

### GitHub Repositories

| Project | Language | License | Repository |
|---------|----------|---------|------------|
| **OpenClaw** | TypeScript | MIT | https://github.com/openclaw/openclaw |
| **IronClaw** | Rust | MIT/Apache 2.0 | https://github.com/nearai/ironclaw |
| **LocalGPT** | Rust | Apache 2.0 | https://github.com/localgpt-app/localgpt |
| **Moltis** | Rust | MIT | https://github.com/moltis-org/moltis |
| **Nanobot** | Python | MIT | https://github.com/HKUDS/nanobot |
| **CoPaw** | Python | Apache 2.0 | https://github.com/agentscope-ai/CoPaw |
| **PicoClaw** | Go | MIT | https://github.com/sipeed/picoclaw |
| **ZeroClaw** | Rust | MIT/Apache 2.0 | https://github.com/zeroclaw-labs/zeroclaw |
| **NullClaw** | Zig | MIT | https://github.com/nullclaw/nullclaw |
| **MimiClaw** | C (ESP32) | MIT | https://github.com/memovai/mimiclaw |

**Legend:**
- ✅ Implemented
- 🚧 Partial (in progress or incomplete)
- ❌ Not implemented
- 🔮 Planned (in scope but not started)
- 🚫 Out of scope (intentionally skipped)
- ➖ N/A (not applicable)

---

## 1. Architecture

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Hub-and-spoke architecture | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | MimiClaw: embedded single-purpose |
| WebSocket control plane | ✅ | ✅ | 🚧 | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | |
| Single-user system | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Multi-agent routing | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Session-based messaging | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Loopback-first networking | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Bridge daemon protocol (IPC) | ➖ | ➖ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: tarpc-based localgpt-bridge |
| GraphQL API | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Moltis: HTTP + WebSocket GraphQL |
| Trait-driven architecture | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | |
| Ultra-lightweight runtime | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | |
| Embedded hardware support | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | MimiClaw: ESP32-S3 ($5) |
| OTA updates | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | MimiClaw: over-the-air firmware updates |
| No OS/runtime | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | MimiClaw: bare metal, no Linux/Node.js |

---

## 2. Gateway System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Gateway control plane | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | |
| HTTP endpoints for Control UI | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | CoPaw: Console web UI |
| Channel connection lifecycle | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Session management/routing | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Configuration hot-reload | ✅ | ❌ | ✅ | 🚧 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | |
| Network modes (loopback/LAN/remote) | ✅ | 🚧 | 🚧 | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | |
| OpenAI-compatible HTTP API | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Canvas hosting | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Gateway lock (PID-based) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| launchd/systemd integration | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | CoPaw: daemon mode |
| Bonjour/mDNS discovery | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Tailscale integration | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Health check endpoints | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| `doctor` diagnostics | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | |
| Agent event broadcast | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Channel health monitor | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Presence system | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Trusted-proxy auth mode | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| APNs push pipeline | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Oversized payload guard | ✅ | 🚧 | ✅ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Pre-prompt context diagnostics | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| TLS/HTTPS auto-certs | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| WebAuthn/passkey auth | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Rate limiting (per-IP) | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Prometheus metrics | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | |
| Serial CLI config | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | MimiClaw: runtime config via serial |

---

## 3. Messaging Channels

| Channel | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|----------|-------|
| CLI/TUI | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| HTTP webhook | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| REPL (simple) | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| WASM channels | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | IronClaw innovation |
| WhatsApp | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | P1 | |
| Telegram | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | MimiClaw: primary channel |
| Discord | ✅ | ❌ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P2 | |
| Signal | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | P2 | |
| Slack | ✅ | ✅ | ❌ | 🚧 | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | - | |
| iMessage | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | P3 | |
| Linq | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | ZeroClaw only |
| Feishu/Lark | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | P3 | MimiClaw: supported |
| LINE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | P3 | |
| WebChat | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | - | |
| Matrix | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | P3 | ZeroClaw: E2EE support |
| Mattermost | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | P3 | |
| Google Chat | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| MS Teams | ✅ | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Twitch | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Voice Call | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | P3 | CoPaw: Twilio voice |
| Nostr | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | P3 | |
| QQ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P3 | |
| DingTalk | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P3 | |
| Email (IMAP/SMTP) | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | P3 | |
| IRC | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | P3 | |
| WeCom (企业微信) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | P3 | PicoClaw only |
| MaixCam | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | P3 | Embedded camera |
| OneBot | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | P3 | QQ protocol |
| MQTT | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | ZeroClaw: IoT messaging |
| Nextcloud Talk | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | ZeroClaw only |
| WATI (WhatsApp Business) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | ZeroClaw only |

### Telegram-Specific Features (since Feb 2025)

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| Forum topic creation | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| channel_post support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| User message reactions | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| sendPoll | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Cron/heartbeat topic targeting | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Streaming message edits | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | |

### Discord-Specific Features (since Feb 2025)

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| Forwarded attachment downloads | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Faster reaction state machine | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thread parent binding inheritance | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |

### Slack-Specific Features (since Feb 2025)

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| Streaming draft replies | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Configurable stream modes | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thread ownership | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |

### Channel Features

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| DM pairing codes | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | |
| Allowlist/blocklist | ✅ | 🚧 | 🚧 | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Self-message bypass | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | |
| Mention-based activation | ✅ | ✅ | ✅ | 🚧 | ❌ | ✅ | ✅ | ✅ | ✅ | |
| Per-group tool policies | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thread isolation | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Per-channel media limits | ✅ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Typing indicators | ✅ | 🚧 | ✅ | 🚧 | ❌ | ✅ | ❌ | ✅ | ✅ | |
| Per-channel ackReaction config | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Group session priming | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Sender_id in trusted metadata | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |

---

## 4. CLI Commands

| Command | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|----------|-------|
| `run` (agent) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | MimiClaw: always-on embedded |
| `tool install/list/remove` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `gateway start/stop` | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P2 | |
| `onboard` (wizard) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| `tui` | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | - | CoPaw: Console web UI |
| `config` | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | - | MimiClaw: serial CLI |
| `channels` | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P2 | |
| `models` | ✅ | 🚧 | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | - | MimiClaw: switch provider at runtime |
| `status` | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| `agents` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | |
| `sessions` | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | P3 | |
| `memory` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | - | MimiClaw: local flash storage |
| `skills` | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ | - | MimiClaw: on-device skills |
| `pairing` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | - | |
| `nodes` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `plugins` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `hooks` | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P2 | |
| `cron` | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | P2 | MimiClaw: on-device cron |
| `webhooks` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `message send` | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P2 | |
| `browser` | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | P3 | |
| `sandbox` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | - | |
| `doctor` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | P2 | |
| `logs` | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P3 | |
| `update` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | MimiClaw: OTA updates |
| `completion` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | |
| `/subagents spawn` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `/export-session` | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | |
| `auth` (OAuth management) | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | - | |
| `desktop` (GUI) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: egui/eframe |
| `db` (database management) | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `tailscale` | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `md sign/verify/policy` | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `bridge list/show/remove` | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `hardware` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | - | MimiClaw: ESP32 GPIO |
| `goals` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | - | ZeroClaw: goals system |
| `sop` (Standard Operating Procedures) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | - | ZeroClaw: sop_execute/list/approve/status |
| `ota` (over-the-air update) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | MimiClaw only |

---

## 5. Agent System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Pi agent runtime | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | All Rust/Go/Zig/C impls use custom runtimes |
| RPC-based execution | ✅ | ✅ | 🚧 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: tarpc IPC for bridge daemons |
| Multi-provider failover | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | MimiClaw: Anthropic + OpenAI switchable |
| Per-sender sessions | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Global sessions | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Session pruning | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Context compaction | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Post-compaction read audit | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Post-compaction context injection | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Custom system prompts | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Skills (modular capabilities) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | MimiClaw: on-device skills |
| Skill routing blocks | ✅ | 🚧 | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | |
| Skill path compaction | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thinking modes (low/med/high) | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | CoPaw: optional thinking display |
| Per-model thinkingDefault override | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Block-level streaming | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Tool-level streaming | ✅ | ❌ | 🚧 | 🚧 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | CoPaw: optional tool call display |
| Z.AI tool_stream | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Plugin tools | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | All: MCP tools; IronClaw: WASM |
| Tool policies (allow/deny) | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Exec approvals (`/approve`) | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Elevated mode | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Subagent support | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| `/subagents spawn` command | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Auth profiles | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Generic API key rotation | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Stuck loop detection | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| llms.txt discovery | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Multiple images per tool call | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | |
| URL allowlist (web_search/fetch) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | |
| suppressToolErrors config | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Intent-first tool display | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Transcript file size in status | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Session branching | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Moltis: `branch_session` tool |
| Agent interruption API | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | CoPaw: v0.0.5 |
| Delegate tool | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | Route to specialized subagents |
| SOP execution | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ZeroClaw: Standard Operating Procedures |
| On-device agent loop | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | MimiClaw: ESP32 agent loop |

---

## 6. Model & Provider Support

| Provider | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Priority | Notes |
|----------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|----------|-------|
| NEAR AI | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Anthropic (Claude) | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| OpenAI | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| AWS Bedrock | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | P3 | |
| Google Gemini | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | P3 | |
| NVIDIA API | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| OpenRouter | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| Tinfoil | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | IronClaw-only |
| OpenAI-compatible | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| Ollama (local) | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | - | |
| Perplexity | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| MiniMax | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | P3 | |
| GLM-5 | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | P3 | |
| node-llama-cpp | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | - | N/A for Rust/Go/Zig/C |
| llama.cpp (native) | ❌ | 🔮 | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| X.AI (Grok) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| GitHub Copilot | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | - | |
| CLI-based providers (subprocess) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | - | PicoClaw: claude-cli, codex-cli |
| Kimi/Moonshot | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | - | |
| DeepSeek | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | - | CoPaw: DeepSeek Reasoner |
| Groq | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ | - | |
| DashScope/Qwen | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | - | |
| VolcEngine | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | - | |
| SiliconFlow | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | - | |
| AiHubMix | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| OpenAI Codex (OAuth) | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | - | |
| vLLM | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Antigravity | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | - | PicoClaw only |
| Telnyx | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | - | ZeroClaw: SMS/voice |

### Model Features

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Auto-discovery | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | |
| Failover chains | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | MimiClaw: Anthropic ↔ OpenAI |
| Cooldown management | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | |
| Per-session model override | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | MimiClaw: runtime switch |
| Model selection UI | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | |
| Per-model thinkingDefault | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | |
| 1M context beta header | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Provider-native tool definitions | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Provider aliases | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Model routing config | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ZeroClaw: model_routing_config tool |

---

## 7. Media Handling

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Image processing (Sharp) | ✅ | ❌ | 🚧 | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | P2 | |
| Configurable image resize dims | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| Multiple images per tool call | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | P2 | |
| Audio transcription | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | P2 | CoPaw: Twilio voice; PicoClaw/ZeroClaw: transcription channel |
| Video support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| PDF parsing | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | ZeroClaw: pdf_read tool |
| MIME detection | ✅ | ❌ | 🚧 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| Media caching | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Vision model integration | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | P2 | |
| TTS (Edge TTS) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| TTS (OpenAI) | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | Moltis: 5 providers |
| Incremental TTS playback | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Sticker-to-image | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Procedural audio synthesis | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: FunDSP in Gen mode |
| STT (multiple providers) | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| Web content extraction | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| Screenshot capture | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | - | ZeroClaw/NullClaw: screenshot tool |

---

## 8. Plugin & Extension System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| Dynamic loading | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | |
| Manifest validation | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | |
| HTTP path registration | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Workspace-relative install | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Channel plugins | ✅ | ✅ | 🚧 | 🚧 | ❌ | ✅ | ✅ | ✅ | ✅ | |
| Auth plugins | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Memory plugins | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Tool plugins | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Hook plugins | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | |
| Provider plugins | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Plugin CLI (`install`, `list`) | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | |
| ClawHub registry | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| `before_agent_start` hook | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | |
| `before_message_write` hook | ✅ | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | |
| `llm_input`/`llm_output` hooks | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | |
| MCP support (stdio + HTTP/SSE) | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Browser automation (CDP) | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | |
| Composio integration | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | NullClaw: composio tool |
| WASM module tools | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ZeroClaw: wasmi runtime |

---

## 9. Configuration System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| Primary config file | ✅ `openclaw.json` | ✅ `.env` | ✅ `config.toml` | ✅ `moltis.toml` | ✅ `config.json` | ✅ `config.yaml` | ✅ `config.yaml` | ✅ `config.toml` | ✅ `config.json` | |
| JSON5 support | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | |
| YAML alternative | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | |
| Environment variable interpolation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Config validation/schema | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | |
| Hot-reload | ✅ | ❌ | ✅ | 🚧 | ❌ | ✅ | ❌ | ❌ | ❌ | |
| Legacy migration | ✅ | ➖ | ➖ | ➖ | ➖ | ❌ | ❌ | ✅ | ❌ | ZeroClaw: migration.rs |
| State directory | ✅ `~/.openclaw-state/` | ✅ `~/.ironclaw/` | ✅ `~/.localgpt/` | ✅ `~/.moltis/` | ✅ `~/.nanobot/` | ✅ `~/.copaw/` | ✅ `~/.picoclaw/` | ✅ `~/.zeroclaw/` | ✅ `~/.nullclaw/` | |
| Credentials directory | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ZeroClaw: encrypted with chacha20poly1305 |
| Full model compat fields in schema | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Profile support | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| JSON Schema export | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ZeroClaw: schemars |

---

## 10. Memory & Knowledge System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| Vector memory | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | |
| Session-based memory | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Hybrid search (BM25 + vector) | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | |
| Temporal decay (hybrid search) | ✅ | ❌ | ✅ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | |
| MMR re-ranking | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| LLM-based query expansion | ✅ | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | |
| OpenAI embeddings | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | |
| Gemini embeddings | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Local embeddings | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | |
| SQLite-vec backend | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | |
| LanceDB backend | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| QMD backend | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Atomic reindexing | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Embeddings batching | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Citation support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Memory CLI commands | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | |
| Flexible path structure | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Identity files (AGENTS.md, etc.) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | |
| Daily logs | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | |
| Heartbeat checklist | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| File watcher (workspace changes) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Search result caching | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Two-layer memory (facts + history) | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | |
| RAG system | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ZeroClaw: rag crate |
| Memory store/recall/forget tools | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | |

---

## 11. Mobile Apps

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| iOS app (SwiftUI) | ✅ | 🚫 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: UniFFI + XCFramework |
| Android app (Kotlin) | ✅ | 🚫 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: UniFFI + cargo-ndk |
| Apple Watch companion | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Gateway WebSocket client | ✅ | 🚫 | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Camera/photo access | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Voice input | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Push-to-talk | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Location sharing | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Node pairing | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| APNs push notifications | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Share to OpenClaw (iOS) | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Background listening toggle | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| UniFFI mobile bindings | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| PWA (Progressive Web App) | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | - | CoPaw: Console web UI |
| ESP32 firmware | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | - | ZeroClaw: zeroclaw-esp32 |
| Nucleo firmware | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | - | ZeroClaw: zeroclaw-nucleo |
| MaixCam support | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | - | Embedded camera platform |

---

## 12. macOS / Desktop App

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| SwiftUI native app | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Menu bar presence | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Bundled gateway | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Canvas hosting | ✅ | 🚫 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Voice wake | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Voice wake overlay | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Push-to-talk hotkey | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Exec approval dialogs | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | - | |
| iMessage integration | ✅ | 🚫 | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | - | |
| Instances tab | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Agent events debug window | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Sparkle auto-updates | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Cross-platform desktop GUI | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | - | LocalGPT: egui; CoPaw: Console web UI |
| Robot kit | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | - | ZeroClaw: look/sense/drive/speak/listen/emote |

---

## 13. Web Interface

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Control UI Dashboard | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | - | CoPaw: Console web UI |
| Channel status view | ✅ | 🚧 | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | - | |
| Agent management | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | P3 | |
| Model selection | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | - | |
| Config editing | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | P3 | |
| Debug/logs viewer | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | - | |
| WebChat interface | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | - | |
| Canvas system (A2UI) | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Control UI i18n | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | P3 | CoPaw: i18n support |
| WebChat theme sync | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Partial output on abort | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| GraphQL playground | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Session sharing via URL | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Version update notifications | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | - | CoPaw: v0.0.5 |

---

## 14. Automation

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Cron jobs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| Cron stagger controls | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Cron finished-run webhook | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Timezone support | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| One-shot/recurring jobs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| Channel health monitor | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | P2 | |
| `beforeInbound` hook | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| `beforeOutbound` hook | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| `beforeToolCall` hook | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| `before_agent_start` hook | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| `before_message_write` hook | ✅ | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `onMessage` hook | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | - | |
| `onSessionStart` hook | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| `onSessionEnd` hook | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| `transcribeAudio` hook | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `transformResponse` hook | ✅ | ✅ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `llm_input`/`llm_output` hooks | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P3 | |
| Bundled hooks | ✅ | ✅ | 🚧 | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| Plugin hooks | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Workspace hooks | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| Outbound webhooks | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| Heartbeat system | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| Gmail pub/sub | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Cron delivery routing | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | - | |
| Pushover notifications | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | - | ZeroClaw/NullClaw: pushover tool |

---

## 15. Security Features

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| Gateway token auth | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | |
| Device pairing | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | |
| Tailscale identity | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Trusted-proxy auth | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| OAuth flows | ✅ | 🚧 | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | |
| DM pairing verification | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | |
| Allowlist/blocklist | ✅ | 🚧 | 🚧 | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Per-group tool policies | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Exec approvals | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | |
| TLS 1.3 minimum | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| SSRF protection | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | |
| SSRF IPv6 transition bypass block | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Cron webhook SSRF guard | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Loopback-first | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Docker sandbox | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | |
| Podman support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| WASM sandbox | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ZeroClaw: wasmi |
| Sandbox env sanitization | ✅ | 🚧 | ✅ | 🚧 | ❌ | ❌ | ✅ | ✅ | ✅ | |
| Tool policies | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | |
| Elevated mode | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Safe bins allowlist | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| LD*/DYLD* validation | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Path traversal prevention | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Credential theft via env injection | ✅ | 🚧 | ✅ | 🚧 | ❌ | ❌ | ✅ | ✅ | ✅ | |
| Session file permissions (0o600) | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | |
| Skill download path restriction | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Webhook signature verification | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Media URL validation | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Prompt injection defense | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | |
| Leak detection | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | |
| Dangerous tool re-enable warning | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| OS-level sandbox (Landlock/Seatbelt) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | NullClaw: landlock, firejail, bubblewrap |
| Policy signing (HMAC-SHA256) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| WebAuthn/passkey auth | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Apple Container sandbox | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Zero unsafe code | ❌ | ❌ | ❌ | ✅ | ➖ | ➖ | ❌ | ❌ | ❌ | N/A for Python |
| WebSocket origin validation | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Encrypted secrets storage | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ZeroClaw/NullClaw: chacha20poly1305 AEAD |

---

## 16. Development & Build System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|-------|
| Primary language | TypeScript | Rust | Rust | Rust | Python | Python | Go | Rust | Zig | C (ESP-IDF) | |
| Build tool | tsdown | cargo | cargo | cargo | pip/uv | pip/uv | go build | cargo | zig build | idf.py | |
| Type checking | TypeScript/tsgo | rustc | rustc | rustc | ❌ | ❌ | ❌ | rustc | Zig | ❌ | |
| Linting | Oxlint | clippy | clippy | clippy | ❌ | black/ruff | ❌ | clippy | Zig | ❌ | |
| Formatting | Oxfmt | rustfmt | rustfmt | rustfmt | ❌ | black | gofmt | rustfmt | zig fmt | ❌ | |
| Package manager | pnpm | cargo | cargo | cargo | pip/uv | pip/uv | go mod | cargo | zig | ESP-IDF | |
| Test framework | Vitest | built-in | built-in | built-in | ❌ | pytest | built-in | built-in | built-in | ❌ | |
| Coverage | V8 | tarpaulin/llvm-cov | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| CI/CD | GitHub Actions | GitHub Actions | GitHub Actions | GitHub Actions | ❌ | GitHub Actions | GitHub Actions | GitHub Actions | GitHub Actions | GitHub Actions | |
| Pre-commit hooks | prek | - | - | - | - | - | - | - | - | - | |
| Docker: Chromium + Xvfb | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Docker: init scripts | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | |
| Browser: extraArgs config | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Workspace crate count | ➖ | 1 | 13 | 47 | ➖ | ➖ | ➖ | 2 | ➖ | ➖ | IronClaw: monolithic single crate |
| Mobile build scripts | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ZeroClaw: ESP32/Nucleo firmware |
| Nix/direnv support | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| mdBook documentation | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | |
| Rust edition | ➖ | ➖ | 2024 | 2024 | ➖ | ➖ | ➖ | 2021 | ➖ | ➖ | |
| Go version | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | 1.21+ | ➖ | ➖ | ➖ | |
| Zig version | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | 0.15.2 | ➖ | |
| ESP-IDF version | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | 5.5+ | MimiClaw only |
| Docker multi-arch | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Lightweight profile | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | NullClaw: ReleaseSmall |
| Docker support | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | |
| Systemd service docs | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | |
| Homebrew package | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | |
| Binary size (release) | ~28 MB | ~44 MB | ~15 MB | ~large | N/A | N/A | ~8 MB | ~3.4 MB | ~678 KB | ~firmware | |
| RAM footprint | &gt;1 GB | ~large | ~moderate | ~large | &gt;100 MB | ~moderate | &lt;10 MB | &lt;5 MB | ~1 MB | ~512 KB | MimiClaw: 8MB PSRAM |
| Startup time (0.8 GHz) | &gt;500 s | ~fast | ~fast | ~fast | &gt;30 s | ~fast | &lt;1 s | &lt;10 ms | &lt;8 ms | instant | MimiClaw: instant on power |
| Power consumption | ~100 W | ~moderate | ~moderate | ~moderate | ~moderate | ~moderate | &lt;5 W | &lt;5 W | &lt;1 W | 0.5 W | MimiClaw: USB power |
| Target hardware | Mac/PC | Mac/PC | Mac/PC | Mac/PC | Linux SBC | Mac/PC | $10 board | $10 board | $5 board | $5 ESP32-S3 | |

---

## 17. Gen Mode / 3D Scene Generation

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|-------|
| 3D rendering engine | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: Bevy 0.18 |
| glTF/GLB scene loading | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Entity spawning/modification tools | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Procedural audio (FunDSP) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Spatial audio | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Audio emitters | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Lock-free audio parameters | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Auto-inference sound from entity name | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Headless/remote control mode | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Hardware peripherals (I2C, SPI, GPIO) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | I2C/SPI tools |

---

## Implementation Priorities

### P0 - Core (Already Done)

**All implementations:**
- ✅ Session management + context compaction
- ✅ Heartbeat system
- ✅ Custom system prompts + skills
- ✅ Subagent support
- ✅ Multi-provider LLM

**IronClaw additionally:**
- ✅ TUI (rustyline + termimad) + HTTP webhook + WASM sandbox
- ✅ Web Control UI + WebChat + DM pairing
- ✅ Gateway control plane + WebSocket
- ✅ Docker sandbox + cron scheduling
- ✅ Tinfoil private inference

**LocalGPT additionally:**
- ✅ CLI chat + HTTP server + web UI
- ✅ Telegram/Discord/WhatsApp bridges
- ✅ iOS/Android via UniFFI
- ✅ Gen mode (Bevy 3D + FunDSP audio)
- ✅ OS-level sandbox (Landlock/Seatbelt)
- ✅ OAuth for 4+ providers
- ✅ Desktop GUI (egui)
- ✅ OpenAI-compatible HTTP API (`/v1/chat/completions`, `/v1/models`)
- ✅ MCP support (stdio + HTTP/SSE)
- ✅ Cron scheduling + lifecycle hooks
- ✅ Multi-provider failover + rate limiting + gateway auth
- ✅ Config hot-reload + session pruning + doctor diagnostics

**Moltis additionally:**
- ✅ Gateway (Axum + WS + GraphQL)
- ✅ Telegram + WhatsApp channels + web dashboard
- ✅ Docker + Apple Container sandbox
- ✅ MCP support (stdio + HTTP/SSE)
- ✅ 17 hook event types
- ✅ TTS (5 providers) + STT (9 providers)
- ✅ Browser automation (CDP)
- ✅ Tailscale integration
- ✅ WebAuthn/passkey auth

**Nanobot additionally:**
- ✅ 9 messaging channels + email
- ✅ 15+ LLM providers (strong Chinese ecosystem)
- ✅ MCP support (stdio + HTTP)
- ✅ Cron with delivery routing
- ✅ OAuth for GitHub Copilot + OpenAI Codex
- ✅ Two-layer memory (MEMORY.md + HISTORY.md)

**CoPaw additionally:**
- ✅ Console web UI with channel management
- ✅ DingTalk, Feishu, QQ, Discord, iMessage, Telegram
- ✅ MCP support (stdio + HTTP)
- ✅ Cron with delivery routing
- ✅ Twilio voice channel
- ✅ Daemon mode
- ✅ Agent interruption API

**PicoClaw additionally:**
- ✅ Ultra-lightweight Go binary (&lt;10MB RAM, &lt;1s boot)
- ✅ Multi-arch: RISC-V, ARM, MIPS, x86
- ✅ 10+ channels including WeCom, MaixCam, OneBot
- ✅ CLI-based providers (claude-cli, codex-cli)
- ✅ I2C hardware support
- ✅ MCP support

**ZeroClaw additionally:**
- ✅ Ultra-lightweight Rust binary (&lt;5MB RAM)
- ✅ 26 channels including MQTT, Nextcloud Talk, Linq
- ✅ Matrix E2EE support
- ✅ WASM sandbox (wasmi)
- ✅ Robot kit (look/sense/drive/speak/listen/emote)
- ✅ SOP (Standard Operating Procedures)
- ✅ Goals system
- ✅ RAG system
- ✅ ESP32/Nucleo firmware
- ✅ Encrypted secrets (chacha20poly1305)

**NullClaw additionally:**
- ✅ Ultra-lightweight Zig binary (678KB, ~1MB RAM, &lt;2ms boot)
- ✅ 18 channels + 23 providers + 18 tools
- ✅ Hybrid vector+FTS5 memory
- ✅ Multi-layer sandbox (landlock, firejail, bubblewrap, docker)
- ✅ Hardware peripherals (I2C, SPI)
- ✅ Composio integration
- ✅ 3,230+ tests

**MimiClaw additionally:**
- ✅ ESP32 bare metal (no Linux, no Node.js, pure C)
- ✅ $5 chip — cheapest AI assistant deployment
- ✅ Telegram-first interface
- ✅ OTA firmware updates
- ✅ On-device agent loop
- ✅ Local flash memory storage
- ✅ Dual provider (Anthropic + OpenAI)
- ✅ On-device cron scheduling
- ✅ 0.5W power consumption

### P1 - High Priority
- ❌ WhatsApp channel (IronClaw, CoPaw)
- ❌ OpenAI-compatible API (Moltis, CoPaw, PicoClaw, ZeroClaw, NullClaw)
- ❌ Configuration hot-reload (IronClaw, PicoClaw, ZeroClaw, NullClaw)

### P2 - Medium Priority
- ❌ Media handling: images, PDFs (IronClaw, LocalGPT, PicoClaw)
- ❌ Outbound webhooks (Moltis, CoPaw, PicoClaw, ZeroClaw, NullClaw)
- ❌ Web UI: channel status, config editing (LocalGPT, PicoClaw, ZeroClaw, NullClaw)

### P3 - Lower Priority
- ❌ Discord/Matrix (IronClaw, Moltis)
- ❌ TTS/audio (IronClaw, LocalGPT, PicoClaw, ZeroClaw, NullClaw)
- ❌ WASM sandbox (LocalGPT, Moltis, PicoClaw, NullClaw)
- ❌ Plugin registry (LocalGPT, CoPaw, PicoClaw, ZeroClaw, NullClaw)
- ❌ Mobile apps (IronClaw, Moltis, Nanobot, CoPaw, PicoClaw, ZeroClaw, NullClaw)
- ❌ Desktop app (IronClaw, Moltis, Nanobot, PicoClaw, ZeroClaw, NullClaw)
- ❌ Web UI (Nanobot, PicoClaw, ZeroClaw, NullClaw)

---

## 18. Development Activity

Git repository activity metrics as of 2026-03-06.

### Commit Activity

| Project | Language | Total Commits | Last 90d | Last 30d | Last 7d | First Commit | Last Commit |
|---------|----------|---------------|----------|----------|---------|--------------|-------------|
| **OpenClaw** | TypeScript | 17,089 | 16,488 | 8,179 | 1,737 | 2025-11-24 | 2026-03-06 |
| **LocalGPT** | Rust | 405 | 405 | 325 | 79 | 2026-02-01 | 2026-03-05 |
| **ZeroClaw** | Rust | 1,762 | 1,762 | 1,762 | 132 | 2026-02-13 | 2026-03-05 |
| **Moltis** | Rust | 1,472 | 1,472 | 1,153 | 134 | 2026-01-28 | 2026-03-06 |
| **NullClaw** | Zig | 990 | 990 | 990 | 465 | 2026-02-16 | 2026-03-05 |
| **Nanobot** | Python | 993 | 993 | 869 | 169 | 2026-02-01 | 2026-03-06 |
| **PicoClaw** | Go | 903 | 903 | 903 | 202 | 2026-02-09 | 2026-03-06 |
| **IronClaw** | Rust | 339 | 339 | 296 | 83 | 2026-02-02 | 2026-03-06 |
| **MimiClaw** | C (ESP32) | 181 | 181 | 176 | 22 | 2026-02-04 | 2026-03-06 |
| **CoPaw** | Python | 175 | 175 | 175 | 143 | 2026-02-27 | 2026-03-06 |

### Contributor Activity (Last 90 Days)

| Project | Active Contributors | Total Contributors | Commits/Contributor (90d) |
|---------|---------------------|-------------------|---------------------------|
| **OpenClaw** | 1,147 | 1,150 | 14.4 |
| **Nanobot** | 135 | 135 | 7.4 |
| **PicoClaw** | 144 | 144 | 6.3 |
| **ZeroClaw** | 158 | 158 | 11.2 |
| **NullClaw** | 49 | 49 | 20.2 |
| **CoPaw** | 47 | 47 | 3.7 |
| **IronClaw** | 37 | 37 | 9.2 |
| **LocalGPT** | 12 | 13 | 33.8 |
| **Moltis** | 14 | 14 | 105.1 |
| **MimiClaw** | 6 | 6 | 30.2 |

### Velocity Tiers

**Tier 1 — Hyperactive (>1000 commits/30d):**
- **OpenClaw** (8,179) — Reference implementation, massive community

**Tier 2 — Very Active (500-1000 commits/30d):**
- **ZeroClaw** (1,762) — Rapid development, large community
- **Moltis** (1,153) — Feature-rich Rust implementation
- **NullClaw** (990) — Zig upstart, fast growth
- **Nanobot** (869) — Python lightweight
- **PicoClaw** (903) — Go embedded
- **LocalGPT** (325) — Steady development, small focused team

**Tier 3 — Moderate (&lt;500 commits/30d):**
- **IronClaw** (296) — Security-focused Rust
- **MimiClaw** (176) — ESP32 embedded
- **CoPaw** (175) — Recent launch (Feb 27)

### Development Patterns

| Pattern | Projects | Notes |
|---------|----------|-------|
| **Community-driven** | OpenClaw, Nanobot, PicoClaw, ZeroClaw | 100+ contributors, distributed development |
| **Small team** | Moltis, MimiClaw, LocalGPT | &lt;15 contributors, concentrated development |
| **Corporate-backed** | OpenClaw, CoPaw | OpenClaw: established; CoPaw: Alibaba/AgentScope |
| **Solo/small founder** | MimiClaw, NullClaw | 6 contributors, focused vision |
| **Recent launches (Feb 2026)** | NullClaw, PicoClaw, ZeroClaw, CoPaw, MimiClaw, LocalGPT | New wave of implementations |

---

## Deviations & Unique Strengths

### IronClaw
1. **WASM sandbox** — Lighter weight than Docker, capability-based security
2. **NEAR AI focus** — Primary provider with session-based auth
3. **Tinfoil private inference** — Hardware-attested TEE provider
4. **PostgreSQL + libSQL** — Dual database backend
5. **Rich terminal UI** — rustyline + termimad with approval overlays

### LocalGPT
1. **Gen mode** — Bevy 3D scene generation + FunDSP procedural audio synthesis
2. **Bridge daemon architecture** — tarpc-based IPC for channel isolation (Telegram, Discord, WhatsApp)
3. **UniFFI mobile bindings** — Native iOS (Swift) + Android (Kotlin) from shared Rust core
4. **OS-level sandboxing** — Landlock (Linux) + Seatbelt (macOS) for process isolation without Docker
5. **Policy signing** — HMAC-SHA256 signed LocalGPT.md workspace security policies
6. **CLI-based providers** — Subprocess delegation to claude-cli, gemini-cli, codex-cli
7. **Desktop GUI** — Cross-platform egui/eframe application
8. **Profile isolation** — `--profile` flag for completely isolated config/data directories

### Moltis
1. **GraphQL API** — HTTP + WebSocket GraphQL in addition to RPC
2. **Voice I/O** — 5 TTS + 9 STT providers out-of-box (`moltis-voice`)
3. **Browser automation** — Chrome/Chromium via CDP (`moltis-browser`)
4. **Apple Container sandbox** — Native macOS container support alongside Docker
5. **WebAuthn/passkey auth** — Hardware-backed authentication
6. **Tailscale integration** — Serve + Funnel modes for network exposure
7. **A2UI Canvas** — Agent-controlled HTML UI for mobile/web
8. **17 hook event types** — Comprehensive lifecycle hooks with circuit breaker
9. **Zero unsafe code** — Workspace-level `deny(unsafe)` lint
10. **47-crate workspace** — Highly modular architecture

### Nanobot
1. **Ultra-lightweight Python** — ~4,000 lines of core code, minimal dependencies, fast to deploy
2. **Broadest channel support** — 9 messaging platforms + email (Telegram, Discord, Slack, WhatsApp, Feishu, QQ, DingTalk, Mochat, Email)
3. **Chinese provider ecosystem** — DashScope/Qwen, Moonshot/Kimi, MiniMax, Zhipu/GLM, SiliconFlow, VolcEngine, AiHubMix
4. **MCP integration** — stdio + HTTP transports for tool extensibility
5. **Two-layer memory** — MEMORY.md (long-term facts) + HISTORY.md (searchable log) with LLM-driven consolidation
6. **OAuth provider auth** — GitHub Copilot and OpenAI Codex via device OAuth flow
7. **Cron delivery routing** — Scheduled task results routed to specific messaging channels

### CoPaw
1. **AgentScope/Alibaba ecosystem** — Built by Alibaba's AgentScope team with enterprise focus
2. **Console web UI** — Full-featured browser-based management interface
3. **Chinese channel focus** — DingTalk, Feishu, QQ first-class support
4. **Twilio voice** — Voice call channel via Twilio
5. **Agent interruption API** — Ability to interrupt running agents
6. **i18n support** — Internationalization in web UI
7. **One-click install** — Windows one-click installation script

### PicoClaw
1. **Go-native ultra-lightweight** — &lt;10MB RAM, &lt;1s boot, single binary
2. **Multi-architecture** — RISC-V, ARM, MIPS, x86 from Sipeed
3. **$10 hardware target** — Designed for cheapest Linux boards
4. **WeCom support** — Enterprise WeChat (企业微信) channel
5. **MaixCam integration** — Embedded camera platform
6. **AI-bootstrapped development** — 95% agent-generated core code
7. **Antigravity provider** — Unique provider integration

### ZeroClaw
1. **Robot kit** — look/sense/drive/speak/listen/emote for physical robots
2. **ESP32 + Nucleo firmware** — Embedded hardware support
3. **MQTT channel** — IoT messaging protocol
4. **Matrix E2EE** — End-to-end encrypted Matrix support
5. **SOP system** — Standard Operating Procedures for repeatable workflows
6. **Goals system** — Goal tracking and management
7. **WASM sandbox** — wasmi runtime for sandboxed tool execution
8. **Telnyx integration** — SMS/voice via Telnyx
9. **Linq channel** — Unique messaging platform

### NullClaw
1. **Zig ultra-lightweight** — 678KB binary, ~1MB RAM, &lt;2ms boot (smallest)
2. **3,230+ tests** — Most comprehensive test coverage
3. **Multi-layer sandbox** — landlock, firejail, bubblewrap, docker options
4. **Composio integration** — Third-party tool integration platform
5. **Hardware peripherals** — I2C, SPI, screenshot tools
6. **True portability** — ARM, x86, RISC-V single binary
7. **$5 hardware target** — Cheapest possible deployment

### MimiClaw
1. **ESP32 bare metal** — No Linux, no Node.js, pure C on ESP-IDF
2. **$5 chip** — World's first AI assistant on a $5 chip
3. **Telegram-first** — Primary interface via Telegram bot
4. **Local flash memory** — All data stored on-chip, persists across reboots
5. **OTA updates** — Over-the-air firmware updates
6. **Serial CLI config** — Runtime configuration via serial interface
7. **Dual provider** — Supports both Anthropic (Claude) and OpenAI (GPT)
8. **0.5W power** — USB power, runs 24/7 on minimal energy
9. **Cron scheduling** — On-device cron for automated tasks

---

## Credits

- **IronClaw** ([ironclaw](https://github.com/nearai/ironclaw)) — Initial reference for this feature parity document. IronClaw's comprehensive feature matrix inspired the structure and categories used here.
