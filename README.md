<div align="center">

<img src="docs/images/logo.png" width="104" alt="Giveaway IG logo" />

# Giveaway IG

**Enter Instant Gaming giveaways without spending your evening on it.**

<a href="https://github.com/michoko999/Instant-Gaming-Giveaway/releases/latest"><img src="https://img.shields.io/github/v/release/michoko999/Instant-Gaming-Giveaway?style=flat-square&label=version&color=FFD27A&labelColor=1E2140" alt="Latest version" /></a>
<img src="https://img.shields.io/badge/Windows-10%20%7C%2011-B9A6FF?style=flat-square&labelColor=1E2140" alt="Windows 10 and 11" />
<img src="https://img.shields.io/badge/languages-7-9FE8D5?style=flat-square&labelColor=1E2140" alt="7 languages" />
<a href="LICENSE"><img src="https://img.shields.io/github/license/michoko999/Instant-Gaming-Giveaway?style=flat-square&color=FFB199&labelColor=1E2140" alt="MIT license" /></a>

**English** · [Français](README-FR.md) · [Español](README-ES.md) · [Deutsch](README-DE.md) · [Português](README-PT.md) · [Italiano](README-IT.md) · [Polski](README-PL.md)

<br />

<a href="https://github.com/michoko999/Instant-Gaming-Giveaway/releases/latest"><img src="https://img.shields.io/badge/%E2%AC%87%20Download-GiveawayIG.exe-FFD27A?style=for-the-badge&labelColor=2A2D5A" alt="Download GiveawayIG.exe" /></a>

<sub>Portable · no installation · free and open source</sub>

<br /><br />

<img src="docs/images/screenshots/en-home.png" width="880" alt="Giveaway IG home screen: number of giveaways waiting, start buttons and this month's progress" />

</div>

<br />

Giveaway IG fetches the Instant Gaming giveaways that are still open, checks which ones are active and enters them for you, one page after another, in a starry-night interface. You sign in once; after that, a click on **Start automatic** is all it takes each month.

> [!NOTE]
> Open source project, **not affiliated with Instant Gaming**, with no affiliate or referral links.

## ✨ Features

<table>
<tr>
<td width="50%" valign="top">

### 🤖 Automatic mode
Sign in once in the built-in Instant Gaming window. The app opens each giveaway and clicks “Participate” for you, in a hidden window: **Watch the page live** shows it whenever you want.

</td>
<td width="50%" valign="top">

### 🖐️ Manual mode
Giveaways open one by one. You click “Participate” and the app moves on as soon as it notices. Handy if you prefer to stay in control.

</td>
</tr>
<tr>
<td valign="top">

### 🛡️ Careful mode
On by default: at least 8 s between giveaways, a limited number per session, and an immediate stop if the site shows a warning. Captchas are left to you, **never solved or bypassed**.

</td>
<td valign="top">

### 📚 Lists that stay up to date
Two community lists and two sites that share giveaways (jv-actu.com, ledemondujeu.com) are built in. Add your own: **Paste links**, a file, an online list or any web page.

</td>
</tr>
<tr>
<td valign="top">

### 📅 Your month at a glance
Instant Gaming accepts one entry per giveaway per month: the app skips the ones already done, keeps a history with a chart per month and exports it as CSV.

</td>
<td valign="top">

### ♻️ Nothing is ever lost
Ended giveaways stay in the list and are re-checked every week. If one comes back, it is picked up automatically.

</td>
</tr>
<tr>
<td valign="top">

### 🚀 One-click updates
When a new version is out, **Update** downloads it from this repository's releases, checks its SHA-256 fingerprint and restarts. Nothing happens without your click.

</td>
<td valign="top">

### 💼 Portable, in 7 languages
No installation: everything is stored in a `GiveawayIG-data` folder next to the executable. Français, English, Español, Deutsch, Português, Italiano, Polski.

</td>
</tr>
</table>

## 📸 Screenshots

<table>
<tr>
<td width="50%"><img src="docs/images/screenshots/en-giveaways.png" alt="Giveaways screen: list with status, search and filters" /></td>
<td width="50%"><img src="docs/images/screenshots/en-history.png" alt="History screen: entries per month chart and log" /></td>
</tr>
<tr>
<td align="center"><sub><b>Giveaways</b>: every known giveaway, its status and your last entry</sub></td>
<td align="center"><sub><b>History</b>: entries per month and full log</sub></td>
</tr>
<tr>
<td colspan="2"><img src="docs/images/screenshots/en-settings.png" alt="Settings screen: language, colors, session, giveaway sources" /></td>
</tr>
<tr>
<td colspan="2" align="center"><sub><b>Settings</b>: language, accent color, session pace, careful mode, sources</sub></td>
</tr>
</table>

## 🚀 Getting started

1. **Download** `GiveawayIG.exe` from the [Releases](https://github.com/michoko999/Instant-Gaming-Giveaway/releases/latest) page.
2. **Put it in a folder of your own**, for example `Documents\GiveawayIG`. Your settings will be saved next to it.
3. **Run it.** A short wizard guides you:
   - **Connect my account**: the Instant Gaming site opens in the app; sign in as usual (your password is typed on their site, never in the app);
   - the app fetches the lists and checks which giveaways are open;
   - choose your mode and your pace.
4. Click **Start automatic** (or **Manual mode**) and let it run. Come back next month 🌙

Requirements: Windows 10 or 11 with Microsoft Edge WebView2 (built into Windows 11 and up-to-date Windows 10).

## ⚠️ Before using automatic mode

Giveaway rules often forbid entries made by “bots”. Automatic mode may therefore get entries cancelled or even your Instant Gaming account sanctioned. **It is your call**; if in doubt, use manual mode.

**Careful mode** (Settings → *Careful mode (recommended)*) lowers the risk without removing it:

- at least 8 seconds between giveaways, and a limited number of giveaways per session (40 by default);
- the session stops right away if the site shows a warning or asks for several human verifications, and after several errors in a row;
- giveaways already entered this month are not reopened;
- list checks run without your account, at a moderate pace.

The app does not try to pass itself off as a human: no fake mouse movements, no captcha solving.

## 🛡️ Antivirus and “Windows protected your PC”

Version 2 was rebuilt so it no longer looks like malware:

- a real native executable (Rust + Tauri) that **does not unpack itself** into a temporary folder like the old PyInstaller build did;
- **no simulated keystrokes** and no keyboard hooks: clicks happen inside the app's own window;
- no writes to `%TEMP%`, no auto-start with Windows, no system changes.

Until the executable is signed (a free code-signing request to SignPath Foundation is planned, see the [code signing policy](docs/CODE_SIGNING_POLICY.md)), Windows SmartScreen may show “Windows protected your PC” for a new version. It is a reputation warning, not a virus detection: if the file comes from the Releases page and its fingerprint matches, click **More info** then **Run anyway**.

<details>
<summary><b>Check that the file is genuine</b></summary>
<br />

Each release shows the SHA-256 fingerprint of the executable and a GitHub **build provenance attestation** proving it was built by GitHub Actions from this repository's public code:

```powershell
Get-FileHash .\GiveawayIG.exe -Algorithm SHA256
gh attestation verify .\GiveawayIG.exe --repo michoko999/Instant-Gaming-Giveaway
```

</details>

If an antivirus still flags the app, please open an [issue](https://github.com/michoko999/Instant-Gaming-Giveaway/issues) with the detection name so it can be reported as a false positive (Microsoft Defender: [submission portal](https://www.microsoft.com/wdsi/filesubmission)).

## 🔒 Privacy

- You type your password on the Instant Gaming site, inside the built-in window. The app never sees or stores it.
- The session (cookies) stays on your PC in `GiveawayIG-data\webview`. **Sign out** in Settings clears it.
- No telemetry, no account, no ads. The app only connects to:

| Address | Why |
|:--|:--|
| `instant-gaming.com` | check and enter giveaways |
| `raw.githubusercontent.com`, `jv-actu.com`, `ledemondujeu.com` | download the giveaway lists (each source can be turned off) |
| `api.github.com`, `github.com` | see whether a new version exists and download it (can be turned off) |

## ❓ FAQ

<details>
<summary><b>Why can I only enter once a month?</b></summary>
<br />
That is Instant Gaming's rule: one entry per giveaway per month. The app keeps track and only reopens a giveaway in the following month.
</details>

<details>
<summary><b>How do I add giveaways that are not in the lists?</b></summary>
<br />
In <b>Giveaways</b>, click <b>Paste links</b> and paste Instant Gaming links or codes (any surrounding text is ignored). You can also import a file or add an online list or a web page in <b>Settings → Giveaway sources</b>. To share them with everyone, open an <a href="https://github.com/michoko999/Instant-Gaming-Giveaway/issues">issue</a> and they will be added to the community list.
</details>

<details>
<summary><b>What happens when a verification (captcha) appears?</b></summary>
<br />
The giveaway is set aside and the session continues. At the end, the summary lists the giveaways to finish by hand, with a button to open each page.
</details>

<details>
<summary><b>Where are my settings stored? How do I start over?</b></summary>
<br />
In the <code>GiveawayIG-data</code> folder next to <code>GiveawayIG.exe</code> (or in your Windows user folder if that location is read-only). To move the app, move both together. To start from scratch, close the app and delete that folder.
</details>

<details>
<summary><b>How do I update?</b></summary>
<br />
When a new version is published, a banner appears at the top of the app: click <b>Update</b>. You can also download the new <code>GiveawayIG.exe</code> by hand and replace the old one. Your data stays in place either way.
</details>

<details>
<summary><b>Where is the old Python version?</b></summary>
<br />
In the <a href="legacy/">legacy</a> folder. It is no longer maintained.
</details>

## 🤝 Contributing

- **Giveaways**: send new codes through an [issue](https://github.com/michoko999/Instant-Gaming-Giveaway/issues) or a pull request on [`List-Uncheck.csv`](List-Uncheck.csv). Codes are never removed from it, so a giveaway that comes back is picked up again.
- **Translations**: the texts live in [`src/lib/i18n`](src/lib/i18n), one file per language.
- **Bugs and ideas**: [issues](https://github.com/michoko999/Instant-Gaming-Giveaway/issues) are welcome.

## 🛠️ Build from source

Requirements: [Node.js 22](https://nodejs.org), stable [Rust](https://rustup.rs) and, on Windows, the Visual Studio C++ build tools ([Tauri prerequisites](https://tauri.app/start/prerequisites/)).

```bash
npm ci
npm run tauri dev                 # run in development
npx tauri build --no-bundle       # executable: src-tauri/target/release/giveaway-ig.exe
```

```bash
npm run check                                        # TypeScript / Svelte types
cargo test --manifest-path src-tauri/Cargo.toml      # Rust core tests
```

<details>
<summary><b>Project layout</b></summary>

```
src/                 Interface (Svelte 5 + TypeScript)
  screens/           Screens: wizard, home, session, recap, giveaways, history, settings
  lib/               Starry sky, ring, chart, global state, translations (lib/i18n)
src-tauri/           Rust application (Tauri 2)
  src/sources.rs     Lists: community JSON, CSV, web pages, pasted links
  src/checker.rs     Giveaway status checks
  src/automation.rs  Instant Gaming window and entry script
  src/session.rs     Automatic and manual sessions, careful mode
  src/store.rs       Local storage, history, statistics
  src/update.rs      Update check and installation
scripts/             Icon preparation from Magic.ico
docs/                Code signing policy, README images
legacy/              Previous Python version (v1.3)
List-Uncheck.csv     Community giveaway list (used by the app)
```

</details>

---

<div align="center">

<sub>[MIT](LICENSE) © 2023-2026 Michoko · Instant Gaming is a trademark of its owner; this project is not affiliated with it.</sub>

</div>
