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

	@echo "Running local migrations on dev DB..."
	DATABASE_URL=sqlite://$(DATABASE_PATH) sqlx migrate run

	@echo "Building frontend files..."
	cd frontend && npm ci && npm run build

	@echo "Building Rust binaries..."
	cargo build --release --bin $(BINARY_NAME)
	cargo build --release --bin $(BINARY_AUTHCTL_NAME)
	cargo build --release --bin $(BINARY_USERCTL_NAME)

install:
	@echo "Creating working directory at $(WORK_DIR)..."
	mkdir -p $(WORK_DIR)/data
	mkdir -p $(WORK_DIR)/frontend
	mkdir -p $(WORK_DIR)/migrations
	chown -R www-data:www-data $(WORK_DIR)

	@echo "Installing binaries to $(INSTALL_DIR)..."
	install -Dm755 target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	install -Dm755 target/release/$(BINARY_USERCTL_NAME) $(INSTALL_DIR)/$(BINARY_USERCTL_NAME)
	install -Dm755 target/release/$(BINARY_AUTHCTL_NAME) $(INSTALL_DIR)/$(BINARY_AUTHCTL_NAME)
	
	@echo "Creating empty production database..."
	touch $(WORK_DIR)/data/docwatch.db
	chown www-data:www-data $(WORK_DIR)/data/docwatch.db
	cp -r migrations $(WORK_DIR)/migrations
	
	@echo "Installing frontend files..."
	cp -r frontend/dist/* $(WORK_DIR)/frontend/

	@echo "Installing systemd service file to $(SERVICE_PATH)..."
	install -Dm644 $(SERVICE_FILE) $(SERVICE_PATH)

	@echo "Creating environment file at $(ENV_PATH)..."
	mkdir -p $(dir $(ENV_PATH))
	[ -f $(ENV_PATH) ] || ( [ -f .env ] && cp .env $(ENV_PATH) || echo "Warning: no local .env to copy!" )

	@echo "Docwatch installed, please run the following required commands:\ndocwatch-authctl (sets up Google API authentication. This needs to be done from a GUI)\ndocwatch-userctl (creates your first user for login)\nsudo systemctl daemon-reload\nsudo systemctl start docwatch.service (start the server)"

uninstall:
	@echo "❌ Uninstalling Docwatch..."
	systemctl stop docwatch || true
	systemctl disable docwatch || true
	rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	rm -f $(SERVICE_PATH)
	systemctl daemon-reexec
