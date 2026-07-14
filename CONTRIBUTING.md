# Contributing to LidBridge

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

### Prerequisites
- [Node.js](https://nodejs.org) >= 18
- [Rust](https://rustup.rs) (stable)
- [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/)

### Getting Started

```bash
git clone https://github.com/lidprex/lidbridge.git
cd lidbridge
cp .env.example .env
# Edit .env with your GitHub OAuth credentials (or skip and use Personal Token login)
npm install
npx tauri dev
```

## Project Structure

```
src/                    # Next.js + React frontend
  app/
    page.tsx            # Main UI + translations
    components/
      dashboard-ui.tsx  # Dashboard components
    globals.css         # Design tokens

src-tauri/              # Rust backend
  src/
    lib.rs              # Tauri commands + entry point
    auth/mod.rs         # GitHub OAuth
    cleaner/mod.rs      # Project scanning + secret detection
    git/mod.rs          # Push via GitHub API
    db/                 # SQLite (PostgreSQL deprecated)
```

## How to Contribute

### Bug Reports
Open an issue using the **Bug Report** template.

### Feature Requests
Open an issue using the **Feature Request** template.

### Code Contributions

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Test on your platform
5. Commit with a clear message
6. Push and open a Pull Request

### Translations

To add or update a language:
1. Open `src/app/page.tsx`
2. Find the `translations` object
3. Add or update the language entry
4. All translation keys must be present

Current languages: English, Arabic, Russian, French, Hindi, Chinese.

## Code Style

### Rust
- Follow standard `rustfmt` formatting
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings

### TypeScript/React
- Follow the existing code patterns
- No inline comments unless necessary
- Use Tailwind CSS classes

### Commits
Use [Conventional Commits](https://www.conventionalcommits.org/):
- `feat: add new language support`
- `fix: resolve push failure on Windows`
- `docs: update README`
- `refactor: clean up auth module`

## Testing

Before submitting a PR:
- [ ] `cargo fmt` passes
- [ ] `cargo test` passes
- [ ] `npx tauri dev` runs without errors
- [ ] Manual testing on your OS
- [ ] No secrets or API keys in code

## License

By contributing, you agree that your contributions will be licensed under the GPL v3.0.
