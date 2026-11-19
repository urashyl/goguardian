# GoGuardian MV3 Technical Documentation (`goguardian-docs`)

Independent, public-interest technical documentation explaining what the GoGuardian Chrome Manifest V3 extension (`v4.1.246`) monitors, filters, and controls inside a managed browser session, served by a minimal zero-dependency Rust HTTP/1.1 backend.

## Pages Included

- **`/`** — **Overview & Architecture**: Extension permissions, 10 backend microservices, Pusher/Twilio control plane summary, and key takeaways.
- **`/telemetry`** — **Browsing, Screenshots, `about:blank` & Device Profiling**: `Zephyr` entity flushing, `captureVisibleTab`, `goguardian-21.js` (`html2canvas-pro`) vs. cross-origin iframes, `submitAboutBlankIframes` (`getAllFrames`), hardware/extension/device-ID profiling, Windows native host, `offscreen-document.js` account scraping, and the 25-second health check.
- **`/ai-and-docs`** — **AI Chatbots, Google Docs, YouTube & ML Classifiers**: Dual AI prompt capture (`19.js` + server-pushed `22.js` adapters), Google Docs `_docs_force_html_by_ext` canvas unwrapping, YouTube player hooks, Flagged Terms, Beacon, TF.js Explicit Image Filter, Teacher Off-Task ML (`TeacherPredictTaskiness`), and password/input capture limits.
- **`/filtering`** — **URL Filtering, Panther & SafeTranslate**: 4-layer `webRequest.onBeforeRequest` rules, `sub_frame` handling, Panther categorization, bypass passwords, interactive `*.translate.goog` SafeTranslate decoder, and `/v1/policy` structure.
- **`/teacher-control`** — **Teacher Live Control Plane**: `Inquisition` session polling, Pusher channel/event matrix, `LiveState v6`, Screen Lock, `beforeunload` override, Google Meet auto-mute, live screen annotations, announcements, check-ins, and Twilio WebRTC calls/presentations.
- **`/scripts`** — **Content Scripts & IPC Reference**: Searchable/filterable reference table for all 22 content scripts (`goguardian-1.js`–`22.js`) and the internal `chrome.runtime` IPC message bus.

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

## Copyright & Research Disclaimer

This repository contains **original documentation and an original Rust web server** written from static architectural analysis. **Proprietary GoGuardian source files, assets, model weights, tokens, and personal/school data are intentionally excluded** and are not redistributed due to copyright and privacy protections.

## Contact

For general research questions, corrections, or feedback, contact: **goguardian@mitch.pro**

## License

MIT — see [LICENSE](LICENSE).
