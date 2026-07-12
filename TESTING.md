# Testing LidBridge locally (macOS)

LidBridge is a Tauri desktop app: a **Rust** backend + a **Next.js** UI in one window.

## Prerequisites

- **Node.js** ≥ 18
- **Rust** (stable) — install: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Xcode Command Line Tools** (for the C toolchain the Rust deps build): `xcode-select --install`

## 1. Install dependencies

```bash
cd Lidbridge
npm install
```

## 2. (Optional) configure GitHub OAuth

You can skip this and log in with a token instead (step 4). To use the
"Login with GitHub" button, create an OAuth app at
<https://github.com/settings/developers>:

- Homepage URL: `http://localhost:3000`
- Authorization callback URL: `http://localhost:2026/callback`

Then:

```bash
cp .env.example .env      # fill in GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET
```

> `build.rs` bakes `.env` in at compile time, so rebuild after editing it.

## 3. Run in development

```bash
npm run tauri dev
```

The first run compiles the Rust backend and can take several minutes. A desktop
window opens when it's ready.

## 4. Log in

Two options on the login screen:

- **Login with GitHub** — the OAuth flow (needs step 2 configured).
- **Use a token instead** — paste a GitHub **Personal Access Token** (classic,
  with the `repo` scope) from <https://github.com/settings/tokens>. This is the
  quickest way to test and needs no OAuth app.

## 5. Test cleaning

1. Click **Select Folder** and pick any project directory (e.g. one with
   `node_modules/`, `.git/`, `__pycache__/`).
2. LidBridge scans it and shows file stats.
3. Click **Clean** — it writes a cleaned copy next to the original as
   `<project>_LidBridge/`, skipping junk dirs, build output, secrets, etc.
4. The original folder is never modified.

## 6. Test push (optional)

After cleaning, use **Push to GitHub** to create a repo and push the cleaned
copy. Requires a token/login with the `repo` scope (and `write:org` to push to
an organization).

## Production build

```bash
npm run tauri build
```

The installer/app bundle is written to `src-tauri/target/release/bundle/`.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `cargo` not found | Install Rust (above) and open a new terminal, or `source ~/.cargo/env`. |
| Build fails compiling a C dependency | Run `xcode-select --install`; if it mentions cmake, `brew install cmake`. |
| Login opens the browser but never completes | The OAuth **client secret** isn't set. Use the token option instead, or fill `.env` and rebuild. |
| "command not found" for a Tauri call | You're on an old build — rebuild with `npm run tauri dev`. |
