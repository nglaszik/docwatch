# Makefile
BINARY_NAME=docwatch
BINARY_USERCTL_NAME=docwatch-userctl
BINARY_AUTHCTL_NAME=docwatch-authctl
INSTALL_DIR=/usr/local/bin
SERVICE_FILE=systemd/docwatch.service
SERVICE_PATH=/etc/systemd/system/docwatch.service
ENV_PATH=/etc/docwatch/.env
WORK_DIR=/opt/docwatch
DATABASE_PATH=data/docwatch.db

build:
	@echo "Ensuring dev database exists at $(DATABASE_PATH)..."
	mkdir -p $(dir $(DATABASE_PATH))
	[ -f $(DATABASE_PATH) ] || sqlite3 $(DATABASE_PATH) 'VACUUM;'
	
	@echo "Running local migrations..."
	DATABASE_URL=sqlite://$(DATABASE_PATH) sqlx migrate run
	
	@echo "Building frontend files..."
	cd frontend && npm ci && npm run build
	
	@echo "Building with DATABASE_URL=sqlite://$(DATABASE_PATH)"
	DATABASE_URL=sqlite://$(DATABASE_PATH) cargo build --release --bin docwatch
	
	@echo "Building other binaries"
	cargo build --release --bin docwatch-authctl
	cargo build --release --bin docwatch-userctl

install: build
	@echo "Creating working directory at $(WORK_DIR)..."
	mkdir -p $(WORK_DIR)/data
	chown -R www-data:www-data $(WORK_DIR)

	@echo "Installing binaries to $(INSTALL_DIR)..."
	install -Dm755 target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	install -Dm755 target/release/$(BINARY_USERCTL_NAME) $(INSTALL_DIR)/$(BINARY_USERCTL_NAME)
	install -Dm755 target/release/$(BINARY_AUTHCTL_NAME) $(INSTALL_DIR)/$(BINARY_AUTHCTL_NAME)
	
	@echo "Installing frontend files..."
	mkdir -p $(WORK_DIR)/frontend
	cp -r frontend/dist/* $(WORK_DIR)/frontend/

	@echo "Installing systemd service file to $(SERVICE_PATH)..."
	install -Dm644 $(SERVICE_FILE) $(SERVICE_PATH)

	@echo "Creating environment file at $(ENV_PATH)..."
	mkdir -p $(dir $(ENV_PATH))
	[ -f $(ENV_PATH) ] || echo -e "DATABASE_URL=sqlite://data/docwatch.db\nPORT=3009" > $(ENV_PATH)

	@echo "Reloading systemd and starting service..."
	systemctl daemon-reexec
	systemctl enable docwatch
	systemctl restart docwatch

	@echo "Docwatch installed and running at http://localhost:3009"

uninstall:
	@echo "❌ Uninstalling Docwatch..."
	systemctl stop docwatch || true
	systemctl disable docwatch || true
	rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	rm -f $(SERVICE_PATH)
	systemctl daemon-reexec
