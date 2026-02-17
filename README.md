# MyMolt Core 🇪🇺 🌍

[![License: EUPL v1.2](https://img.shields.io/badge/license-EUPL%20v1.2-blue.svg)](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
[![Based on ZeroClaw](https://img.shields.io/badge/Based%20on-MyMolt-orange)](https://github.com/openagen/mymolt)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Identity](https://img.shields.io/badge/Identity-eIDAS%20%2B%20OIDC-blueviolet)]()

> **The Infrastructure for Digital Sovereignty.**
> A high-performance, private AI agent framework designed to serve the common good—empowering everyone from children to seniors.

**Homepage:** [mymolt.org](https://mymolt.org) | **Mesh Networking & Privacy:** [mymolt.network](https://mymolt.network)| **Identity Service:** [mymolt.id](https://mymolt.id)

---

## 🌍 About MyMolt

MyMolt is more than just software; it is a digital public utility. In an era where personal data is often exploited, MyMolt returns control to the individual.

Built on the ultra-efficient **MyMolt Framework (Rust)**, MyMolt enables every citizen to run their own sovereign AI agent. Our system runs on minimal hardware (starting at <€1 VPS or Raspberry Pi), ensuring that privacy and high-tech assistance are accessible to all parts of society, not just the wealthy.

### Core Philosophy
*   **Digital Sovereignty:** Your data, your agent, your rules.
*   **Societal Welfare:** Technology designed to protect the vulnerable (Kids/Seniors) and empower the capable.
*   **Neutrality:** Open Source, transparent, and legally anchored by the **EUPL v1.2**.

---

## 🚀 Key Features

### 1. High-Performance Core (Rust)
MyMolt Core utilizes the MyMolt architecture to deliver maximum efficiency, making digital independence affordable for everyone.
*   **<5 MB RAM** idle usage.
*   **<10ms** startup time.
*   Runs on **€1 VPS** instances, legacy laptops, or Single Board Computers (Olimex, Raspberry Pi).

### 2. The Sovereign Identity Bridge
We solve the web's trust problem by bridging commercial convenience with civic security:
*   **Access:** Login via **Google/Apple ID (OIDC)** or **eIDAS 2.0** (EU Digital Identity Wallet).
*   **Privacy:** Platforms confirm *who* you are, but gain **no access** to your agent's memory or "Soul."
*   **Safety:** Your identity is cryptographically bound to your agent, preventing bot impersonation and ensuring trusted communication.

### 3. Mesh Networking & Privacy
*   **P2P Communication:** Agents communicate directly via **WireGuard** mesh networking—no central surveillance.
*   **Local-First:** Data stays on your device or private VPS by default.
*   **Secure Gateway:** Native integration with Telegram, WhatsApp, and Signal means seniors and children can use MyMolt without learning complex new interfaces.

### 4. Adaptive Modes
MyMolt adapts to the human it serves:
*   **Mentor Mode (Kids):** Filters manipulative content and ads; focuses on education and media literacy.
*   **Care Mode (Seniors):** Simplifies interfaces, protects against scams/phishing, and assists with daily digital tasks via voice or chat.

---

## 🛠 Installation & Start

MyMolt Core is available as a single binary.

### Quick Start (Linux/macOS)

```bash
# Clone Repository
git clone https://github.com/mymolt/mymolt-core.git
cd mymolt-core

# Build (requires Rust)
cargo build --release

# Initialize & Onboard
./target/release/mymolt onboard
Setting up Identity
To link your agent to your real-world identity:
1. Start the onboarding process.
2. Select mymolt.id as the Identity Provider.
3. Authenticate via Google, Apple, or your EU Wallet.
4. Your agent generates a cryptographic key pair and binds it to your identity in the local SOUL.md file.
```
--------------------------------------------------------------------------------
🏗 Architecture
MyMolt extends the MyMolt design with layers focused on compliance, identity, and social safety:
Layer
Technology
Description
Core
Rust / MyMolt
The runtime environment. Efficient, secure, sandboxed.
Identity
OIDC / eIDAS
Verifiable Credentials and Identity Proofing.
Brain
Local LLM / API
Connects to Ollama (via Mesh Tunnel) for privacy or APIs.
Frontend
Antigravity / Messenger
Web Dashboard (Antigravity) & Chat via Telegram/WhatsApp.
Network
WireGuard
Encrypted P2P mesh network between agents.

--------------------------------------------------------------------------------
🤝 Contributing
We are building infrastructure for the public good.
Everyone is invited to contribute to MyMolt. Whether you are a Rust developer, a legal expert on eIDAS, or a designer interested in accessible UI for seniors—your help is welcome.
1. Fork the repository.
2. Create a feature branch (git checkout -b feature/AmazingFeature).
3. Commit your changes (git commit -m 'Add some AmazingFeature').
4. Push to the branch (git push origin feature/AmazingFeature).
5. Open a Pull Request.
Please note our Code of Conduct and Security Policy.

--------------------------------------------------------------------------------
⚖️ License
This project is licensed under the European Union Public Licence (EUPL v. 1.2).
This license was chosen to ensure MyMolt remains a free tool for society. It is compatible with the GPL but legally valid in all EU languages and specifically designed for the public sector.
Note: The core framework is based on MyMolt (MIT License); all MyMolt-specific extensions (Identity, Governance, Mesh Logic) are subject to the EUPL.

--------------------------------------------------------------------------------
