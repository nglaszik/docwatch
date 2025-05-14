#!/bin/bash

set -e

# Configurable paths
WORK_DIR="/opt/docwatch"
INSTALL_DIR="/usr/local/bin"
SERVICE_PATH="/etc/systemd/system"
ENV_PATH="/etc/docwatch/.env"
SERVICE_FILE="docwatch.service"

echo "Installing Docwatch..."

# 1. Create working directories
echo "Creating working directories at $WORK_DIR..."
sudo mkdir -p "$WORK_DIR/data" "$WORK_DIR/frontend" "$WORK_DIR/migrations"
sudo chown -R www-data:www-data "$WORK_DIR"

# 2. Install binaries
for BIN in docwatch docwatch-authctl docwatch-userctl; do
    if [ -f "./$BIN" ]; then
        sudo install -Dm755 "./$BIN" "$INSTALL_DIR/$BIN"
        echo "Installed $BIN to $INSTALL_DIR/$BIN"
    else
        echo "Error: Binary $BIN not found in current directory."
        exit 1
    fi
done

# 3. Create empty production database
echo "Creating empty production database at $WORK_DIR/data/docwatch.db..."
sudo touch "$WORK_DIR/data/docwatch.db"
sudo chown www-data:www-data "$WORK_DIR/data/docwatch.db"

# 4. Copy migrations folder
if [ -d "./migrations" ]; then
    sudo cp -r ./migrations "$WORK_DIR/"
    echo "Copied migrations to $WORK_DIR/migrations"
else
    echo "Error: migrations directory not found."
    exit 1
fi

# 5. Install frontend files
if [ -d "./frontend" ]; then
    sudo cp -r ./frontend/* "$WORK_DIR/frontend/"
    echo "Copied frontend files to $WORK_DIR/frontend"
else
    echo "Error: frontend directory not found."
    exit 1
fi

# 6. Install systemd service
if [ -f "./$SERVICE_FILE" ]; then
    sudo install -Dm644 "./$SERVICE_FILE" "$SERVICE_PATH/$SERVICE_FILE"
    echo "Installed systemd service to $SERVICE_PATH/$SERVICE_FILE"
else
    echo "Error: Service file $SERVICE_FILE not found."
    exit 1
fi

# 7. Install environment file
sudo mkdir -p $(dirname "$ENV_PATH")
if [ ! -f "$ENV_PATH" ]; then
    if [ -f ".env" ]; then
        sudo cp .env "$ENV_PATH"
        echo "Copied .env to $ENV_PATH"
    else
        echo "Warning: no local .env to copy! Creating a template at $ENV_PATH"
        sudo tee "$ENV_PATH" > /dev/null <<EOF
# Example .env file for Docwatch
DATABASE_URL=sqlite://data/docwatch.db
PORT=3009
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
EOF
    fi
fi

echo -e "\n✅ Docwatch installed and started successfully."
echo -e "\n➡️  Next steps:"
echo -e "1. Copy your google client id and secret to /etc/docwatch/.env"
echo -e "2. Run: sudo docwatch-authctl  # (Google API auth setup)"
echo -e "3. Run: sudo docwatch-userctl  # (Create initial user)"
echo -e "4. Run: sudo systemctl daemon-reload"
echo -e "5. Run: sudo systemctl enable docwatch"
echo -e "6. Run: sudo systemctl restart docwatch"
