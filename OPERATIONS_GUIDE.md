# MyMolt Core: Operations & Functionality Guide 🦎

This guide explains the current capabilities of MyMolt Core (v0.1.0) and provides step-by-step instructions for interacting with the system via the Command Line Interface (CLI) and the Web UI.

---

## 1. System Architecture Overview

MyMolt Core consists of two main parts:
- **Backend (Rust)**: High-performance daemon that handles the secure gateway, identity storage (Soul), and VPN management.
- **Frontend (React/Vite)**: A modern dashboard for visual management and interaction.

---

## 2. Terminal (CLI) Functionality

The `mymolt` binary is the entry point for all operations.

### Running the Daemon
To start the MyMolt server and all its components (Gateway, Identity, VPN):
```bash
cargo run --bin mymolt -- daemon
```
**Output Highlights:**
- **Local Address**: Usually `127.0.0.1:3000`.
- **Pairing Code**: A 6-digit one-time code generated at startup (e.g., `🔐 371484`). This is required to link your browser UI securely to the backend.

### Onboarding & Initialization
The onboarding flow helps set up your "Soul" (local identity file).
```bash
cargo run --bin mymolt -- onboard
```

### Components Managed by CLI
- **Gateway**: Listens for HTTP/WebSocket requests.
- **VPN Engine**: Manages the local `wg0.conf` configuration.
- **Identity Store**: Persists your verified claims and linked accounts.

---

## 3. Web UI Functionality (Dashboard)

The Dashboard provides a premium, interactive experience for managing your digital sovereignty.

### Secure Pairing
1. Open `http://localhost:5173` (development) or `http://localhost:3000` (production).
2. Enter the **Pairing Code** shown in your terminal.
3. Once connected, your browser stores a secure token for future access.

### Identity Management 💳
- **SSI Wallet**: Link your Self-Sovereign Identity wallet to verify your DID (Decentralized Identifier).
- **Google/Social Login**: Use OIDC for convenient, low-trust identity bridging.
- **Verify eID (eIDAS)**: 
    - Click "Verify eID" to upload a digital certificate.
    - System verifies the claim and adds a blue "EIDAS" badge to your account.
    - **Trust Level**: Identities are marked with trust levels (High for eIDAS, Low/Medium for Social).

### Secure VPN (WireGuard) 🛡️
- **Initial Setup**: Click "Create VPN Network" if it's your first time. This generates the server's private keys and initial bridge.
- **Adding Devices**:
    1. Click "+ Add Device".
    2. Input a device name (e.g., "iPhone").
    3. **QR Code**: A WireGuard-compatible QR code will appear. Scan this with your phone to instantly configure the VPN.
    4. **Download**: You can also download a `.conf` file for desktop clients.
- **Management**: Revoke access for any device by clicking the "Trash" icon.

### Security Controls & Testing 🔐
- **Pairing Toggle**: Enable/Disable the requirement for a pairing code for new connections.
- **Voice Echo**: Set to "Loopback" to hear your own voice processed by the system (for testing).
- **Test Bot Voice**: Clicking this triggers the agent to speak a test phrase, verifying that the audio output pipeline is functional.

### Voice & Chat 🎙️
- **Voice Button**: Hold or click the large microphone button at the bottom to talk to your agent.
- **Mascot Interaction**: The chameleon mascot provides visual feedback on system status and loading.

---

## 4. Operational Tips

- **Configuration**: All configuration is stored in `config.toml`. 
- **Workspace**: The `workspace_dir` (default: `./workspace`) contains your identity files (`soul.json`) and VPN configs (`network/wg0.conf`).
- **Development Mode**:
    - Backend: `cargo run --bin mymolt -- daemon`
    - Frontend: `cd frontend && npm run dev`
- **Production Mode**:
    - Backend: `cargo build --release && ./target/release/mymolt daemon`
    - Frontend: `cd frontend && npm run build` (Static files are served automatically by the Rust binary from `frontend/dist`).

---

## 5. Security Note

MyMolt is designed for **privacy by default**. 
- Your keys are generated locally.
- Your identity data never leaves your device unless you explicitly link a third-party provider.
- All VPN traffic is encrypted P2P via WireGuard.
