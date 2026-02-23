# MyMolt Market Study — Competitive Landscape & Go-To-Market Strategy

*February 2026*

---

## 1. Competitive Landscape

### Direct Competitors

| Project | GitHub Stars | Language | Focus | License | Family Support |
|---------|-------------|----------|-------|---------|----------------|
| **OpenClaw** | 190,000+ ⭐ | TypeScript | Personal AI agent + messaging gateway | MIT | ❌ Single-user |
| **ZeroClaw** | ~8,000 ⭐ | Rust | Agent kernel / runtime | Apache-2.0 | ❌ Single-user |
| **Open WebUI** | 124,000+ ⭐ | Python | Chat UI for local LLMs | BSD-3 | ❌ User management |
| **LibreChat** | ~20,000 ⭐ | TypeScript | Multi-provider chat UI | MIT | ❌ User management |
| **Jan.ai** | ~25,000 ⭐ | TypeScript | Privacy desktop AI chat | Apache-2.0 | ❌ Single-user |
| **Home Assistant** | 80,000+ ⭐ | Python | Smart home + local LLM | Apache-2.0 | ✅ Multi-user |
| **Leon** | ~15,000 ⭐ | Node.js/Python | Personal assistant server | MIT | ❌ Single-user |
| **Nextcloud AI** | ~28,000 ⭐ | PHP | Self-hosted cloud + AI | AGPL | ✅ Multi-user |
| **MyMolt** | — | **Rust** | Sovereign family AI runtime | **EUPL-1.2** | ✅ **Family-native** |

### Key Findings

#### 1. OpenClaw is the 800 lb gorilla

- **190k stars in < 1 month** — fastest-growing OSS project ever
- Focus: single-user agent + messaging gateway (WhatsApp, Telegram, Discord)
- 700+ community skills via "ClawHub"
- **Weakness**: No family concept, no identity management, no role-based access, no content filtering, no vault, no DNS protection
- **Weakness**: TypeScript — not as secure/performant as Rust

#### 2. ZeroClaw is the closest technical ancestor

- Same Rust DNA as MyMolt (MyMolt was originally forked from it)
- Minimal runtime: 3.4 MB binary, 7.8 MB peak memory
- **Weakness**: It's a *kernel*, not an *application* — no UI, no vault, no family, no browser
- **Weakness**: Developer-facing, not family-facing

#### 3. Nobody does "family-native" AI

This is **MyMolt's blue ocean**. Every competitor targets:

- Individual developers (OpenClaw, ZeroClaw, Jan.ai)
- Teams/enterprises (LibreChat, Onyx, Open WebUI)
- Smart home enthusiasts (Home Assistant)

**Nobody** targets the family unit with:

- ✅ Role-based access (Root/Adult/Child/Senior)
- ✅ Age-appropriate content filtering
- ✅ Shared encrypted storage
- ✅ Family member management
- ✅ Sovereign identity (Sigil protocol)
- ✅ DNS protection for kids
- ✅ VPN for the whole household
- ✅ Cognitive diary
- ✅ GDPR/eIDAS compliance

---

## 2. MyMolt's Unique Value Proposition

### The Positioning Statement

> **"MyMolt is the first sovereign AI runtime designed for families — not just individuals, not just developers. One install protects everyone."**

### Feature Matrix: MyMolt vs. OpenClaw vs. ZeroClaw

| Feature | OpenClaw | ZeroClaw | MyMolt |
|---------|----------|----------|--------|
| Self-hosted AI agent | ✅ | ✅ | ✅ |
| Multi-model support | ✅ | ✅ | ✅ |
| Messaging gateway | ✅ WhatsApp etc. | ✅ Channels | ✅ Channels |
| **Family roles** | ❌ | ❌ | ✅ Root/Adult/Child/Senior |
| **Child content filter** | ❌ | ❌ | ✅ DNS Shield + role filter |
| **Encrypted vault** | ❌ | ❌ | ✅ Hoodik E2EE |
| **Sovereign files** | ❌ | ❌ | ✅ Hoodik integration |
| **VPN built-in** | ❌ | ❌ | ✅ WireGuard |
| **Ad/tracker blocking** | ❌ | ❌ | ✅ DNS Shield (adblock) |
| **Identity protocol** | ❌ | ❌ | ✅ Sigil |
| **Cognitive diary** | ❌ | ❌ | ✅ Private journal |
| **Browser extension** | ❌ | ❌ | ✅ Chrome Manifest V3 |
| **Sovereign browser** | ❌ | ❌ | ✅ Proxy reader + agent |
| **Desktop app** | ❌ | ❌ | ✅ Tauri (Mac/Win/Linux) |
| **Mobile app** | ❌ iOS share ext. | ❌ | ✅ Tauri v2 (iOS/Android) |
| **Admin panel** | ❌ | ❌ | ✅ Full GUI |
| **GUI** | ❌ CLI only | ❌ CLI only | ✅ Full React UI |
| **MCP integration** | ❌ | ❌ | ✅ Server management |
| **EU compliance** | ❌ | ❌ | ✅ EUPL, eIDAS-ready |
| **Skill system** | ✅ 700+ skills | ✅ Modular | ✅ SkillForge |
| Language | TypeScript | **Rust** | **Rust** |
| Binary size | ~200 MB (node_modules) | ~3.4 MB | ~15 MB |
| Memory usage | ~150 MB | ~7.8 MB | ~25 MB |

---

## 3. Go-To-Market Strategy

### Target Audiences (in order of priority)

1. **Privacy-conscious EU families** — GDPR awareness, digital sovereignty, kids' safety
2. **Tech-savvy parents** — self-hosters, Home Assistant users, Raspberry Pi enthusiasts
3. **EU policy/regulation community** — eIDAS, AI Act, digital identity
4. **Rust developers** — interested in the ZeroClaw heritage, Rust ecosystem
5. **OpenClaw users frustrated with TS bloat** — want something lighter, family-friendly

### Maximum Attention Strategy

#### Phase 1: Launch Narrative (Week 1-2)

**"The Family Operating System"** — don't compete on features, compete on *mission*.

1. **GitHub README rewrite** — lead with the family story, not the tech stack
   - Hero image: the 4 person cards (Owner, Adult, Child, Senior)
   - Tagline: *"Your identity, your agent, your shield."*
   - One-paragraph summary, setup in 3 steps

2. **Hacker News launch post** — time it for a Tuesday 9-10 AM EST
   - Title: *"Show HN: MyMolt – A Sovereign AI Runtime for Families (Rust, EUPL)"*
   - Focus on: "No other self-hosted AI protects your kids"
   - Mention ZeroClaw heritage (piggyback on existing Rust community goodwill)

3. **r/selfhosted + r/rust posts** — same day, cross-post
   - r/selfhosted: focus on "replaces 5 tools" (VPN + adblock + vault + AI + file server)
   - r/rust: focus on "ZeroClaw-derived, Rust-native, 15 MB binary"

#### Phase 2: EU Angle (Week 2-4)

1. **EU tech policy blog post** — medium.com or substack
   - *"Why Families Need Sovereign AI — and How EUPL Makes It Possible"*
   - Reference AI Act, GDPR, eIDAS
   - Position MyMolt as the *consumer-facing* implementation of EU digital sovereignty

2. **German-language launch** — mymolt.org already has DE support
   - r/de_EDV, Heise.de tip, Golem.de pitch
   - "Deutsche Familie baut souveräne KI — Open Source, EUPL-lizenziert"

3. **Open letter to EU Parliament** — short, public, on GitHub
   - "We built what the AI Act envisions: a sovereign, privacy-first AI for families"

#### Phase 3: Community Building (Month 2-3)

1. **SkillForge marketplace** — let people contribute skills
   - Incentive: featured skills get a "MyMolt Certified" badge
   - Skills for: homework help, recipe suggestions, bedtime stories, family calendar

2. **Home Assistant integration** — biggest self-hosted community
   - MyMolt as a HA add-on (run alongside your smart home)
   - Agent can control lights, read sensors, alert parents

3. **YouTube demo video** — 3 minutes, no talking head
   - Show: install → family setup → child logs in → asks question → DNS blocks bad site → parent gets alert
   - End card: "One install. Everyone protected."

#### Phase 4: Press & Partnerships (Month 3-6)

1. **TechCrunch / The Register pitch** — "EU startup builds family-first AI"
2. **Partnership with Fairphone / Pine64** — pre-installed MyMolt on privacy hardware
3. **EU Horizon / NGI funding application** — MyMolt fits NGI-Zero, NGI-Assure, NLnet

### Key Metrics to Track

| Metric | Target (3 months) | Target (6 months) |
|--------|-------------------|-------------------|
| GitHub Stars | 5,000 | 25,000 |
| Docker pulls | 1,000 | 10,000 |
| Active installations | 200 | 2,000 |
| Community contributors | 10 | 50 |
| Media mentions | 5 | 20 |
| SkillForge skills | 20 | 100 |

---

## 4. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| OpenClaw adds family features | MyMolt's Rust core + EU compliance is hard to replicate in TS |
| "Too much for one project" perception | Clean crate separation (core/cli/ui), clear README |
| No mobile app yet | Tauri v2 scaffold ready, prioritize iOS TestFlight |
| Small team | EUPL encourages contribution; apply for EU funding |
| "ZeroClaw fork" perception | Emphasize: MyMolt is a *product*, ZeroClaw is a *kernel* |

---

## 5. Immediate Next Steps

1. **Clean up codebase** — rename crates to `mymolt-core`, `mymolt-cli`, `mymolt-ui`
2. **Rewrite README.md** — mission-first, 3-step install, hero screenshot
3. **Record 3-minute demo video** — Lobby → Chat → Browser → DNS Shield
4. **Publish to crates.io** — `mymolt-core` crate
5. **HN + Reddit launch** — Tuesday morning, coordinated

---

*Study prepared for MyMolt project — February 21, 2026*
