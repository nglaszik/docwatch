# Docwatch

**Docwatch** is a web app designed to replace traditional AI detectors by analyzing the **revision history** of Google Docs (and soon, OneDrive documents). Instead of using unreliable classifiers, Docwatch highlights *when* content was added and how documents evolve over time — giving instructors and reviewers a transparent view of writing patterns.

Docwatch is developed to be installed on a server with minimal overhead. Users can then access the app via a browser.

Having a document be visible in Docwatch is as simple as a student sharing the document with the Docwatch service account.

This project is still heavily under development, so bugs are guaranteed.

---

# Normal Installation

## Google API Setup

Before installation, set up a Google Cloud project and enable API access. You’ll need to:

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
- Write down your `client_id` and `client_secret` for later...

---

## Download and Install

### 1. Download the latest release
Head to the [Releases](https://github.com/nglaszik/docwatch/releases) page and download the latest `.tar.gz` package for your system.

Command line example: (Update the version)
```bash
curl -LO https://github.com/nglaszik/docwatch/releases/download/v1.0.0/docwatch-v1.0.0.tar.gz
```

### 2. Extract the release package
```bash
tar -xzf docwatch-v1.0.0.tar.gz
cd docwatch-v1.0.0
```

### 3. Run the installer script
```bash
sudo ./install.sh
```

This will:
- Install the backend binaries to `/usr/local/bin`
- Set up working directories at `/opt/docwatch`
- Initialize an empty production database
- Install frontend assets and migrations
- The server is not running since you need to first set up authentication

### 4. Post-install required setup

First, put your google client id and secret from earlier into `/etc/docwatch/.env`

Next, run the following commands:

```bash
# 1. Authenticate with Google API (must run from command line in a GUI-enabled session. Install ThinLinc if needed)
sudo docwatch-authctl

# 2. Create your first user for login
sudo docwatch-userctl

# 3. Start service
sudo systemctl daemon-reload
sudo systemctl enable docwatch
sudo systemctl restart docwatch
```

---

## Adding Documents to Docwatch

To add documents to Docwatch, someone must simply share the document with the service account, and Docwatch will automatically begin watching for revisions.

Authenticated users can then search for shared documents and add them to their watchlist.

---

# Development Installation

## Dependencies

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install build-essential libssl-dev pkg-config sqlite3
```

---

### Rust

Install the Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then install additional Rust components:

```bash
rustup component add clippy rustfmt
```

---

### Node + npm

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

## Build/Install

Before building and installing, set up the [Google API](#google-api-setup)

To build and install:

```bash
git clone git@github.com:nglaszik/docwatch.git
cd docwatch
make
sudo make install
```

> This will:
> - Build the app
> - Move binaries and pages
> - The service is not started since we must first authenticate

Follow the [Post Installation Instructions](#4-post-install-required-setup)

---

## Testing

You can make changes and test them by running the following:

```bash
# Frontend
cd frontend
npm install

# Backend
cargo run --bin docwatch
```

---

# Roadmap

- [x] Google docs & docx support
- [x] Per-user document dashboards
- [x] Revision diff visualization
- [x] Visualization of writing pace
- [x] Instructor dashboard for class-level views
- [ ] OneDrive support (in progress)

---

# Acknowledgments

- [Axum](https://docs.rs/axum)
- [Svelte](https://svelte.dev)
- [Flowbite](https://flowbite.com)

---

# License

MIT License — use, share, and build on it freely!
