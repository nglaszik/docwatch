# Docwatch

**Docwatch** is a web app designed to replace traditional AI detectors by analyzing the **revision history** of Google Docs (and soon, OneDrive documents). Instead of relying on unreliable classifiers, Docwatch highlights *when* content was added and how documents evolve over time — giving instructors and reviewers a transparent view of writing patterns.

This project is still heavily under development, so bugs are guaranteed.

Docwatch is developed to be as lightweight as possible so it can be easily installed anywhere with minimal overhead.

---

# Development Prerequisites

Before building or running Docwatch locally, ensure your system has the following tools installed:

---

## System Dependencies

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install build-essential libssl-dev pkg-config sqlite3
```

### macOS (using Homebrew)

```bash
brew install openssl sqlite
```

---

## Rust (Backend)

Install the Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then install additional Rust components:

```bash
rustup component add clippy rustfmt
```

---

## Node + npm (Frontend)

Make sure you have a recent version of Node.js and npm:

```bash
node --version
npm --version
```

If not installed:

```bash
# Ubuntu/Debian
sudo apt install nodejs npm

# Or use nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.5/install.sh | bash
nvm install --lts
```

---

## Project Setup

Once all dependencies are installed:

```bash
# Frontend
cd frontend
npm install

# Backend
cargo build
```

---

## Installation

To build and install:

```bash
make
sudo make install
```

> This will:
> - Build the app
> - Copy the backend binary to `/usr/local/bin/docwatch`
> - Install the systemd service file to `/etc/systemd/system/docwatch.service`

---

## Starting the Service

After installation, you can start and enable the service:

```bash
sudo systemctl daemon-reexec
sudo systemctl daemon-reload
sudo systemctl enable --now docwatch.service
```

The app will now be running (by default) at [http://localhost:3009](http://localhost:3009)

---

## Google API Setup

This app requires access to the **Google Drive API**. You’ll need to:

### 1. Enable Google Drive API
- Visit [https://console.cloud.google.com](https://console.cloud.google.com)
- Create or select a project
- Enable the **Google Drive API** for your project

### 2. (Optional) Create a Service Account
- Go to **IAM & Admin → Service Accounts**
- Click "Create Service Account"

### 3. Generate OAuth2 Credentials
- Under “Clients,” create an **OAuth2 client ID**
- Use type **Desktop App**
- Save your `client_id` and `client_secret`

### 4. Add to Environment

Create (or edit) the `.env` file used by the app:

```ini
GOOGLE_CLIENT_ID=your_client_id_here
GOOGLE_CLIENT_SECRET=your_client_secret_here
```

> Place this file at `/etc/docwatch/.env`  
> (The app reads it automatically on startup.)

---

## Starting the Service

Before running the app, run

```bash
docwatch-userctl
```

To add a user to the app's database

---

## Starting the Service

After installation, you can start and enable the service:

```bash
sudo systemctl daemon-reexec
sudo systemctl daemon-reload
sudo systemctl enable --now docwatch.service
```

The app will now be running (by default) at [http://localhost:3009](http://localhost:3009)

---

## Initial OAuth Setup

To authorize the app to access Google Drive:

```bash
docwatch-authctl
```

This will:
- Open a browser window (or give you a URL if headless)
- Ask you to log in with the service account or authorized Google user
- Store your token in `google_token.json`

After this, the app can begin polling documents!

---

## Adding Documents to Docwatch

To add documents to docwatch, someone must simply share the document with the service account, and Docwatch will automatically begin watching for revisions.

---

## Roadmap

- [x] Google Docs support
- [x] Per-user document dashboards
- [x] Revision diff visualization
- [ ] Visualization of writing pace
- [ ] OneDrive support (in progress)
- [ ] Instructor dashboard for class-level views
- [ ] PDF/CSV export of change history

---

## Acknowledgments

- [Axum](https://docs.rs/axum)
- [Svelte](https://svelte.dev)
- [Flowbite](https://flowbite.com)

---

## License

MIT License — use, share, and build on it freely!
