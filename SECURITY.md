# Security Policy

## Reporting Vulnerabilities

If you discover a security vulnerability in LidBridge, please report it responsibly.

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, please email: [security@lidprex.onrender.com](mailto:security@lidprex.gmail.com)

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours and work with you to resolve the issue.

## Supported Versions

| Version | Supported |
|---|---|
| v2.0.x | Yes |
| v1.0.x | No |

## Security Measures

### OAuth Credentials
- GitHub OAuth Client ID and Secret are stored in `.env` — never hardcoded in source code
- `.env` is listed in `.gitignore` and will never be committed to the repository
- Users must create their own OAuth App or use a Personal Access Token

### Secret Detection
LidBridge scans projects for exposed secrets before pushing:
- API keys (GitHub, OpenAI, AWS, Google, Stripe, Discord, Slack)
- Passwords and tokens in code
- Secret files (`.env`, `credentials.json`, SSH keys, certificates)
- Private keys and certificates

### Data Privacy
- LidBridge runs entirely on your local machine
- No data is sent to external servers (except GitHub API for authentication and push)
- The optional PostgreSQL analytics database is **deprecated** in v2.0.0
- Session tokens are stored locally in SQLite

### Build Integrity
- Release builds are compiled with `strip = true` and `lto = true` for optimization
- GitHub Actions workflows build for all platforms from the same source code
- All builds are attached to GitHub Releases with version tags
