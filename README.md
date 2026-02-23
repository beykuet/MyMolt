<p align="center">
  <img src="mymolt.png" width="120" alt="MyMolt">
</p>

<h1 align="center">MyMolt</h1>
<p align="center"><strong>The sovereign AI runtime for families.</strong></p>
<p align="center">One install. Everyone protected.</p>

<p align="center">
  <a href="https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12"><img src="https://img.shields.io/badge/license-EUPL%20v1.2-blue.svg" alt="License"></a>
  <a href="https://mymolt.org"><img src="https://img.shields.io/badge/Status-Operational-brightgreen.svg" alt="Status"></a>
  <a href="#"><img src="https://img.shields.io/badge/Rust-native-orange.svg" alt="Rust"></a>
  <a href="#"><img src="https://img.shields.io/badge/Family-native-purple.svg" alt="Family"></a>
  <img src="https://img.shields.io/badge/Patent%20Pending-%F0%9F%87%A9%F0%9F%87%AA%20DE%20Gebrauchsmuster-blueviolet.svg" alt="Patent Pending DE">
</p>

---

## What is MyMolt?

MyMolt is a **self-hosted AI runtime** that gives your family a private AI assistant, encrypted file storage, ad blocking, VPN, and identity management — all in one Rust binary.

Unlike other self-hosted AI tools that target individual developers, MyMolt is designed for **families**:

- 🛡️ **Root** — Full admin, security controls, system management
- 💼 **Adult** — Productivity, finance, full AI access
- 🌟 **Child** — Safe mode with content filtering and DNS protection
- 💛 **Senior** — Simplified interface, voice-first

## Quick Start

```bash
# 1. Clone
git clone https://github.com/beykuet/MyMolt.git && cd MyMolt

# 2. Build
cargo build --release
cd frontend && npm install && npm run build && cd ..

# 3. Run
./target/release/mymolt daemon
# → Open http://localhost:3000
```

## What's Inside

| Module | What It Does |
| --- | --- |
| **Sovereign Chat** | AI assistant with voice, text, and multi-model support (Ollama, OpenAI, Anthropic, etc.) |
| **Sovereign Browser** | Built-in proxy reader with agent comprehension — ask MyMolt about any page |
| **DNS Shield** | Ad/tracker blocking for the whole household |
| **Secure Vault** | E2E encrypted file storage (powered by Hoodik) |
| **VPN Connect** | WireGuard mesh network with QR code pairing |
| **Soul Identity** | eIDAS + OIDC identity linking, Sigil protocol support |
| **SkillForge** | Modular skill system — teach your agent new abilities |
| **Admin Panel** | Family management, MCP servers, security overview, provider config |
| **Cognitive Diary** | Private AI-powered journal |
| **Browser Extension** | Chrome extension with vault autofill and DNS Shield |
| **Desktop App** | Tauri native app (Mac/Windows/Linux) with system tray |

## Architecture

```
mymolt-core     (Rust)    — Backend: agent, gateway, security, identity, VPN, DNS
mymolt-ui       (React)   — Frontend: dashboard, chat, browser, admin panel
mymolt-tauri    (Rust)    — Desktop/mobile app wrapper
mymolt-ext      (TS)      — Chrome browser extension
```

```
┌─────────────────────────────────────────────────────────────┐
│                    MyMolt Desktop (Tauri)                     │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                 React Frontend (mymolt-ui)              │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │  │
│  │  │  Lobby   │ │   Chat   │ │ Browser  │ │  Admin   │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │  │
│  └────────────────────────┬───────────────────────────────┘  │
│                           │ HTTP/WS API                       │
│  ┌────────────────────────▼───────────────────────────────┐  │
│  │                Rust Backend (mymolt-core)               │  │
│  │  Agent │ Gateway │ VPN │ DNS │ Vault │ Identity │ MCP  │  │
│  └────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Why MyMolt?

| | OpenClaw | Open WebUI | MyMolt |
| --- | --- | --- | --- |
| Self-hosted AI | ✅ | ✅ | ✅ |
| Family roles | ❌ | ❌ | ✅ |
| Child content filter | ❌ | ❌ | ✅ |
| Encrypted vault | ❌ | ❌ | ✅ |
| Built-in VPN | ❌ | ❌ | ✅ |
| Ad blocking | ❌ | ❌ | ✅ |
| Desktop app | ❌ | ❌ | ✅ |
| EU-compliant | ❌ | ❌ | ✅ |
| Language | TypeScript | Python | **Rust** |

## Development

```bash
# Backend
cargo check                    # Type-check
cargo build --release          # Build binary

# Frontend
cd frontend
npm run dev                    # Dev server (http://localhost:5173)
npm run build                  # Production build

# Desktop App
cd frontend
npm run tauri:dev              # Open native window
npm run tauri:build            # Build .dmg / .msi / .deb
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). MyMolt uses the **EUPL v1.2** license, which encourages contribution and ensures the project remains open.

## Patent

**Patent Pending** — German Utility Model (*Gebrauchsmuster*) filed with the DPMA.

> Priority date: **2026-02-23** · Applicant: Benjamin Küttner
> Invention: *MyMolt — Sovereign Multi-Persona AI Agent Platform with Generational Identity Hierarchy and eIDAS-Compatible Policy Enforcement*

See [`PATENT-PRIORITY.md`](./PATENT-PRIORITY.md) for full priority documentation.

## License

**European Union Public Licence (EUPL v. 1.2)**

> *Your identity, your agent, your shield.*
