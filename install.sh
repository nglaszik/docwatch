#!/bin/bash

set -e

# Configurable
BINARY_NAME="docwatch"
INSTALL_PREFIX="/usr/local/bin"
FRONTEND_DIR="/opt/docwatch/frontend"
SYSTEMD_SERVICE="/etc/systemd/system/docwatch.service"
ENV_FILE="/etc/docwatch/.env"

echo "Installing $BINARY_NAME..."

# 1. Install backend binary
if [ -f "./$BINARY_NAME" ] || [ -f "./$BINARY_NAME.exe" ]; then
    BIN_SRC="./$BINARY_NAME"
    # Windows case (optional .exe extension handling)
    if [ -f "./$BINARY_NAME.exe" ]; then
        BIN_SRC="./$BINARY_NAME.exe"
    fi
    sudo install "$BIN_SRC" "$INSTALL_PREFIX/$BINARY_NAME"
    echo "Installed backend binary to $INSTALL_PREFIX/$BINARY_NAME"
else
    echo "Error: Binary $BINARY_NAME not found in current directory."
    exit 1
fi

# 2. Install frontend static files
if [ -d "./frontend" ]; then
    sudo mkdir -p "$FRONTEND_DIR"
    sudo cp -r ./frontend/* "$FRONTEND_DIR/"
    echo "Copied frontend files to $FRONTEND_DIR"
else
    echo "Error: Frontend directory not found."
    exit 1
fi

# 3. Install systemd service
sudo tee "$SYSTEMD_SERVICE" > /dev/null <<EOF
[Unit]
Description=Docwatch Web App
After=network.target

[Service]
ExecStart=$INSTALL_PREFIX/$BINARY_NAME
WorkingDirectory=/opt/docwatch
EnvironmentFile=$ENV_FILE
Restart=on-failure
User=www-data
Group=www-data

[Install]
WantedBy=multi-user.target
EOF

echo "Installed systemd service at $SYSTEMD_SERVICE"

# 4. Ensure env file exists
if [ ! -f "$ENV_FILE" ]; then
    echo "Warning: $ENV_FILE does not exist. Creating a template."
    sudo mkdir -p $(dirname "$ENV_FILE")
    sudo tee "$ENV_FILE" > /dev/null <<EOF
# Example .env file for Docwatch
DATABASE_URL=sqlite://data/docwatch.db
PORT=3009
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
EOF
fi

# 5. Reload systemd, enable & start service
sudo systemctl daemon-reload
sudo systemctl enable docwatch
sudo systemctl restart docwatch

echo "Docwatch installed and started successfully."
