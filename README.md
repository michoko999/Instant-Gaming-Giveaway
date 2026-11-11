# Giveaway IG

<div align="center">

<img src="Magic.ico" width="96" alt="" />

**Enter Instant Gaming giveaways without spending your evening on it.**

English · [Français](README-FR.md)

</div>

Giveaway IG is a **portable** Windows app (no installer) that fetches the Instant Gaming giveaways that are still open, checks which ones are active and enters them for you, wrapped in a starry-night interface.

> [!NOTE]
> Open source project, **not affiliated with Instant Gaming**, with no affiliate or referral links.

## ✨ Features

- **Automatic mode**: sign in to your account once in the built-in Instant Gaming window. The app then opens each giveaway and clicks “Participate” for you.
  - If the site asks for a human verification (captcha), the giveaway is set aside for you to handle at the end. **The app never solves or bypasses captchas.**
  - If your account gets signed out, the session pauses until you sign in again.
  - The Instant Gaming window stays hidden while it works; “Watch the page live” shows it whenever you want (closing it just hides it again).
- **Manual mode**: giveaways open one by one; click “Participate” and the app moves on as soon as it notices.
- **Always up-to-date lists**: two built-in community lists and two web pages that share giveaways (jv-actu.com, ledemondujeu.com), plus your own files (CSV, TXT, JSON), online lists or any web page with Instant Gaming giveaway links. You can also paste links straight into the Giveaways screen.
- **Fast checking**: hundreds of giveaways tested in parallel in under a minute.
- **Monthly tracking**: each giveaway accepts one entry per month; the app skips the ones already done this month and keeps a history you can export as CSV, with a chart of your entries per month.
- **7 languages**: Français, English, Español, Deutsch, Português, Italiano, Polski.
- **Portable**: settings and session live in a `GiveawayIG-data` folder next to the executable.
- **Nothing is ever deleted**: ended giveaways stay in the list and are re-checked every week; if one comes back, it is picked up automatically.
- **One-click updates**: the app tells you when a new version is out; clicking “Update” downloads the new executable from the GitHub release, checks its SHA-256 fingerprint and replaces it. Nothing happens without your click.

## 📥 Download

1. Open the [Releases](https://github.com/michoko999/Instant-Gaming-Giveaway/releases) page and download `GiveawayIG.exe`.
2. Put it in a folder of your own, for example `Documents\GiveawayIG`.
3. Run it. On first launch a short wizard walks you through signing in, fetching the lists and choosing a mode.

Requirements: Windows 10 or 11 with Microsoft Edge WebView2 (built into Windows 11 and up-to-date Windows 10).

### Check that the file is genuine

Each release shows the SHA-256 hash of the executable and a GitHub **build provenance attestation** proving it was built by GitHub Actions from this repository's public code:

```powershell
Get-FileHash .\GiveawayIG.exe -Algorithm SHA256
gh attestation verify .\GiveawayIG.exe --repo michoko999/Instant-Gaming-Giveaway
```

## 🛡️ Antivirus and “Windows protected your PC”

Version 2 was designed so it no longer looks like malware:

- a real native executable (Rust + Tauri) that **does not unpack itself** into a temporary folder like the old PyInstaller build did;
- **no simulated keystrokes** and no keyboard hooks: clicks happen inside the app's own window;
- no writes to `%TEMP%`, no auto-start with Windows, no system changes.

Until the executable is signed (a free signing request to SignPath Foundation is in progress, see the [code signing policy](docs/CODE_SIGNING_POLICY.md)), Windows SmartScreen may show “Windows protected your PC” for a new version: it is a reputation warning, not a virus detection. If you downloaded the file from the Releases page and the hash matches, click “More info” then “Run anyway”.

If an antivirus still flags the app, please open an [issue](https://github.com/michoko999/Instant-Gaming-Giveaway/issues) with the detection name so it can be reported as a false positive (for Microsoft Defender: [submission portal](https://www.microsoft.com/wdsi/filesubmission)).

## ⚠️ Before using automatic mode

Giveaway rules often forbid entries made by “bots”. Using automatic mode may therefore get entries cancelled or even your Instant Gaming account sanctioned. **It is your call**; if in doubt, use manual mode.

**Careful mode**, on by default, lowers the risk without removing it:

- at least 8 seconds between giveaways, and a limited number of giveaways per session (40 by default);
- the session stops right away if the site shows a warning or asks for several human verifications;
- it also stops after several errors in a row;
- giveaways already entered this month are not reopened;
- list checks run without your account, at a moderate pace.

The app does not try to pass itself off as a human (no fake mouse movements, no captcha bypass).

## 🔒 Privacy

- You type your password directly on the Instant Gaming site, inside the built-in window. The app never sees or stores it.
- The session (cookies) stays on your PC in `GiveawayIG-data\webview`. “Sign out” in the settings clears it.
- No telemetry. The app only connects to:
  - `raw.githubusercontent.com` to download the giveaway lists;
  - `instant-gaming.com` to check and enter giveaways;
  - `api.github.com` to see whether a new version exists (can be turned off).

## 🛠️ Build from source

Requirements: [Node.js 22](https://nodejs.org), stable [Rust](https://rustup.rs) and, on Windows, the Visual Studio C++ build tools ([Tauri prerequisites](https://tauri.app/start/prerequisites/)).

```bash
npm ci
npm run tauri dev                 # run in development
npx tauri build --no-bundle       # executable: src-tauri/target/release/giveaway-ig.exe
```

Checks:

```bash
npm run check                                        # TypeScript / Svelte types
cargo test --manifest-path src-tauri/Cargo.toml      # Rust core tests
```

## 📁 Layout

```
src/                 Interface (Svelte 5 + TypeScript)
  screens/           Screens: wizard, home, session, recap, giveaways, history, settings
  lib/               Starry sky, ring, global state, translations (lib/i18n)
src-tauri/           Rust application (Tauri 2)
  src/sources.rs     List parsing (community JSON, CSV, imports)
  src/checker.rs     Giveaway status checks
  src/automation.rs  Instant Gaming window and entry script
  src/session.rs     Automatic and manual sessions
  src/store.rs       Local storage, history, stats
scripts/             Icon preparation from Magic.ico
legacy/              Previous Python version (v1.3)
List-Uncheck.csv     Community giveaway list (used by the app)
```

## 📜 License

[MIT](LICENSE) © 2023-2026 Michoko.
