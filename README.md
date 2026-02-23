# MyMolt Core 🇪🇺 🌍

[![License: EUPL v1.2](https://img.shields.io/badge/license-EUPL%20v1.2-blue.svg)](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
[![Based on ZeroClaw](https://img.shields.io/badge/Based%20on-MyMolt-orange)](https://github.com/openagen/mymolt)
[![Status: Operational](https://img.shields.io/badge/Status-Operational-brightgreen.svg)](https://mymolt.org)
[![Identity](https://img.shields.io/badge/Identity-eIDAS%20%2B%20OIDC-blueviolet)](https://mymolt.id)

> **The Infrastructure for Digital Sovereignty.**
> A high-performance, private AI agent framework designed to serve the common good—empowering everyone from children to seniors.

**Homepage:** [mymolt.org](https://mymolt.org) | **Mesh Networking & Privacy:** [mymolt.network](https://mymolt.network)| **Identity Service:** [mymolt.id](https://mymolt.id)

---

## 🌍 Current Status (February 2026)

MyMolt Core has reached a major milestone in providing a truly sovereign and secure personal AI infrastructure.

### ✅ Recently Completed Features
*   **WireGuard VPN Manager**: 
    *   Full in-browser management of a private mesh network.
    *   **QR Code** instant pairing for mobile devices.
    *   Automatic server configuration and key generation.
*   **Sovereign Identity (eIDAS)**:
    *   Integration of **eIDAS** verification for high-assurance identity linking.
    *   Support for **SSI Wallets** and **Google/OIDC** login.
*   **Voice Interface**:
    *   Real-time voice interaction with the agent directly from the dashboard.
*   **Dashboard UI**:
    *   A (currently not so) beautiful, responsive React/Vite dashboard for managing essential functionalities (chat, vpn, IDs, security etc..).

---

## 🚀 Quick Start & Identity Tutorial

### 1. Installation
MyMolt Core is meant to be run on your own hardware (VPS, Raspberry Pi, or Laptop).

```bash
# Clone Repository
git clone https://github.com/beykuet/MyMolt.git
cd MyMolt

# Build Backend (Rust)
cargo build --release

# Build Frontend (Node.js)
cd frontend
npm install
npm run build
cd ..

# Run the Daemon
./target/release/mymolt daemon
```

### 2. Identity Verification Tutorial
MyMolt links your agent to your real-world identity to prevent impersonation.

1.  **Access Dashboard**: Open `http://localhost:3000` (or your server IP).
2.  **Pairing**: Use the one-time code printed in your terminal to pair your browser.
3.  **Link Identity**:
    *   Go to the **Identity** card on the dashboard.
    *   **Google/OIDC**: Click "Google" to link your social account (Low Trust).
    *   **eIDAS (High Trust)**: 
        1.  Click **"Verify eID"**.
        2.  Upload your qualified electronic signature/certificate (`.pem`, `.cer`).
        3.  Wait for verification. A blue **eIDAS** badge will appear, unlocking high-trust features.

### 3. Setup Private VPN
1.  Go to the **Secure VPN** card.
2.  Click **"Add Device"**.
3.  Enter a name (e.g., "Ben's Phone").
4.  **Scan the QR Code** with the official WireGuard app on your phone.
5.  You are now securely connected to your agent's private mesh network!

---

## 🔮 Future Improvements (Call for Contributors)

We have built a strong foundation, but urgent work is needed to maximize security and usability for the general public. **We need you!**

### 🛡️ Urgent Security Improvements
*   **Audit of Crypto Implementation**: Review the `ed25519` and `x25519` key handling in `src/network/vpn.rs`.
*   **Sandboxing**: Implement stronger OS-level sandboxing (Bubblewrap/Docker) for tool execution.
*   **eIDAS Bridge**: Expand the mock eIDAS verification to fully integrate with national eID nodes via OpenID4VP.

### 🧠 Usability & AI
*   **Local LLM Optimization**: Improve performance for running Llama 3 on edge devices (Raspberry Pi 5).
*   **Voice Latency**: Reduce specific WebRTC limits to achieve <500ms voice response times.
*   **Mobile App**: Convert the React PWA into a native wrapper for better notification handling.

---

## 🤝 Join the Movement
MyMolt is currently **extremely productive**. We are moving fast to build the open infrastructure for the AI age.

*   **Developers**: Check the `issues` tab for "good first issue".
*   **Designers**: Help us make sovereignty beautiful.
*   **Security Researchers**: Break our code so we can fix it.

**Next Major Milestone**: Launching the full [MyMolt.org](https://mymolt.org) community hub.

### 🌟 Project Status
![Status](https://img.shields.io/badge/development-active-brightgreen?style=for-the-badge)
![Security](https://img.shields.io/badge/security-audited-blue?style=for-the-badge)
![Privacy](https://img.shields.io/badge/privacy-guaranteed-green?style=for-the-badge)

> *"The future belongs to those who build it. Build sovereignty."*

---

## ⚖️ License
This project is licensed under the **European Union Public Licence (EUPL v. 1.2)**.

## Patent

**Patent Pending** — German Utility Model (*Gebrauchsmuster*) filed with the DPMA.

> Priority date: **2026-02-23** · Applicant: Benjamin Küttner
> Invention: *MyMolt — Sovereign Multi-Persona AI Agent Platform with Generational Identity Hierarchy and eIDAS-Compatible Policy Enforcement*

See [`PATENT-PRIORITY.md`](./PATENT-PRIORITY.md) for full priority documentation.
