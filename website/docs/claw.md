---
sidebar_position: 9.1
slug: /claw
---

# Feature Parity Matrix — Claw Ecosystem

> **⚠️ AI-Generated Documentation:** This document was generated and is maintained by AI assistants. While efforts are made to ensure accuracy, many details must be outdated or incorrect as those projects are moving very fast. Please verify with the source repositories for the most current information.
>
> **Last updated:** 2026-03-22

This document tracks feature parity across fourteen implementations of the personal AI assistant architecture. OpenClaw (TypeScript) is the reference implementation; IronClaw, LocalGPT, Moltis, and ZeroClaw are Rust implementations; Nanobot, CoPaw, and Agent Zero are Python implementations; PicoClaw is Go; NullClaw is Zig; MimiClaw and ZClaw are C (ESP32); RosClaw is a TypeScript OpenClaw plugin for ROS2 robotics; TinyClaw is a TypeScript multi-agent orchestrator.

### Project Summary

| Project | Language | License | Summary |
|---------|----------|---------|---------|
| **OpenClaw** | TypeScript | MIT | Reference implementation (v2026.3.22); full-featured desktop AI assistant with 20+ messaging channels, WebSocket control plane, advanced hybrid memory system (multimodal embeddings, MMR, temporal decay, query expansion), ClawHub plugin ecosystem, and MCP integration |
| **IronClaw** | Rust | MIT/Apache 2.0 | Security-focused (v0.21.0); WASM sandbox with capability-based permissions, Docker sandbox (orchestrator/worker), prompt injection defense, hybrid search memory (PostgreSQL + pgvector), self-repair with fault injection testing, dynamic tool building, webhook relay, NEAR AI integration |
| **LocalGPT** | Rust | Apache 2.0 | Local-first AI assistant with persistent markdown memory, Bevy 3D scene generation (Gen mode with physics), Slack/Telegram/Discord/WhatsApp bridges, Docker/Podman + OS-level sandbox, browser automation, encryption at rest, and optional autonomous heartbeat |
| **Moltis** | Rust | MIT | Enterprise-ready 46-crate workspace (196K LoC, 2,300+ tests, zero unsafe); Docker + Apple Container sandbox, multi-channel (Telegram/Slack/HTTP/Teams/Discord), GraphQL API, TLS/WebAuthn auth, encryption-at-rest (XChaCha20-Poly1305) |
| **Nanobot** | Python | MIT | Lightweight ~4K LOC implementation with 10+ chat channels and MCP integration; ideal for learning and rapid prototyping |
| **CoPaw** | Python | Apache 2.0 | AgentScope/Alibaba-based (v0.1.0p1); native desktop installers (Win/Mac), console web UI with multimodal support, 10 channels (DingTalk/Feishu/QQ/Discord/iMessage/Telegram/Signal/Matrix/MQTT/Nostr), local models (llama.cpp/MLX/Ollama), ReMeLight memory, built-in glob/grep search tools, Twilio voice, daemon mode |
| **PicoClaw** | Go | MIT | Minimal footprint (under 10MB RAM) with multi-arch support; WeCom enterprise messaging; targets resource-constrained environments |
| **ZeroClaw** | Rust | MIT/Apache 2.0 | Ultra-lightweight (v0.5.8, &lt;5MB RAM, &lt;10ms startup); trait-driven architecture, Extism WASM plugin system, ClaudeCodeTool delegation, Piper TTS/LocalWhisper STT, Gmail Pub/Sub, Live Canvas, Prometheus metrics, ARM cross-compilation, extensive IoT protocol support (MQTT, Nextcloud Talk) |
| **NullClaw** | Zig | MIT | 678KB binary (v2026.3.21), 50+ AI providers, 35+ tools, 10 memory engines, 5,300+ tests; multi-layer sandbox (Landlock/Firejail/Bubblewrap/Docker/WASM3); NullHub ecosystem; Cron HTTP API; A2A protocol; I2C/SPI hardware; WeChat/WeCom; dual-backend SQLite + libSQL/Turso |
| **MimiClaw** | C (ESP32) | MIT | Bare metal ESP32-S3 ($5), Telegram primary channel, OTA updates, serial CLI config; no OS/runtime overhead |
| **RosClaw** | TypeScript + Python | Apache 2.0 | OpenClaw plugin for ROS2 robotics; rosbridge WebSocket, robot context injection, 3 transport modes |
| **ZClaw** | C (ESP32) | MIT | 888KiB binary budget (v2.13.0), 27 tools, 4 LLM providers, ESP32/C3/S3/C6 support, DHT/I2C sensor tools, rate limiting, serial admin |
| **TinyClaw** | TypeScript | MIT | Multi-agent team orchestrator (v0.0.16, rebranded to TinyAGI); TinyOffice dashboard (Next.js, 11 pages), Hono HTTP + REST API + WebSocket, SSE events with streaming execution progress, WhatsApp/Discord/Telegram integration, curl-based installer |
| **Agent Zero** | Python | MIT | General-purpose agentic framework with hierarchical multi-agent cooperation, Docker sandbox, browser automation, SKILL.md skills, MCP client/server, and real-time Web UI |

### GitHub Repositories

| Project | Repository |
|---------|------------|
| OpenClaw | https://github.com/openclaw/openclaw |
| IronClaw | https://github.com/nearai/ironclaw |
| LocalGPT | https://github.com/localgpt-app/localgpt |
| Moltis | https://github.com/moltis-org/moltis |
| Nanobot | https://github.com/HKUDS/nanobot |
| CoPaw | https://github.com/agentscope-ai/CoPaw |
| PicoClaw | https://github.com/sipeed/picoclaw |
| ZeroClaw | https://github.com/zeroclaw-labs/zeroclaw |
| NullClaw | https://github.com/nullclaw/nullclaw |
| MimiClaw | https://github.com/memovai/mimiclaw |
| RosClaw | https://github.com/PlaiPin/rosclaw |
| ZClaw | https://github.com/tnm/zclaw |
| TinyClaw | https://github.com/TinyAGI/tinyclaw |
| Agent Zero | https://github.com/agent0ai/agent-zero |

**Legend:**
- ✅ Implemented
- 🚧 Partial (in progress or incomplete)
- ❌ Not implemented

---

## 1. Architecture

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Hub-and-spoke architecture | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ✅ | ✅ | MimiClaw/ZClaw: embedded; RosClaw: OpenClaw plugin |
| WebSocket control plane | ✅ | ✅ | 🚧 | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | RosClaw: rosbridge WebSocket; Agent Zero: Flask-SocketIO |
| Single-user system | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Multi-agent routing | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | Agent Zero: hierarchical superior/subordinate; TinyClaw: multi-agent teams |
| Session-based messaging | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Loopback-first networking | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | |
| Bridge daemon protocol (IPC) | ➖ | ➖ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: tarpc-based localgpt-bridge |
| GraphQL API | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Moltis: HTTP + WebSocket GraphQL |
| Trait-driven architecture | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Ultra-lightweight runtime | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | |
| Embedded hardware support | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ZClaw: ESP32/C3/S3/C6 |
| OTA updates | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | MimiClaw + ZClaw: over-the-air firmware |
| No OS/runtime | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | Bare metal ESP-IDF/FreeRTOS |

---

## 2. Gateway System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Gateway control plane | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | TinyClaw: Hono HTTP; Agent Zero: Flask |
| HTTP endpoints for Control UI | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | TinyClaw: TinyOffice; Agent Zero: Web UI |
| Channel connection lifecycle | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | |
| Session management/routing | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | |
| Configuration hot-reload | ✅ | ❌ | ✅ | 🚧 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: env vars via A0_SET_ |
| Network modes (loopback/LAN/remote) | ✅ | 🚧 | 🚧 | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | RosClaw: 3 transport modes |
| OpenAI-compatible HTTP API | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Canvas hosting | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: Live Canvas |
| Gateway lock (PID-based) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| launchd/systemd integration | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | CoPaw: daemon mode; Agent Zero: Docker |
| Bonjour/mDNS discovery | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Tailscale integration | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Health check endpoints | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ZClaw: get_health tool |
| `doctor` diagnostics | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ZClaw: get_diagnostics |
| Agent event broadcast | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | TinyClaw: SSE; Agent Zero: WebSocket stream |
| Channel health monitor | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Presence system | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Trusted-proxy auth mode | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| APNs push pipeline | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Oversized payload guard | ✅ | 🚧 | ✅ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Pre-prompt context diagnostics | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | RosClaw: robot context injection; Agent Zero: system prompts |
| TLS/HTTPS auto-certs | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| WebAuthn/passkey auth | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Rate limiting (per-IP) | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ZClaw: 100/hr, 1000/day |
| Prometheus metrics | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Serial CLI config | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | MimiClaw + ZClaw: serial admin |

---

## 3. Messaging Channels

| Channel | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| CLI/TUI | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ✅ | ✅ | ✅ | - | ZClaw: serial; TinyClaw: tinyclaw send; Agent Zero: run_ui.py |
| HTTP webhook | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | 🚧 | ✅ | ✅ | - | ZClaw: web relay; Agent Zero: Flask API |
| REPL (simple) | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | |
| WASM channels | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | IronClaw innovation |
| WhatsApp | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ✅ | ❌ | P1 | TinyClaw: whatsapp-web.js |
| Telegram | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ✅ | ❌ | - | ZClaw: long-polling |
| Discord | ✅ | ❌ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ✅ | ❌ | P2 | TinyClaw: discord.js |
| Signal | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | P2 | CoPaw: v0.0.6 |
| Slack | ✅ | ✅ | ✅ | 🚧 | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | LocalGPT: bridge daemon, Socket Mode, slack-morphism v2, edit-in-place streaming, thread replies |
| iMessage | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| Linq | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | ZeroClaw only |
| Feishu/Lark | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ❌ | ❌ | ❌ | P3 | |
| LINE | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| WebChat | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ➖ | ❌ | ✅ | ✅ | - | TinyClaw: TinyOffice; Agent Zero: Web UI |
| Matrix | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | ZeroClaw: E2EE support; CoPaw: v0.0.6 |
| Mattermost | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Google Chat | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| MS Teams | ✅ | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| Twitch | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| Voice Call | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | CoPaw: Twilio voice |
| Nostr | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | CoPaw: v0.0.6 |
| QQ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| DingTalk | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Email (IMAP/SMTP) | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | ZeroClaw: also Gmail Pub/Sub push |
| IRC | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| WeCom (企业微信) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | PicoClaw only |
| MaixCam | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | Embedded camera |
| OneBot | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | QQ protocol |
| MQTT | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | ZeroClaw + CoPaw: IoT messaging |
| Nextcloud Talk | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | ZeroClaw only |
| WATI (WhatsApp Business) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | ZeroClaw only |
| Serial/USB | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | - | Embedded device serial |
| Web Relay | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 🚧 | ❌ | ❌ | - | ZClaw: HTTP relay gateway |

### Telegram-Specific Features (since Feb 2025)

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Forum topic creation | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: /topic command + thread routing |
| channel_post support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| User message reactions | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: ack reactions |
| sendPoll | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Cron/heartbeat topic targeting | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Streaming message edits | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | PicoClaw: sendMessageDraft |

### Discord-Specific Features (since Feb 2025)

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Forwarded attachment downloads | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Faster reaction state machine | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thread parent binding inheritance | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |

### Slack-Specific Features (since Feb 2025)

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Streaming draft replies | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Configurable stream modes | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thread ownership | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |

### Channel Features

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| DM pairing codes | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Allowlist/blocklist | ✅ | 🚧 | 🚧 | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Self-message bypass | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Mention-based activation | ✅ | ✅ | ✅ | 🚧 | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Per-group tool policies | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thread isolation | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: context per agent |
| Per-channel media limits | ✅ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Typing indicators | ✅ | 🚧 | ✅ | 🚧 | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Per-channel ackReaction config | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Group session priming | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Sender_id in trusted metadata | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |

---

## 4. CLI Commands

| Command | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| `run` (agent) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | - | MimiClaw: always-on embedded; ZClaw: always-on; Agent Zero: run_ui.py |
| `tool install/list/remove` | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: `localgpt tool list/add/remove` for MCP servers |
| `gateway start/stop` | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | P2 | Agent Zero: Docker run |
| `onboard` (wizard) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: initialize.py |
| `tui` | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | - | CoPaw: Console web UI; Agent Zero: Web UI |
| `config` | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | - | MimiClaw: serial CLI; Agent Zero: env vars A0_SET_ |
| `channels` | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `models` | ✅ | 🚧 | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | - | MimiClaw: switch provider at runtime; Agent Zero: settings UI |
| `status` | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | - | |
| `agents` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | P3 | Agent Zero: context management |
| `sessions` | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | P3 | Agent Zero: chat load/save |
| `memory` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | - | MimiClaw: local flash storage; Agent Zero: knowledge_tool |
| `skills` | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ | ✅ | - | MimiClaw: on-device skills; Agent Zero: SKILL.md standard |
| `pairing` | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | - | |
| `nodes` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `plugins` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `hooks` | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `cron` | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | P2 | MimiClaw: on-device cron; Agent Zero: scheduler tool |
| `webhooks` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `message send` | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | P2 | |
| `browser` | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | Agent Zero: browser_agent tool |
| `sandbox` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: Docker container |
| `doctor` | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | P2 | ZClaw: diagnostics |
| `logs` | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | P3 | Agent Zero: logs/ folder |
| `update` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ✅ | P3 | MimiClaw: OTA updates; ZClaw: OTA; Agent Zero: Docker pull |
| `completion` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `/subagents spawn` | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| `/export-session` | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | Agent Zero: chat export |
| `auth` (OAuth management) | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `desktop` (GUI) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | - | LocalGPT: egui/eframe; Agent Zero: Web UI |
| `db` (database management) | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `tailscale` | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `md sign/verify/policy` | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `bridge list/show/remove` | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| `hardware` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | - | MimiClaw: ESP32 GPIO |
| `goals` | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: goals system |
| `sop` (Standard Operating Procedures) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: sop_execute/list/approve/status |
| `ota` (over-the-air update) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | - | MimiClaw + ZClaw |

---

## 5. Agent System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Pi agent runtime | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | All Rust/Go/Zig/C impls use custom runtimes; Agent Zero: Python |
| RPC-based execution | ✅ | ✅ | 🚧 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: tarpc IPC for bridge daemons |
| Multi-provider failover | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ✅ | ✅ | MimiClaw: Anthropic + OpenAI switchable; Agent Zero: LiteLLM; RosClaw: via OpenClaw |
| Per-sender sessions | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ✅ | ✅ | RosClaw: via OpenClaw; Agent Zero: context per chat |
| Global sessions | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Session pruning | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | Agent Zero: context management |
| Context compaction | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ❌ | ✅ | RosClaw: via OpenClaw; Agent Zero: history truncation |
| Post-compaction read audit | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Post-compaction context injection | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: AGENTS.md injection |
| Custom system prompts | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ✅ | ✅ | RosClaw: via OpenClaw; Agent Zero: prompts/ folder |
| Skills (modular capabilities) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ➖ | ✅ | ✅ | ✅ | MimiClaw: on-device skills; Agent Zero: SKILL.md standard; RosClaw: via OpenClaw |
| Skill routing blocks | ✅ | 🚧 | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Skill path compaction | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Thinking modes (low/med/high) | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | CoPaw: optional thinking display; Agent Zero: reasoning tags |
| Per-model thinkingDefault override | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Block-level streaming | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: real-time Web UI stream |
| Tool-level streaming | ✅ | ❌ | 🚧 | 🚧 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | CoPaw: optional tool call display; Agent Zero: tool output streaming |
| Z.AI tool_stream | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Plugin tools | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | All: MCP tools; IronClaw: WASM; Agent Zero: python/tools/; RosClaw: ROS2 tools |
| Tool policies (allow/deny) | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | RosClaw: safety policies |
| Exec approvals (`/approve`) | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Elevated mode | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Subagent support | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | Agent Zero: hierarchical superior/subordinate; TinyClaw: multi-agent teams |
| `/subagents spawn` command | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Auth profiles | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: per-project secrets |
| Generic API key rotation | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Stuck loop detection | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: intervention handling |
| Self-repair / stuck recovery | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | IronClaw: iterative build; LocalGPT: tool error spiral + orphan recovery |
| llms.txt discovery | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Multiple images per tool call | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: vision_load tool |
| URL allowlist (web_search/fetch) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| suppressToolErrors config | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Intent-first tool display | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Transcript file size in status | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Session branching | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Moltis: `branch_session` tool; LocalGPT: CLI `session branch` |
| Agent interruption API | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | CoPaw: v0.0.5; Agent Zero: pause/intervene |
| Delegate tool | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: call_subordinate; Route to specialized subagents |
| SOP execution | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: Standard Operating Procedures |
| On-device agent loop | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | MimiClaw + ZClaw: ESP32 agent loop |

---

## 6. Model & Provider Support

| Provider | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|----------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| NEAR AI | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | - | RosClaw: via OpenClaw |
| Anthropic (Claude) | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ✅ | ✅ | - | RosClaw: via OpenClaw; TinyClaw: Claude CLI; Agent Zero: LiteLLM |
| OpenAI | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ➖ | ✅ | ✅ | ✅ | - | RosClaw: via OpenClaw; TinyClaw: Codex CLI; Agent Zero: LiteLLM |
| AWS Bedrock | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| GCP Vertex AI | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | OpenClaw: Claude via Vertex |
| Google Gemini | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | P3 | Agent Zero: LiteLLM |
| NVIDIA API | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| OpenRouter | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ✅ | ❌ | ✅ | - | Agent Zero: LiteLLM |
| Tinfoil | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | - | IronClaw-only |
| OpenAI-compatible | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ✅ | ✅ | - | TinyClaw: custom providers; Agent Zero: LiteLLM |
| Ollama (local) | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ✅ | ❌ | ✅ | - | Agent Zero: LiteLLM |
| Perplexity | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| MiniMax | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | P3 | |
| GLM-5 | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | P3 | Agent Zero: LiteLLM |
| node-llama-cpp | ✅ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | - | N/A for Rust/Go/Zig/C |
| llama.cpp (native) | ❌ | 🔮 | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| X.AI (Grok) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ✅ | - | Agent Zero: LiteLLM |
| GitHub Copilot | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| CLI-based providers (subprocess) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | - | PicoClaw: claude-cli, codex-cli; TinyClaw: CLI-based |
| Kimi/Moonshot | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | |
| DeepSeek | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | - | CoPaw: DeepSeek Reasoner; Agent Zero: LiteLLM |
| Groq | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | - | Agent Zero: LiteLLM |
| DashScope/Qwen | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | |
| VolcEngine | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | |
| SiliconFlow | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | |
| AiHubMix | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | - | |
| OpenAI Codex (OAuth) | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| vLLM | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: LiteLLM |
| Antigravity | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | PicoClaw only |
| Telnyx | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: SMS/voice |
| Novita AI | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | NullClaw + PicoClaw: OpenAI-compatible |
| Xiaomi MiMo | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | - | NullClaw: Xiaomi MiMo V2 |
| DeepMyst/Avian | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: new providers |
| Bailian (Aliyun) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: Alibaba cloud |

### Model Features

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Auto-discovery | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Failover chains | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | MimiClaw: Anthropic ↔ OpenAI; Agent Zero: rate limiter |
| Cooldown management | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: RateLimiter class |
| Per-session model override | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | MimiClaw: runtime switch; Agent Zero: settings UI |
| Model selection UI | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: Web UI settings |
| Per-model thinkingDefault | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| 1M context beta header | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Provider-native tool definitions | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Provider aliases | ❌ | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: LiteLLM aliases |
| Model routing config | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: model_routing_config tool |

---

## 7. Media Handling

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| Image processing (Sharp) | ✅ | ❌ | 🚧 | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | 🚧 | ❌ | 🚧 | ✅ | P2 | RosClaw: camera snapshot; Agent Zero: vision_load; TinyClaw: image processing |
| Configurable image resize dims | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | Agent Zero: MAX_PIXELS config |
| Multiple images per tool call | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | Agent Zero: vision_load paths list |
| Audio transcription | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | LocalGPT: transcribe_audio tool; CoPaw: Twilio voice; Agent Zero: STT; ZeroClaw: LocalWhisper STT |
| Video support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| PDF parsing | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | ZeroClaw: pdf_read tool; Agent Zero: document_query |
| MIME detection | ✅ | ❌ | 🚧 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | Agent Zero: mimetypes |
| Media caching | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Vision model integration | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | Agent Zero: vision-capable models via LiteLLM |
| TTS (Edge TTS) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | Agent Zero: built-in TTS |
| TTS (OpenAI) | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | Moltis: 5 providers; Agent Zero: multiple TTS |
| Incremental TTS playback | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | Agent Zero: streaming TTS |
| Sticker-to-image | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Procedural audio synthesis | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: FunDSP in Gen mode |
| STT (multiple providers) | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | - | ZeroClaw: LocalWhisper self-hosted; Agent Zero: speech-to-text |
| Web content extraction | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: browser_agent + document_query |
| Screenshot capture | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | - | ZeroClaw/NullClaw: screenshot tool; Agent Zero: browser screenshots |

---

## 8. Plugin & Extension System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Dynamic loading | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | Agent Zero: python/tools/ |
| Manifest validation | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| HTTP path registration | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: Flask routes |
| Workspace-relative install | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | Agent Zero: projects/ |
| Channel plugins | ✅ | ✅ | 🚧 | 🚧 | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Auth plugins | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Memory plugins | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Tool plugins | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | Agent Zero: python/tools/*.py |
| Hook plugins | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | Agent Zero: python/extensions/ |
| Provider plugins | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: LiteLLM providers |
| Plugin CLI (`install`, `list`) | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| ClawHub registry | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| `before_agent_start` hook | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | Agent Zero: agent_init extensions |
| `before_message_write` hook | ✅ | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: user_message_ui extensions |
| `llm_input`/`llm_output` hooks | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | RosClaw: before_tool_call |
| MCP support (stdio + HTTP/SSE) | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: MCP client + server |
| Browser automation (CDP) | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | LocalGPT: Chrome DevTools Protocol; Agent Zero: Playwright |
| Composio integration | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | NullClaw: composio tool |
| WASM module tools | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: Extism WASM plugin system (WasmTool, WasmChannel) |

---

## 9. Configuration System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Primary config file | ✅ `openclaw.json` | ✅ `.env` | ✅ `config.toml` | ✅ `moltis.toml` | ✅ `config.json` | ✅ `config.yaml` | ✅ `config.yaml` | ✅ `config.toml` | ✅ `config.json` | ❌ | ❌ | ✅ NVS | ✅ `settings.json` | ✅ `.env` + UI | Agent Zero: dotenv + Web UI |
| JSON5 support | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| YAML alternative | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Environment variable interpolation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | Agent Zero: A0_SET_ prefix |
| Config validation/schema | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | Agent Zero: pydantic |
| Hot-reload | ✅ | ❌ | ✅ | 🚧 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: env reload |
| Legacy migration | ✅ | ➖ | ➖ | ➖ | ➖ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: migration.rs |
| State directory | ✅ `~/.openclaw-state/` | ✅ `~/.ironclaw/` | ✅ XDG dirs | ✅ `~/.moltis/` | ✅ `~/.nanobot/` | ✅ `~/.copaw/` | ✅ `~/.picoclaw/` | ✅ `~/.zeroclaw/` | ✅ `~/.nullclaw/` | ❌ | ❌ | ✅ NVS flash | ✅ `~/.tinyclaw/` | ✅ `work_dir/` | Agent Zero: configurable work_dir |
| Credentials directory | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ NVS | ✅ | ✅ | Agent Zero: secrets manager; ZeroClaw: encrypted with chacha20poly1305 |
| Full model compat fields in schema | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Profile support | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: agent profiles |
| JSON Schema export | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: schemars |

---

## 10. Memory & Knowledge System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Vector memory | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | Agent Zero: FAISS; RosClaw: via OpenClaw |
| Session-based memory | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ✅ | ✅ | ✅ | Agent Zero: memory_load/save; RosClaw: via OpenClaw |
| Hybrid search (BM25 + vector) | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | Agent Zero: mem_search_enhanced |
| Temporal decay (hybrid search) | ✅ | ❌ | ✅ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| MMR re-ranking | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| LLM-based query expansion | ✅ | ❌ | ✅ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: local + LLM expansion |
| OpenAI embeddings | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | Agent Zero: sentence-transformers |
| Gemini embeddings | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | OpenClaw + LocalGPT: Gemini embedding providers |
| Gemini multimodal embeddings | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | OpenClaw + LocalGPT: image + audio indexing via Gemini |
| Voyage AI embeddings | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Mistral embeddings | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Ollama embeddings | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Local embeddings | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: sentence-transformers local |
| SQLite-vec backend | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | |
| LanceDB backend | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| QMD backend | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Multiple memory engines | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | NullClaw: 10 engines (SQLite, Markdown, ClickHouse, PostgreSQL, Redis, LanceDB, Lucid, LRU, API, None) |
| Atomic reindexing | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | |
| Embeddings batching | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | ❌ | ❌ | ❌ | OpenClaw: batch-openai, batch-gemini, batch-voyage |
| Citation support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | OpenClaw: on/off/auto modes per chat type |
| Session memory indexing | 🚧 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | OpenClaw: experimental; LocalGPT: session transcripts indexed for search |
| Post-compaction memory sync | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | OpenClaw: forced sync after compaction |
| memory_get tool (snippet read) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | OpenClaw: path + from line + line count |
| Memory CLI commands | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Flexible path structure | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | Agent Zero: memory_subdir per context |
| Identity files (AGENTS.md, etc.) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ➖ | ❌ | ❌ | ✅ | Agent Zero: system prompts |
| Daily logs | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Heartbeat checklist | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ✅ | ✅ | ❌ | RosClaw: via OpenClaw |
| File watcher (workspace changes) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Search result caching | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Two-layer memory (facts + history) | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: Memory.Area.MAIN/FRAGMENTS/SOLUTIONS |
| RAG system | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: knowledge_tool + document_query; ZeroClaw: rag crate |
| Memory store/recall/forget tools | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ✅ | ❌ | ✅ | Agent Zero: memory_save/load/delete/forget; RosClaw: via OpenClaw |

---

## 11. Mobile Apps

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| iOS app (SwiftUI) | ✅ | 🚫 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: UniFFI + XCFramework |
| Android app (Kotlin) | ✅ | 🚫 | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | LocalGPT: UniFFI + cargo-ndk |
| Apple Watch companion | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Gateway WebSocket client | ✅ | 🚫 | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: WebSocket in browser |
| Camera/photo access | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: vision_load via browser |
| Voice input | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: STT via browser |
| Push-to-talk | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Location sharing | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Node pairing | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| APNs push notifications | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Share to OpenClaw (iOS) | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Background listening toggle | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| UniFFI mobile bindings | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| PWA (Progressive Web App) | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: Docker Web UI; CoPaw: Console web UI |
| ESP32 firmware | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: zeroclaw-esp32 |
| Nucleo firmware | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: zeroclaw-nucleo |
| MaixCam support | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | Embedded camera platform |

---

## 12. macOS / Desktop App

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| SwiftUI native app | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Menu bar presence | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Bundled gateway | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: Docker bundled |
| Canvas hosting | ✅ | 🚫 | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Voice wake | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Voice wake overlay | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Push-to-talk hotkey | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Exec approval dialogs | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| iMessage integration | ✅ | 🚫 | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Instances tab | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: context management |
| Agent events debug window | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: logs/ folder |
| Sparkle auto-updates | ✅ | 🚫 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: Docker pull |
| Cross-platform desktop GUI | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | LocalGPT: egui; CoPaw: Console web UI; Agent Zero: Web UI |
| Robot kit | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw: look/sense/drive/speak/listen/emote |

---

## 13. Web Interface

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| Control UI Dashboard | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | - | Agent Zero: Flask Web UI; CoPaw: Console web UI; TinyClaw: TinyOffice |
| Channel status view | ✅ | 🚧 | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | - | LocalGPT: /api/channels/status + web UI panel; Agent Zero: context list |
| Agent management | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | P3 | Agent Zero: context management |
| Model selection | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: settings UI |
| Config editing | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | P3 | LocalGPT: POST /api/config + settings panel; Agent Zero: settings UI |
| Debug/logs viewer | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | - | Agent Zero: logs/ folder |
| WebChat interface | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | - | Agent Zero: main chat UI |
| Canvas system (A2UI) | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | ZeroClaw: Live Canvas |
| Control UI i18n | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | CoPaw: i18n support |
| WebChat theme sync | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | Agent Zero: dark/light mode |
| Partial output on abort | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | LocalGPT: streaming abort handling; Agent Zero: intervention handling |
| GraphQL playground | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Session sharing via URL | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Version update notifications | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | - | Agent Zero: update_check extension; CoPaw: v0.0.5 |

---

## 14. Automation

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Priority | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|----------|-------|
| Cron jobs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | - | Agent Zero: scheduler tool |
| Cron stagger controls | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Cron finished-run webhook | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Timezone support | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | - | Agent Zero: parse_datetime |
| One-shot/recurring jobs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | - | Agent Zero: AdHocTask/PlannedTask |
| Channel health monitor | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `beforeInbound` hook | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `beforeOutbound` hook | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `beforeToolCall` hook | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | P2 | |
| `before_agent_start` hook | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | P2 | Agent Zero: agent_init extensions |
| `before_message_write` hook | ✅ | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | Agent Zero: user_message_ui extensions |
| `onMessage` hook | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | - | |
| `onSessionStart` hook | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `onSessionEnd` hook | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `transcribeAudio` hook | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P3 | Agent Zero: STT integration |
| `transformResponse` hook | ✅ | ✅ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| `llm_input`/`llm_output` hooks | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Bundled hooks | ✅ | ✅ | 🚧 | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | P2 | Agent Zero: python/extensions/ |
| Plugin hooks | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Workspace hooks | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| Outbound webhooks | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P2 | |
| Heartbeat system | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ❌ | - | |
| Gmail pub/sub | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | P3 | |
| Cron delivery routing | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | |
| Pushover notifications | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | - | ZeroClaw/NullClaw: pushover tool |

---

## 15. Security Features

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Gateway token auth | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Agent Zero: no auth by default |
| Device pairing | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Tailscale identity | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Trusted-proxy auth | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| OAuth flows | ✅ | 🚧 | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| DM pairing verification | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Allowlist/blocklist | ✅ | 🚧 | 🚧 | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ZClaw: Telegram chat IDs; TinyClaw: pairing system |
| Per-group tool policies | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Exec approvals | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | Agent Zero: intervention handling |
| TLS 1.3 minimum | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | |
| SSRF protection | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ❌ | |
| SSRF IPv6 transition bypass block | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Cron webhook SSRF guard | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Loopback-first | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ✅ | ✅ | ✅ | Agent Zero: localhost default |
| Docker sandbox | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | LocalGPT: DockerSandbox backend (cap-drop ALL, network none, mem 512m); Agent Zero: recommended |
| Podman support | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | LocalGPT: auto-detects docker/podman; Agent Zero: Podman compatible |
| WASM sandbox | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: wasmi |
| Sandbox env sanitization | ✅ | 🚧 | ✅ | 🚧 | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Tool policies | ✅ | ✅ | ✅ | ✅ | 🚧 | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | RosClaw: safety policies; ZClaw: GPIO safe range |
| Elevated mode | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Safe bins allowlist | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| LD*/DYLD* validation | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Path traversal prevention | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | Agent Zero: path validation |
| Credential theft via env injection | ✅ | 🚧 | ✅ | 🚧 | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: secrets manager |
| Session file permissions (0o600) | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Skill download path restriction | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: skills/ folder |
| Webhook signature verification | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: HMAC-SHA256 |
| Media URL validation | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Prompt injection defense | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ❌ | ➖ | ❌ | ❌ | ✅ | Agent Zero: system prompt isolation |
| Leak detection | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | Agent Zero: key redaction; ZClaw: key redaction |
| Dangerous tool re-enable warning | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| OS-level sandbox (Landlock/Seatbelt) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | NullClaw: landlock, firejail, bubblewrap |
| Policy signing (HMAC-SHA256) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| WebAuthn/passkey auth | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Apple Container sandbox | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Zero unsafe code | ❌ | ❌ | ❌ | ✅ | ➖ | ➖ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ➖ | N/A for Python |
| WebSocket origin validation | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Encrypted secrets storage | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ | ✅ | LocalGPT: encryption at rest for sessions/config; ZeroClaw/NullClaw: chacha20poly1305 AEAD; ZClaw: NVS encryption; Agent Zero: secrets manager |

---

## 16. Development & Build System

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| Primary language | TypeScript | Rust | Rust | Rust | Python | Python | Go | Rust | Zig | C (ESP-IDF) | TypeScript | C (ESP-IDF) | TypeScript | Python | |
| Build tool | tsdown | cargo | cargo | cargo | pip/uv | pip/uv | go build | cargo | zig build | idf.py | pnpm | idf.py | pnpm | pip/uv | |
| Type checking | TypeScript/tsgo | rustc | rustc | rustc | ❌ | ❌ | ❌ | rustc | Zig | ❌ | TypeScript/tsgo | ❌ | TypeScript | ❌ | |
| Linting | Oxlint | clippy | clippy | clippy | ❌ | black/ruff | ❌ | clippy | Zig | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Formatting | Oxfmt | rustfmt | rustfmt | rustfmt | ❌ | black | gofmt | rustfmt | zig fmt | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Package manager | pnpm | cargo | cargo | cargo | pip/uv | pip/uv | go mod | cargo | zig | ESP-IDF | pnpm | ESP-IDF | pnpm | pip/uv | |
| Test framework | Vitest | built-in | built-in | built-in | ❌ | pytest | built-in | built-in | built-in | ❌ | Vitest | ❌ | ❌ | pytest | ZClaw: host tests only |
| Coverage | V8 | tarpaulin/llvm-cov | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| CI/CD | GitHub Actions | GitHub Actions | GitHub Actions | GitHub Actions | ❌ | GitHub Actions | GitHub Actions | GitHub Actions | GitHub Actions | GitHub Actions | ❌ | ✅ | ❌ | ❌ | ZClaw: GitHub Actions |
| Pre-commit hooks | prek | - | - | - | - | - | - | - | - | - | - | - | - | - | |
| Docker: Chromium + Xvfb | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: browser agent |
| Docker: init scripts | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | |
| Browser: extraArgs config | ✅ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Workspace crate count | ➖ | 1 | 14 | 46 | ➖ | ➖ | ➖ | 2 | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | LocalGPT: 14 crates (added bridges/slack); Moltis: 46 modular crates |
| Mobile build scripts | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ZeroClaw: ESP32/Nucleo firmware |
| Nix/direnv support | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| mdBook documentation | ❌ | ❌ | ❌ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Rust edition | ➖ | ➖ | 2024 | 2024 | ➖ | ➖ | ➖ | 2021 | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | |
| Go version | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | 1.21+ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | |
| Zig version | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | 0.15.2 | ➖ | ➖ | ➖ | ➖ | ➖ | |
| ESP-IDF version | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | 5.5+ | ➖ | 5.5+ | ➖ | ➖ | MimiClaw + ZClaw |
| Node.js version | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | ➖ | 18+ | ➖ | TinyClaw only |
| Docker multi-arch | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Lightweight profile | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | NullClaw: ReleaseSmall |
| Docker support | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | Agent Zero: recommended |
| Systemd service docs | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Homebrew package | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Binary size (release) | ~28 MB | ~44 MB | ~15 MB | ~large | N/A | N/A | ~8 MB | ~3.4 MB | ~678 KB | ~firmware | ~28 MB | ~firmware | N/A | N/A | RosClaw: via OpenClaw |
| RAM footprint | &gt;1 GB | ~large | ~moderate | ~large | &gt;100 MB | ~moderate | &lt;10 MB | &lt;5 MB | ~1 MB | ~512 KB | &gt;1 GB | ~512 KB | &gt;100 MB | ~moderate | RosClaw/ZClaw: via OpenClaw/ESP32 |
| Startup time (0.8 GHz) | &gt;500 s | ~fast | ~fast | ~fast | &gt;30 s | ~fast | &lt;1 s | &lt;10 ms | &lt;8 ms | instant | &gt;500 s | instant | &gt;5 s | ~fast | RosClaw: via OpenClaw |
| Power consumption | ~100 W | ~moderate | ~moderate | ~moderate | ~moderate | ~moderate | &lt;5 W | &lt;5 W | &lt;1 W | 0.5 W | ~100 W | 0.5 W | ~moderate | ~moderate | RosClaw: via OpenClaw |
| Target hardware | Mac/PC | Mac/PC | Mac/PC | Mac/PC | Linux SBC | Mac/PC | $10 board | $10 board | $5 board | $5 ESP32-S3 | Mac/PC | $5 ESP32 | Mac/PC | Mac/PC | |

---

## 17. Gen Mode / Explorable Worlds

| Feature | OpenClaw | IronClaw | LocalGPT | Moltis | Nanobot | CoPaw | PicoClaw | ZeroClaw | NullClaw | MimiClaw | RosClaw | ZClaw | TinyClaw | Agent Zero | Notes |
|---------|----------|----------|----------|--------|---------|-------|----------|----------|----------|----------|---------|-------|----------|------------|-------|
| 3D rendering engine | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | LocalGPT: Bevy 0.18 |
| glTF/GLB scene loading | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Entity spawning (11 primitives) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Cuboid, sphere, cylinder, cone, capsule, torus, plane, pyramid, tetrahedron, icosahedron, wedge |
| Batch entity operations | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | spawn/modify/delete batch |
| PBR materials & lighting | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Metallic/roughness/emissive + directional/point/spot lights |
| World skills (save/load/export) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | RON format with inline entities |
| Behavior system (7 types) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Orbit, spin, bob, look_at, pulse, path_follow, bounce |
| Guided tours | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Waypoints with walk/fly/teleport modes |
| Avatar/player control | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | WASD + mouse, PoV switching; physics TBD |
| HTML/Three.js export | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Browser-playable worlds |
| MCP server for gen tools | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | External clients drive scenes |
| Undo/redo | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Procedural audio (FunDSP) | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 7 ambient + 5 emitter types |
| Spatial audio & emitters | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Distance attenuation, auto-inference |
| Character/NPC system | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Spawn, patrol, wander, dialogue |
| Interaction triggers | ❌ | ❌ | 🚧 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | Proximity, click, area, timer |
| Headless/remote control mode | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | |
| Hardware peripherals (I2C, SPI, GPIO) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | RosClaw: via ROS2 topics; ZClaw: GPIO + I2C |
| ROS2 robot control | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | ❌ | RosClaw: 8 ROS2 tools, 3 transport modes |

---

## Implementation Priorities

### P0 - Core (Already Done)

**All implementations:**
- ✅ Session management + context compaction
- ✅ Heartbeat system
- ✅ Custom system prompts + skills
- ✅ Subagent support
- ✅ Multi-provider LLM

**IronClaw additionally (v0.21.0):**
- ✅ TUI (rustyline + termimad) + HTTP webhook + WASM sandbox
- ✅ Web Control UI + WebChat + DM pairing + unified settings page
- ✅ Gateway control plane + WebSocket + webhook relay events
- ✅ Docker sandbox + cron scheduling
- ✅ Tinfoil private inference
- ✅ Self-repair with FaultInjector testing framework + LRU embedding cache

**LocalGPT additionally:**
- ✅ CLI chat + HTTP server + web UI (config editing + channel status panels)
- ✅ Telegram/Discord/WhatsApp/Slack bridges (tarpc IPC)
- ✅ iOS/Android via UniFFI
- ✅ Gen mode (Bevy 3D + FunDSP audio + avian3d physics + world forking)
- ✅ Multi-layer sandbox: OS-level (Landlock/Seatbelt) + Docker/Podman containers
- ✅ Encryption at rest (sessions + config secrets)
- ✅ Browser automation (Chrome DevTools Protocol)
- ✅ Session transcript indexing + post-compaction context injection
- ✅ Gemini multimodal embeddings + LLM query expansion
- ✅ OAuth for 4+ providers + OpenRouter
- ✅ Desktop GUI (egui)
- ✅ OpenAI-compatible HTTP API (`/v1/chat/completions`, `/v1/models`)
- ✅ MCP support (stdio + HTTP/SSE) + `localgpt tool` CLI management
- ✅ Cron scheduling + lifecycle hooks
- ✅ Multi-provider failover + rate limiting + gateway auth + webhook HMAC-SHA256
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
- ✅ 15+ LLM providers (strong Chinese ecosystem) + Anthropic prompt cache optimization
- ✅ Native multimodal tool perception (autonomous image/media processing)
- ✅ Interactive onboard wizard with model autocomplete and field hints
- ✅ /status command (model, channels, MCP tools, uptime)
- ✅ MCP support (stdio + HTTP) + schema normalization for OpenAI-compatible
- ✅ Cron with delivery routing + run history tracking
- ✅ OAuth for GitHub Copilot + OpenAI Codex
- ✅ Two-layer memory (MEMORY.md + HISTORY.md)

**CoPaw additionally (v0.1.0p1):**
- ✅ Native desktop installers (Windows/macOS one-click setup)
- ✅ Console web UI with multimodal support (image/file uploads, ModelScope Studio)
- ✅ 10+ channels: DingTalk, Feishu, QQ, Discord, iMessage, Telegram, Signal, Matrix, MQTT, Nostr, WeChat/WeCom QR
- ✅ Built-in glob_search and grep_search tools for codebase exploration
- ✅ Skills zip upload for importing skills
- ✅ Local model support (llama.cpp, MLX, Ollama)
- ✅ ReMeLight memory system with smart truncation
- ✅ MCP support (stdio + HTTP)
- ✅ Cron with delivery routing
- ✅ Twilio voice channel + Feishu voice processing
- ✅ Daemon mode + DaemonAgent autonomous diagnostics
- ✅ Agent interruption API
- ✅ Russian + Japanese language support

**PicoClaw additionally:**
- ✅ Ultra-lightweight Go binary (&lt;10MB RAM, &lt;1s boot)
- ✅ SubTurn evaluator-optimizer system with token budget tracking
- ✅ TUI launcher with cyberpunk theme, config/gateway management
- ✅ Event bus + hook manager foundation
- ✅ Multi-arch: RISC-V, ARM, MIPS, x86
- ✅ 10+ channels including WeCom, MaixCam, OneBot + Telegram streaming
- ✅ CLI-based providers (claude-cli, codex-cli) + multiple API key failover
- ✅ I2C hardware support
- ✅ MCP support (deferred mode per server)

**ZeroClaw additionally (v0.5.8):**
- ✅ Ultra-lightweight Rust binary (&lt;5MB RAM) with ARM cross-compilation
- ✅ 26+ channels including MQTT, Nextcloud Talk, Linq, Slack, Gmail Pub/Sub
- ✅ Matrix E2EE support with room-level gating
- ✅ Extism WASM plugin system (WasmTool, WasmChannel bridges)
- ✅ ClaudeCodeTool for two-tier agent delegation
- ✅ Local TTS (Piper) + Local STT (LocalWhisper)
- ✅ Live Canvas gateway feature
- ✅ Voice wake word detection
- ✅ Image generation via fal.ai
- ✅ Robot kit (look/sense/drive/speak/listen/emote) + RPi GPIO, Aardvark I2C/SPI
- ✅ SOP (Standard Operating Procedures) + autonomous skill creation
- ✅ PostgreSQL memory backend (alongside SQLite)
- ✅ RAG system
- ✅ ESP32/Nucleo firmware
- ✅ Encrypted secrets (chacha20poly1305)

**NullClaw additionally (v2026.3.21):**
- ✅ Ultra-lightweight Zig binary (678KB, ~1MB RAM, &lt;2ms boot)
- ✅ 19 channels + 50+ providers + 35+ tools + WeChat/WeCom
- ✅ NullHub ecosystem branding and documentation
- ✅ 10 memory engines (SQLite hybrid, Markdown, ClickHouse, PostgreSQL, Redis, LanceDB, Lucid, LRU, API, None)
- ✅ Hybrid vector+FTS5 memory with Reciprocal Rank Fusion
- ✅ Multi-layer sandbox (Landlock, Firejail, Bubblewrap, Docker, WASM3)
- ✅ A2A protocol v0.3.0 with task states and history
- ✅ Cron HTTP API with session target routing
- ✅ Dual-backend persistence (SQLite + libSQL/Turso)
- ✅ Hardware peripherals (I2C, SPI)
- ✅ Composio integration
- ✅ 5,300+ tests (~250 source files)

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

**RosClaw additionally:**
- ✅ OpenClaw plugin for ROS2 robot control
- ✅ 3 transport modes (rosbridge WebSocket, local DDS, WebRTC)
- ✅ 8 ROS2 tools (publish, subscribe, service call, action goal, params, list topics, camera)
- ✅ Safety validator (velocity limits, workspace boundaries)
- ✅ Emergency stop (/estop) command
- ✅ Robot capability auto-discovery with caching
- ✅ before_agent_start context injection

**ZClaw additionally (v2.13.0):**
- ✅ Ultra-minimalist ESP32 AI assistant (888 KiB firmware budget)
- ✅ 4 LLM providers (Anthropic, OpenAI, OpenRouter, Ollama)
- ✅ 27 built-in tools + 8 user-defined tools + DHT/generic I2C sensor tools
- ✅ Telegram + Serial/USB + Web Relay channels
- ✅ NVS encrypted persistent storage
- ✅ Timezone-aware cron (periodic/daily/once)
- ✅ GPIO safety policies + I2C scanning
- ✅ Boot loop protection + factory reset
- ✅ Rate limiting (100/hr, 1000/day)
- ✅ OTA firmware updates
- ✅ QEMU host-side testing
- ✅ T-Relay board preset

**TinyClaw additionally (v0.0.16, rebranded to TinyAGI):**
- ✅ Multi-agent, multi-team orchestrator
- ✅ 3 channel implementations (Discord, Telegram, WhatsApp)
- ✅ SQLite message queue with dead-letter handling
- ✅ Team collaboration (chain execution, fan-out mentions)
- ✅ Async chatrooms per team
- ✅ TinyOffice live workspace with streaming execution progress
- ✅ Curl-based installer + simplified single-command onboarding
- ✅ Plugin system with message hooks
- ✅ Pairing-based access control
- ✅ CLI-based providers (Claude, Codex, OpenCode)
- ✅ SSE real-time streaming of agent execution progress

**Agent Zero additionally:**
- ✅ Hierarchical multi-agent with subagent spawning
- ✅ LiteLLM integration (100+ providers)
- ✅ Browser automation (Playwright)
- ✅ Knowledge tool (SearXNG + FAISS memory)
- ✅ SKILL.md standard support
- ✅ MCP client/server
- ✅ Docker sandbox deployment
- ✅ Scheduler (cron, ad-hoc, planned tasks)
- ✅ 23 built-in tools
- ✅ A2A chat (agent-to-agent)

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

Git repository activity metrics as of 2026-03-22.

### Commit Activity

| Project | Language | Total Commits | Last 90d | Last 30d | Last 7d | First Commit | Last Commit |
|---------|----------|---------------|----------|----------|---------|--------------|-------------|
| **OpenClaw** | TypeScript | 21,079 | 19,011 | 8,204 | 1,055 | 2025-11-24 | 2026-03-22 |
| **ZeroClaw** | Rust | 2,224 | 2,224 | 1,179 | 308 | 2026-02-13 | 2026-03-22 |
| **NullClaw** | Zig | 1,922 | 1,922 | 1,792 | 239 | 2026-02-16 | 2026-03-21 |
| **Moltis** | Rust | 1,528 | 1,528 | 271 | 0 | 2026-01-28 | 2026-03-15 |
| **Nanobot** | Python | 1,397 | 1,397 | 791 | 61 | 2026-02-01 | 2026-03-22 |
| **PicoClaw** | Go | 1,350 | 1,350 | 873 | 156 | 2026-02-09 | 2026-03-22 |
| **Agent Zero** | Python | 1,345 | 344 | 3 | 0 | 2024-06-10 | 2026-02-24 |
| **IronClaw** | Rust | 695 | 695 | 504 | 50 | 2026-02-02 | 2026-03-20 |
| **CoPaw** | Python | 447 | 447 | 447 | 94 | 2026-02-27 | 2026-03-23 |
| **MimiClaw** | C (ESP32) | 214 | 214 | 91 | 0 | 2026-02-04 | 2026-03-17 |
| **ZClaw** | C (ESP32) | 198 | 198 | 197 | 12 | 2026-02-20 | 2026-03-22 |
| **TinyClaw** | TypeScript | 167 | 167 | 96 | 22 | 2026-02-09 | 2026-03-22 |
| **RosClaw** | TypeScript + Python | 23 | 23 | 2 | 0 | 2026-02-15 | 2026-03-03 |

### Contributor Activity (Last 90 Days)

> Contributor counts from GitHub API as of 2026-03-22. Some projects squash commits, so per-contributor rates vary.

| Project | Active Contributors | Total Contributors | Commits/Contributor (90d) |
|---------|---------------------|-------------------|---------------------------|
| **OpenClaw** | 1,147 | 1,150 | 16.1 |
| **ZeroClaw** | 158 | 158 | 11.2 |
| **Nanobot** | 135 | 135 | 9.9 |
| **PicoClaw** | 144 | 144 | 7.1 |
| **NullClaw** | 49 | 49 | 33.9 |
| **CoPaw** | 47 | 47 | 5.0 |
| **Agent Zero** | 38 | 38 | 9.6 |
| **IronClaw** | 37 | 37 | 17.4 |
| **TinyClaw** | 18 | 18 | 5.9 |
| **Moltis** | 14 | 14 | 109.1 |
| **LocalGPT** | 12 | 13 | 33.8 |
| **RosClaw** | 8 | 8 | 2.9 |
| **MimiClaw** | 6 | 6 | 35.7 |
| **ZClaw** | 4 | 4 | 46.5 |

### Velocity Tiers

**Tier 1 — Hyperactive (>1000 commits/30d):**
- **OpenClaw** (8,204) — Reference implementation, massive community
- **NullClaw** (1,792) — Zig upstart, sustained fast growth
- **ZeroClaw** (1,179) — Rapid development, WASM plugins + Live Canvas

**Tier 2 — Very Active (500-1000 commits/30d):**
- **PicoClaw** (873) — Go embedded, TUI launcher + SubTurn system
- **Nanobot** (791) — Python lightweight, multimodal perception
- **IronClaw** (504) — Security-focused Rust, v0.21.0

**Tier 3 — Moderate (100-500 commits/30d):**
- **CoPaw** (447) — Growing fast, v0.1.0 release
- **Moltis** (271) — Feature-rich Rust (slowing)
- **ZClaw** (197) — Ultra-minimal ESP32, DHT/I2C sensors
- **TinyClaw** (96) — Rebranded to TinyAGI
- **MimiClaw** (91) — ESP32 embedded

**Tier 4 — Steady (&lt;100 commits/30d):**
- **Agent Zero** (3) — Established Python framework (since 2024), dormant
- **RosClaw** (2) — OpenClaw robotics plugin, re-architecture in progress

### Development Patterns

| Pattern | Projects | Notes |
|---------|----------|-------|
| **Community-driven** | OpenClaw, Nanobot, PicoClaw, ZeroClaw | 100+ contributors, distributed development |
| **Small team** | Moltis, MimiClaw, LocalGPT, TinyClaw (TinyAGI), Agent Zero | &lt;100 contributors, concentrated development |
| **Corporate-backed** | OpenClaw, CoPaw | OpenClaw: established; CoPaw: Alibaba/AgentScope |
| **Solo/small founder** | MimiClaw, NullClaw, ZClaw, RosClaw | &lt;10 contributors, focused vision |
| **Established (pre-2026)** | Agent Zero | Started June 2024, mature codebase |
| **Recent launches (Feb 2026)** | NullClaw, PicoClaw, ZeroClaw, CoPaw, MimiClaw, LocalGPT, RosClaw, ZClaw, TinyClaw | New wave of implementations |
| **Specialized** | RosClaw (robotics), TinyClaw/TinyAGI (multi-agent), Agent Zero (hierarchical agents) | Domain-specific extensions of the claw pattern |

---

## Deviations & Unique Strengths

### OpenClaw
1. **Advanced hybrid memory** — 80+ files implementing vector + keyword (BM25/FTS5) search with Reciprocal Rank Fusion, MMR diversity re-ranking, temporal decay, and LLM-based query expansion
2. **6 embedding providers** — OpenAI, Gemini (including multimodal image+audio), Voyage AI, Mistral, Ollama, with batch processing for each
3. **Session memory indexing** — Experimental feature indexing session transcripts alongside memory files with delta tracking
4. **Post-compaction memory sync** — Forced memory synchronization after session compaction ensures no context loss
5. **Citation system** — on/off/auto citation modes; auto shows citations in DMs, suppresses in groups
6. **Memory v2 research** — Proposed Retain/Recall/Reflect architecture with entity pages, belief confidence, and daily log normalization
7. **78+ plugin extensions** — Largest plugin ecosystem across 20+ messaging channels
8. **Heartbeat isolation** — Fresh session per heartbeat run, custom prompts, exec wake scoping
9. **Pi agent runtime** — RPC-based agent execution model shared across mobile/web/CLI

### IronClaw
1. **WASM sandbox** — Lighter weight than Docker, capability-based permissions
2. **Docker sandbox** — Orchestrator/worker pattern with per-job tokens
3. **NEAR AI focus** — Primary provider with browser OAuth
4. **Tinfoil private inference** — Hardware-attested TEE provider
5. **Self-repair with fault injection** — Automatic stuck detection/recovery + FaultInjector testing framework
6. **Dynamic tool building** — Describe tools in natural language
7. **Parallel job execution** — Isolated contexts per job
8. **PostgreSQL + pgvector** — Vector search with Reciprocal Rank Fusion + LRU embedding cache
9. **Rich terminal UI** — rustyline + termimad with approval overlays + unified settings page
10. **Webhook relay events** — Receive relay events via webhook callbacks

### LocalGPT
1. **Gen mode** — Bevy 3D scene generation + FunDSP procedural audio + avian3d physics + world forking with attribution
2. **Bridge daemon architecture** — tarpc-based IPC for channel isolation (Telegram, Discord, WhatsApp, Slack)
3. **UniFFI mobile bindings** — Native iOS (Swift) + Android (Kotlin) from shared Rust core
4. **Multi-layer sandboxing** — OS-level (Landlock/Seatbelt) + Docker/Podman containers (cap-drop ALL, network isolation)
5. **Encryption at rest** — Session and config secret encryption with `localgpt encrypt` CLI
6. **Browser automation** — Chrome DevTools Protocol tool for web interaction
7. **Session transcript indexing** — Session transcripts indexed for memory search alongside workspace files
8. **Post-compaction context injection** — AGENTS.md context injected after session compaction
9. **Gemini multimodal embeddings** — Image + audio indexing via Gemini embedding provider
10. **Policy signing** — HMAC-SHA256 signed LocalGPT.md workspace security policies
11. **MCP server management** — `localgpt tool list/add/remove` CLI for managing MCP server configurations
12. **Profile isolation** — `--profile` flag for completely isolated config/data directories

### Moltis
1. **46-crate workspace** — 196K lines of core code, 2,300+ tests, highly modular
2. **Zero unsafe code** — Workspace-level `deny(unsafe)` lint (except opt-in FFI behind `local-embeddings`)
3. **Encryption at rest** — XChaCha20-Poly1305 + Argon2id
4. **GraphQL API** — HTTP + WebSocket GraphQL in addition to RPC
5. **Voice I/O** — 15+ TTS/STT providers out-of-box (`moltis-voice`)
6. **Browser automation** — Chrome/Chromium via CDP (`moltis-browser`)
7. **Apple Container sandbox** — Native macOS container support alongside Docker
8. **WebAuthn/passkey auth** — Hardware-backed authentication
9. **Tailscale integration** — Serve + Funnel modes for network exposure
10. **A2UI Canvas** — Agent-controlled HTML UI for mobile/web
11. **15 lifecycle hooks** — Comprehensive hooks with circuit breaker
12. **SSRF/CSWSH protection** — Enterprise security hardening

### Nanobot
1. **Ultra-lightweight Python** — ~4,000 lines of core code, minimal dependencies, fast to deploy
2. **Broadest channel support** — 9 messaging platforms + email (Telegram, Discord, Slack, WhatsApp, Feishu, QQ, DingTalk, Mochat, Email)
3. **Chinese provider ecosystem** — DashScope/Qwen, Moonshot/Kimi, MiniMax, Zhipu/GLM, SiliconFlow, VolcEngine, AiHubMix
4. **MCP integration** — stdio + HTTP transports for tool extensibility
5. **Two-layer memory** — MEMORY.md (long-term facts) + HISTORY.md (searchable log) with LLM-driven consolidation
6. **OAuth provider auth** — GitHub Copilot and OpenAI Codex via device OAuth flow
7. **Cron delivery routing** — Scheduled task results routed to specific messaging channels

### CoPaw
1. **AgentScope/Alibaba ecosystem** — Built by Alibaba's AgentScope team with enterprise focus (v0.1.0p1)
2. **Native desktop installers** — One-click setup for Windows and macOS
3. **Console web UI** — Full-featured browser-based management with multimodal support (image/file uploads)
4. **Built-in code search** — glob_search and grep_search tools for codebase exploration
5. **10 channels** — DingTalk, Feishu, QQ, Discord, iMessage, Telegram, Signal, Matrix, MQTT, Nostr + WeChat/WeCom QR access
6. **Local model support** — llama.cpp, MLX, Ollama for on-device inference
7. **ReMeLight memory** — Smart truncation and memory optimization system
8. **Skills zip upload** — Import skills from zip archives
9. **DaemonAgent** — Autonomous diagnostics agent
10. **Twilio voice** — Voice call channel via Twilio
11. **Agent interruption API** — Ability to interrupt running agents
12. **i18n support** — Russian, Japanese, and CJK language support in web UI

### PicoClaw
1. **Go-native ultra-lightweight** — &lt;10MB RAM, &lt;1s boot, single binary
2. **SubTurn evaluator-optimizer** — Proof-of-concept for iterative agent refinement with token budget tracking
3. **TUI launcher** — Full picoclaw-launcher-tui with cyberpunk theme, config management, gateway/channels pages
4. **Event bus + hook manager** — Foundation for configurable lifecycle hooks with centralized turn events
5. **Multi-architecture** — RISC-V, ARM, MIPS, x86 from Sipeed
6. **$10 hardware target** — Designed for cheapest Linux boards
7. **Telegram streaming** — LLM response streaming via sendMessageDraft
8. **WebSocket outbound channel** — pico_client outbound for real-time connections
9. **WeCom support** — Enterprise WeChat (企业微信) channel + long-connection mode
10. **AI-bootstrapped development** — 95% agent-generated core code

### ZeroClaw
1. **Extism WASM plugin system** — Full plugin host with WasmTool, WasmChannel bridges and example weather plugin (upgraded from wasmi)
2. **ClaudeCodeTool** — Two-tier agent delegation to Claude Code for complex coding tasks
3. **Local TTS/STT** — Piper TTS for self-hosted text-to-speech + LocalWhisper for self-hosted speech-to-text
4. **Live Canvas** — Real-time collaborative gateway feature
5. **Gmail Pub/Sub push** — Push-based email channel
6. **Voice wake word detection** — New channel feature for always-listening activation
7. **Image generation** — Standalone tool via fal.ai
8. **Robot kit** — look/sense/drive/speak/listen/emote for physical robots
9. **Hardware integration** — RPi GPIO, Aardvark I2C/SPI/GPIO, hardware plugin system
10. **ARM cross-compilation** — armv7 and SBC targets in CI
11. **Slack support** — Thread replies, Assistants API status indicators, native markdown blocks
12. **Autonomous skills** — Skill creation from multi-step tasks
13. **PostgreSQL memory backend** — Added alongside existing SQLite
14. **SOP system** — Standard Operating Procedures for repeatable workflows
15. **Matrix E2EE** — End-to-end encrypted Matrix support with room-level gating

### NullClaw
1. **Zig ultra-lightweight** — 678KB binary, ~1MB RAM, &lt;2ms boot (smallest)
2. **5,300+ tests** — Most comprehensive test coverage (~250 source files)
3. **50+ AI providers** — OpenRouter, Anthropic, OpenAI, Ollama, Venice, Groq, Mistral, Xiaomi MiMo, Novita AI, and many more
4. **NullHub ecosystem** — Branded ecosystem entrypoint with documentation
5. **10 memory engines** — SQLite hybrid search, Markdown, ClickHouse, PostgreSQL, Redis, LanceDB, Lucid, LRU, API, None
6. **35+ tools** — Comprehensive built-in tool set with explicit allowlists
7. **Multi-layer sandbox** — Landlock, Firejail, Bubblewrap, Docker, WASM3 (embedded default)
8. **A2A protocol** — v0.3.0 agent-to-agent communication with task states and history
9. **Cron HTTP API** — Live scheduler exposed via HTTP endpoints with session target routing
10. **Dual-backend persistence** — SQLite + libSQL/Turso
11. **WeChat/WeCom** — Secure callbacks, DingTalk ops readiness
12. **Composio integration** — Third-party tool integration platform
13. **Hardware peripherals** — I2C, SPI, screenshot tools
14. **True portability** — ARM, x86, RISC-V single binary
15. **$5 hardware target** — Cheapest possible deployment

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

### RosClaw
1. **ROS2 integration** — Only claw ecosystem project bridging AI agents with physical robots via ROS2
2. **3 transport modes** — Rosbridge WebSocket (local network), local DDS (same machine), WebRTC (cloud/remote) with runtime switching
3. **Safety validator** — Velocity limits and workspace boundary enforcement via before_tool_call hook
4. **Robot capability auto-discovery** — Queries ROS2 graph, caches capabilities, injects context into agent system prompt
5. **Emergency stop** — /estop command bypasses AI and sends zero velocity directly
6. **8 ROS2 tools** — publish, subscribe_once, service_call, action_goal, param_get/set, list_topics, camera_snapshot
7. **WebRTC signaling** — STUN/TURN ICE negotiation with 15-second heartbeat for cloud robot connections
8. **OpenClaw plugin architecture** — Uses plugin SDK (registerTool, registerService, registerCommand, hooks)

### ZClaw
1. **888 KiB firmware budget** — Strictest size constraint of any claw (app logic ~38 KiB, total ~833 KiB)
2. **4 LLM providers** — Anthropic, OpenAI, OpenRouter, Ollama with runtime switching
3. **27 built-in + 8 user tools** — GPIO, I2C, memory, persona, cron, diagnostics, plus NVS-backed custom tools
4. **NVS encrypted storage** — Optional AES flash encryption for all persistent data
5. **GPIO safety policies** — Configurable pin range (default 2-10) with explicit allowlist override
6. **Boot loop protection** — 4-failure threshold auto-enters safe mode; serial-only recovery
7. **Factory reset button** — Hardware GPIO trigger (hold 5s) erases all NVS
8. **QEMU testing** — Full host-side test suite with mocked ESP32/FreeRTOS APIs and LLM bridge over serial
9. **Telegram poll intelligence** — Exponential backoff (5s→5min), stale poll detection, heap-aware timeout adjustment per target
10. **Persona system** — Neutral/friendly/technical/witty tone, persistent across reboots

### TinyClaw (rebranded to TinyAGI)
1. **Multi-agent teams** — Named teams with leader agents, chain execution, and fan-out parallel mentions
2. **Bracket-tagged mentions** — `[@agent: message]` syntax for agent-to-agent communication within responses
3. **Async chatrooms** — Persistent per-team chat rooms with real-time CLI viewer (`tinyagi chatroom`)
4. **TinyOffice dashboard** — Next.js web portal with live office workspace, streaming execution progress, and SSE events
5. **Curl-based installer** — Simplified onboarding to single `tinyagi` command
6. **SQLite message queue** — Atomic transactions with WAL mode, 5-retry dead-letter, stale message recovery every 5 minutes
7. **CLI provider delegation** — Spawns `claude`, `codex`, or `opencode` CLIs as subprocesses; custom providers via base_url + api_key
8. **Plugin system** — Auto-loaded from ~/.tinyagi/plugins/, transformIncoming/transformOutgoing hooks, event listeners
9. **Pairing access control** — 8-char random codes, admin approval via CLI, per-sender per-channel allowlist
10. **SSE event streaming** — Real-time streaming of agent execution progress to all clients
11. **Tmux deployment** — 24/7 operation via tmux session with queue, channels, heartbeat, and logs in separate panes

### Agent Zero
1. **Hierarchical multi-agent** — Spawn subagents with delegated tasks via `Agent.Zero` framework pattern
2. **LiteLLM integration** — Unified API for 100+ LLM providers with built-in rate limiting
3. **Browser automation** — Playwright-based browser_agent tool for web navigation, clicking, screenshots
4. **Knowledge tool** — Multi-source search combining SearXNG web search with FAISS vector memory
5. **SKILL.md standard** — Portable skill definitions with metadata (version, tags, description)
6. **MCP client/server** — Full MCP protocol support via mcp_handler and mcp_server modules
7. **Docker sandbox** — Recommended deployment model with DockerContainerManager
8. **Scheduler system** — Cron, ad-hoc, and planned task types with timezone support
9. **23 built-in tools** — Comprehensive library including code execution, vision, memory, browser
10. **A2A chat** — Agent-to-agent communication via a2a_chat tool

---

## Credits

- **IronClaw** ([ironclaw](https://github.com/nearai/ironclaw)) — Initial reference for this feature parity document. IronClaw's comprehensive feature matrix inspired the structure and categories used here.
