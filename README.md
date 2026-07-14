<div align="center">

<img src="https://res.cloudinary.com/ddqedxovk/image/upload/v1784021168/nkatmpgrqbglxqdsuqrc.png" width="120" height="120" style="border-radius:20px" />

# LidBridge

**Clean your projects. Push to GitHub. In seconds.**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri%202-24C8DB?logo=tauri)](https://tauri.app)
[![Next.js](https://img.shields.io/badge/Frontend-Next.js%2014-black?logo=next.js)](https://nextjs.org)
[![Rust](https://img.shields.io/badge/Backend-Rust-orange?logo=rust)](https://www.rust-lang.org)
[![Made by Lidprex](https://img.shields.io/badge/Made%20by-Lidprex%20Labs-8B5CF6)](https://lidprex.onrender.com)

</div>

---

## What is LidBridge?

LidBridge is an open-source **desktop application** that helps developers clean their local project directories and publish them to GitHub — with one click.

No more manually deleting `node_modules`, `__pycache__`, `.next`, or `target` folders before sharing your code. LidBridge handles all of that automatically, detects secret files before they leak, and pushes your clean project to a new GitHub repository.

Ideal for developers preparing projects for AI code review with ChatGPT, Claude, or Gemini — clean once, push instantly, no terminal required.

---

## What's New in v2.0.0

Complete rewrite from the ground up. Here's what changed from [v1.0.0](https://github.com/Lidprex/Lidbridge-LidBridge-v1.0.0):

| Area | v1.0.0 | v2.0.0 |
|---|---|---|
| **Push System** | Used `git2` (libgit2) — broken on Windows, never actually pushed | GitHub Contents API (base64 upload) — works on all platforms |
| **Secret Detection** | Content patterns only (20 patterns) | Content patterns (20+) + filename detection (28 names + 10 extensions) |
| **Secret Detection Limit** | 1MB max file scan | No size limit — scans everything |
| **Repository Config** | 5 fields | 7 fields (added `license_template`, `repo_type`) |
| **Languages** | 5 (EN, AR, FR, HI, ZH) | 6 (EN, AR, **RU**, FR, HI, ZH) |
| **OAuth Credentials** | Hardcoded in source code | Read from `.env` file — never committed |
| **Database** | PostgreSQL analytics active | Deprecated — will return in a future version |
| **UI/UX** | Basic steps | RTL support, hover menus, security warnings, modern design |
| **Security Warning** | None | Login screen warns users to download only from official sources |
| **Build System** | Manual | GitHub Actions CI/CD (Windows, macOS Intel, macOS Apple Silicon, Linux) |
| **Platform Support** | Windows only (partially) | Windows, macOS (Intel + Apple Silicon), Linux |

---

## Features

| Feature | Description |
|---|---|
| Smart Cleaning | Automatically removes 30+ types of junk directories and files |
| Secret Detection | Scans for exposed API keys, tokens, passwords, and secret files |
| One-Click Push | Creates a GitHub repository and pushes your code in one step |
| Org Support | Push to personal accounts or any GitHub Organization you belong to |
| Project Scan | Displays file statistics before cleaning |
| Multilingual UI | Supports English, Arabic, Russian, French, Hindi, and Chinese |
| RTL Support | Full right-to-left layout for Arabic |
| Repo History | Tracks all repositories created through the app |
| License Selection | Choose a license template when creating your repository |

---

## Supported Platforms

| Platform | Status | Notes |
|---|---|---|
| Windows 10/11 | Fully supported | Primary development platform |
| macOS (Apple Silicon) | Fully supported | Built via GitHub Actions |
| macOS (Intel) | Fully supported | Built via GitHub Actions |
| Linux (x86_64) | Fully supported | Built via GitHub Actions |

---

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop Framework | [Tauri v2](https://tauri.app) |
| Backend | [Rust](https://www.rust-lang.org) |
| Frontend | [Next.js 14](https://nextjs.org) + TypeScript |
| Styling | Tailwind CSS |
| Local Database | SQLite (via `rusqlite`) |
| Push System | GitHub REST API (Contents API) |
| Auth | GitHub OAuth 2.0 |
| Build/CI | GitHub Actions |

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org) >= 18
- [Rust](https://rustup.rs) (stable toolchain)
- [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/) for your platform

### 1. Clone the Repository

```bash
git clone https://github.com/lidprex/lidbridge.git
cd lidbridge
```

### 2. Configure Environment Variables

```bash
cp .env.example .env
```

Open `.env` and fill in your GitHub OAuth credentials:

```env
GITHUB_CLIENT_ID=your_github_client_id_here
GITHUB_CLIENT_SECRET=your_github_client_secret_here
```

### 3. Set Up GitHub OAuth App

1. Go to [github.com/settings/developers](https://github.com/settings/developers)
2. Click **New OAuth App**
3. Fill in:
   - **Application name**: `LidBridge`
   - **Homepage URL**: `http://localhost:3000`
   - **Authorization callback URL**: `http://localhost:2026/callback`
4. Copy the **Client ID** and **Client Secret** into your `.env`

> You can also skip OAuth entirely: on the login screen, use **"Use Personal Token"** and paste a GitHub PAT (classic, with `repo` scope). This needs no client ID/secret.

### 4. Install Dependencies

```bash
npm install
```

### 5. Run in Development Mode

```bash
npx tauri dev
```

---

## Building for Production

### Local Build

```bash
npx tauri build
```

The installer will be generated in `src-tauri/target/release/bundle/`.

### Automated Builds (GitHub Actions)

When you push a tag like `v2.0.0`, GitHub Actions automatically builds for:

- **Windows** (.msi, .exe)
- **macOS Intel** (.dmg)
- **macOS Apple Silicon** (.dmg)
- **Linux** (.deb, .AppImage)

Build artifacts are attached to the GitHub Release as a draft.

```bash
git tag v2.0.0
git push origin v2.0.0
```

---

## Project Structure

```
lidbridge/
├── src/                        # Next.js frontend
│   └── app/
│       ├── page.tsx            # Main application + translations
│       ├── components/
│       │   └── dashboard-ui.tsx # UI components
│       └── globals.css         # Global styles & design tokens
│
├── src-tauri/                  # Rust/Tauri backend
│   ├── src/
│   │   ├── lib.rs              # Application entry point & Tauri commands
│   │   ├── main.rs             # Binary entry point
│   │   ├── auth/               # GitHub OAuth (reads from .env)
│   │   ├── cleaner/            # Project scanning & secret detection
│   │   ├── db/                 # Local SQLite + deprecated PostgreSQL
│   │   ├── git/                # Push via GitHub Contents API
│   │   ├── secret/             # Encryption utilities
│   │   ├── github_app.rs       # GitHub App JWT token generation
│   │   └── history_store.rs    # Encrypted local repo history
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri window & bundle configuration
│
├── .github/workflows/
│   └── build.yml               # CI/CD — builds for Win/Mac/Linux
│
├── .env.example                # Environment variable template
├── .env                        # Your secrets (NEVER committed)
├── .gitignore                  # Protects secrets from being committed
├── package.json                # Node.js scripts & dependencies
└── README.md                   # This file
```

---

## What Gets Cleaned?

| Category | Examples |
|---|---|
| Version Control | `.git/`, `.svn/`, `.hg/` |
| Package Managers | `node_modules/`, `.npm/`, `bower_components/` |
| Build Outputs | `dist/`, `build/`, `target/`, `out/`, `.next/` |
| Python | `__pycache__/`, `.venv/`, `.tox/`, `*.pyc` |
| IDE Files | `.idea/`, `.vscode/`, `.vs/` |
| Lock Files | `package-lock.json`, `yarn.lock`, `Cargo.lock` |
| OS Artifacts | `.DS_Store`, `Thumbs.db`, `desktop.ini` |
| Archives | `.zip`, `.tar.gz`, `.rar`, `.7z` |
| Logs | `*.log`, `logs/` |
| Environment | `.env`, `.env.local`, `.env.production` |

---

## Secret Detection

LidBridge scans your project for:

### By Content (Regex Patterns)
- GitHub tokens (`ghp_`, `gho_`, `ghu_`, `ghs_`)
- OpenAI API keys (`sk-...`, `sk-proj-...`)
- AWS Access Keys (`AKIA...`) and Secret Keys
- Google API keys (`AIza...`)
- Stripe keys (`sk_live_`, `rk_live_`)
- Discord bot tokens
- Slack tokens (`xox...`)
- JWT tokens (`eyJ...`)
- Generic `api_key=`, `password=`, `token=`, `secret_key=` patterns
- OAuth client secrets, private key references, embedded credentials in URLs

### By Filename
- `.env` files (all variants: `.env.local`, `.env.production`, etc.)
- `credentials.json`, `secrets.json`, `service-account.json`
- SSH keys (`id_rsa`, `id_ed25519`, `deploy_key`)
- Certificate files (`.pem`, `.p12`, `.key`, `.crt`)

---

## Security

### For Users
The login screen displays a warning to verify you downloaded LidBridge from an official source:
- **GitHub**: [github.com/Lidprex/Lidbridge](https://github.com/Lidprex/Lidbridge)
- **Releases**: [github.com/Lidprex/Lidbridge/releases](https://github.com/Lidprex/Lidbridge/releases)
- **Website**: [lidbridge.onrender.com](https://lidbridge.onrender.com)

### For Developers
- OAuth credentials are stored in `.env` — never hardcoded in source
- `.env` is listed in `.gitignore` and will never be committed
- The PostgreSQL analytics database is **deprecated** in v2.0.0 and will return with a proper backend

---

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Copy `.env.example` to `.env` and fill in your OAuth credentials
4. Commit your changes: `git commit -m 'feat: add my feature'`
5. Push to the branch: `git push origin feature/my-feature`
6. Open a Pull Request

Please make sure you do **not** commit `.env` or any private keys.

---

## License

This project is licensed under the **GNU General Public License v3.0**.
See [LICENSE.txt](src-tauri/LICENSE.txt) for the full license text.

```
LidBridge — Copyright (C) 2026 Lidprex Labs
This program comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it
under the conditions of the GNU GPL v3.
```

---

## Links

| | |
|---|---|
| Website | [lidprex.onrender.com](https://lidprex.onrender.com) |
| Labs | [lidprex-labs.onrender.com](https://lidprex-labs.onrender.com) |
| Lead Developer | [github.com/bxat01](https://github.com/bxat01) |
| Organization | [github.com/lidprex](https://github.com/lidprex) |

---

<div align="center">
  <sub>Built with care by <strong>Lidprex Labs</strong></sub>
</div>
