# Makefile
BINARY_NAME=docwatch
INSTALL_DIR=/usr/local/bin
SERVICE_FILE=systemd/docwatch.service
SERVICE_PATH=/etc/systemd/system/docwatch.service
ENV_PATH=/etc/docwatch/.env
WORK_DIR=/opt/docwatch
DATABASE_PATH=data/docwatch.db

build:
	@echo "📂 Ensuring dev database exists at $(DATABASE_PATH)..."
	mkdir -p $(dir $(DATABASE_PATH))
	[ -f $(DATABASE_PATH) ] || sqlite3 $(DATABASE_PATH) 'VACUUM;'
	
	@echo "🧱 Running local migrations..."
	DATABASE_URL=sqlite://$(DATABASE_PATH) sqlx migrate run
	
	@echo "🔨 Building with DATABASE_URL=sqlite://$(DATABASE_PATH)"
	DATABASE_URL=sqlite://$(DATABASE_PATH) cargo build --release --bin docwatch

install: build
	@echo "📂 Creating working directory at $(WORK_DIR)..."
	mkdir -p $(WORK_DIR)/data
	chown -R www-data:www-data $(WORK_DIR)

	@echo "📦 Installing binary to $(INSTALL_DIR)..."
	install -Dm755 target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)

	@echo "📄 Installing systemd service file to $(SERVICE_PATH)..."
	install -Dm644 $(SERVICE_FILE) $(SERVICE_PATH)

	@echo "📝 Creating environment file at $(ENV_PATH)..."
	mkdir -p $(dir $(ENV_PATH))
	[ -f $(ENV_PATH) ] || echo -e "DATABASE_URL=sqlite://data/docwatch.db\nPORT=3000" > $(ENV_PATH)

	@echo "📡 Reloading systemd and starting service..."
	systemctl daemon-reexec
	systemctl enable docwatch
	systemctl restart docwatch

	@echo "✅ Docwatch installed and running at http://localhost:3000"

uninstall:
	@echo "❌ Uninstalling Docwatch..."
	systemctl stop docwatch || true
	systemctl disable docwatch || true
	rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	rm -f $(SERVICE_PATH)
	systemctl daemon-reexec
