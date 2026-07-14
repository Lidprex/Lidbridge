# LidBridge v2 — Implementation Record

## Completed

- Database initialization, central analytics, and database calls are disabled. The deprecated database source remains under `src-tauri/src/db` for reference.
- OAuth uses GitHub Device Authorization Flow with a public client ID only. The build script does not read or embed `.env` values. Sessions expire after seven days.
- Repository history is stored locally in encrypted files under the application data directory. Each encryption operation uses a fresh random nonce.
- Project scanning and cleaning run on blocking worker threads. Cleaning streams files one at a time instead of collecting and copying every file in a single parallel batch.
- The interface shows scan progress before cleaning is available, and secret detections are shown for review before output is published.
- The title bar contains history, AI, minimize, maximize, and close controls. Resize handles are provided around the undecorated window.
- App icons were regenerated from the supplied 1024px source for platform bundle sizes, and the same source is used in the UI.

## Before / After

| Area | Before | After |
| --- | --- | --- |
| Persistence | Active SQLite and optional central database paths | Encrypted local session/history only |
| Scan/clean work | UI could appear idle; cleaning accumulated work | Visible progress; worker-thread and streamed copying |
| OAuth | Environment values could be embedded at build time | Public device-flow client ID only; no `.env` embedding |
| Encryption | Reused nonce | Random nonce per encrypted payload |

## Outstanding

- AI analysis remains a UI-only Copilot placeholder; implementing Copilot requests requires a separate user-authorized API/service design.
- The secret review supports replacement preparation. A file-scoped editor and automatic context-aware placeholders remain future work.
- Repository license, tag, and type selection are not yet connected to the GitHub create/push command.
