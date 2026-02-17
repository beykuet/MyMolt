#!/bin/bash
set -e

# Config
REPO="beykuet/MyMolt"
INSTALL_DIR="/opt/mymolt"
BINARY_NAME="zeroclaw"

echo "🚀 Installing MyMolt Core..."

# 1. Prepare Directory
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

# 2. Download latest release (mock logic for now, requiring manual URL or GitHub CLI)
# In a real scenario, we'd query GitHub API. For now, we assume a local binary or direct URL is provided as arg.
if [ -z "$1" ]; then
    echo "Usage: $0 <url_to_binary_tarball>"
    echo "Example: $0 https://github.com/$REPO/releases/download/v0.1.0/zeroclaw-x86_64-unknown-linux-musl.tar.gz"
    # Fallback to update logic if we already have the binary?
    # For MVP, we just exit if no URL.
    exit 1
fi

URL="$1"
echo "⬇️  Downloading from $URL..."
curl -L -o release.tar.gz "$URL"

# 3. Extract
tar xzf release.tar.gz
rm release.tar.gz
chmod +x "$BINARY_NAME"

# 4. Install Systemd Service
if [ -f "$INSTALL_DIR/$BINARY_NAME.service" ]; then
    echo "📦 Installing Systemd Service..."
    cp "$INSTALL_DIR/deploy/zeroclaw.service" /etc/systemd/system/
    systemctl daemon-reload
    systemctl enable zeroclaw
fi

# 5. Restart
echo "🔄 Restarting Service..."
systemctl restart zeroclaw

echo "✅ Done! MyMolt Core is running."
echo "   Logs: journalctl -u zeroclaw -f"
