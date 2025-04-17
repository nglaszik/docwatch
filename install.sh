#!/bin/bash
set -e

INSTALL_DIR=/usr/local/bin
WORK_DIR=/opt/docwatch
ENV_PATH=/etc/docwatch/.env
SERVICE_PATH=/etc/systemd/system/docwatch.service
BINARY_NAME=docwatch

echo "📂 Creating working directory at $WORK_DIR..."
mkdir -p "$WORK_DIR/data"
chown -R www-data:www-data "$WORK_DIR"

echo "📦 Installing precompiled binary to $INSTALL_DIR..."
install -Dm755 "./$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

echo "📄 Installing systemd service file to $SERVICE_PATH..."
cat > "$SERVICE_PATH" <<EOF
[Unit]
Description=Docwatch Server
After=network.target

[Service]
ExecStart=$INSTALL_DIR/$BINARY_NAME
WorkingDirectory=$WORK_DIR
EnvironmentFile=$ENV_PATH
Restart=always
RestartSec=2
User=www-data
Group=www-data

[Install]
WantedBy=multi-user.target
EOF

echo "📝 Creating environment file at $ENV_PATH..."
mkdir -p "$(dirname "$ENV_PATH")"
if [ ! -f "$ENV_PATH" ]; then
  cat > "$ENV_PATH" <<EOF
DATABASE_URL=sqlite://data/docwatch.db
PORT=3009
EOF
fi

echo "📡 Reloading and starting systemd service..."
systemctl daemon-reexec
systemctl enable docwatch
systemctl restart docwatch

echo "✅ Docwatch installed and running at http://localhost:3000"
