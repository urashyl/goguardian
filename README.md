# GoGuardian MV3 Technical Documentation (`goguardian-docs`)

Independent, public-interest documentation explaining what the GoGuardian Chrome Manifest V3 extension (`v4.1.246`) monitors, filters, and controls inside a managed browser session. Includes a plain-English **Simple** view for parents and students plus a toggleable **Advanced** view with technical details, served by a minimal zero-dependency Rust HTTP/1.1 backend.

## Pages Included

- **`/`** — **Overview**: Plain-English parent takeaways (3-second live capture, 24/7 off-campus scheduling, browser webcam preview screenshots, Google Docs & AI reading, myth checks, questions for school districts) plus Advanced Manifest V3 permissions & 10-microservice topology.
- **`/telemetry`** — **Screens, Webcams & Privacy**: `Zephyr` entity flushing, `captureVisibleTab`, webcam preview screenshots vs. `chrome://camera-app`, background-tab focus-switching, `goguardian-21.js` (`html2canvas-pro`) vs. cross-origin iframes, `submitAboutBlankIframes` (`getAllFrames`), hardware/extension/device-ID profiling, Windows native host, `offscreen-document.js` account scraping, and the 25-second health check.
- **`/ai-and-docs`** — **AI Chatbots, Google Docs, YouTube & ML Classifiers**: Dual AI prompt capture (`19.js` + server-pushed `22.js` adapters), Google Docs `_docs_force_html_by_ext` canvas unwrapping, YouTube player hooks, Flagged Terms, Beacon, TF.js Explicit Image Filter, Teacher Off-Task ML (`TeacherPredictTaskiness`), and password/input capture limits.
- **`/filtering`** — **URL Filtering, Panther & SafeTranslate**: 4-layer `webRequest.onBeforeRequest` rules, `sub_frame` handling, Panther categorization, bypass passwords, interactive `*.translate.goog` SafeTranslate decoder, and `/v1/policy` structure.
- **`/teacher-control`** — **Teacher Live Control Plane**: `Inquisition` session polling, Pusher channel/event matrix, `LiveState v6`, Screen Lock, `beforeunload` override, Google Meet auto-mute, live screen annotations, announcements, check-ins, and Twilio WebRTC calls/presentations.
- **`/methodology`** — **How This Information Was Obtained**: Plain-English overview and sanitized 6-stage offline ChromeOS Flex forensic pipeline (user-space `debugfs` `ext4` extraction, `password.gaia` vs. TPM, Cryptohome `scrypt` + `HKDF-SHA256` + `AES-256-GCM` `USS` unwrapping, FlatBuffer `fscrypt` v2 verification, offline `AES-256-CTS`/`AES-256-XTS` decryption, LevelDB & extension audit).
- **`/scripts`** — **Content Scripts & IPC Reference**: Searchable/filterable reference table for all 22 content scripts (`goguardian-1.js`–`22.js`) and the internal `chrome.runtime` IPC message bus.

## Reading Modes & Deployment

- **Simple / Advanced Toggle**: Pages default to a plain-English **Simple** view and persist the user's choice (`Simple` vs. `Advanced`) via `localStorage` or `?mode=advanced` / `?mode=simple`.
- **Static Hosting**: The files in `static/` are completely self-contained and can also be deployed directly on static hosts (Cloudflare Pages, Netlify, GitHub Pages, or a custom domain) if you don't need the Rust server routes (`/api/health`, `/api/meta`, extensionless URL mapping).

## Running Locally

Requires Rust (`rustc` / `cargo`):

```bash
cargo test
cargo run
```

By default, the server listens on `http://127.0.0.1:8080`. You can override the bind address and port via CLI arguments or environment variables:

```bash
PORT=3000 HOST=0.0.0.0 cargo run
# or
cargo run -- --host 127.0.0.1 --port 3000
```

Read-only backend endpoints:
- `GET /api/health` — Backend status check (`{"status":"ok","service":"goguardian-docs","backend":"rust"}`)
- `GET /api/meta` — High-level architectural counters

## Running 24/7 with systemd (User Service)

To run the documentation server continuously under `~/goguardian` on port `8080`:

```bash
git clone https://github.com/urashyl/goguardian.git ~/goguardian
cd ~/goguardian
cargo build --release
mkdir -p ~/.config/systemd/user
cp goguardian-docs.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now goguardian-docs.service
loginctl enable-linger "$USER"
```

Note: [goguardian-docs.service](goguardian-docs.service) binds `127.0.0.1:8080` by default for use behind a reverse proxy (Caddy, Nginx, or Cloudflare Tunnel). Change `Environment=HOST=127.0.0.1` to `Environment=HOST=0.0.0.0` if you want to expose port `8080` directly.

## Copyright & Research Disclaimer

This repository contains **original documentation and an original Rust web server** written from static architectural analysis. **Proprietary GoGuardian source files, assets, model weights, tokens, and personal/school data are intentionally excluded** and are not redistributed due to copyright and privacy protections.

## Contact

For general research questions, corrections, or feedback, contact: **goguardian@urashyl.dev**

## License

MIT — see [LICENSE](LICENSE).
