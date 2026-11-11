# Code signing policy / Politique de signature

## Status

The Windows executable is **not signed yet**. A free code signing request for open source projects has been prepared for [SignPath Foundation](https://signpath.org/).

Once approved, this page will carry the required statement:
*Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org/).*

Until then, every release provides:

- `SHA256SUMS.txt`, with the SHA-256 hash of each file (also shown in the release notes);
- a GitHub build provenance attestation for each file (`gh attestation verify <file> --repo michoko999/Instant-Gaming-Giveaway`), which proves it was built by GitHub Actions from this repository.

## What gets signed

Only `GiveawayIG.exe` (Windows), built by the [Release workflow](../.github/workflows/release.yml) from this repository's `main` branch and tagged `v2.x.y`. The Linux AppImage is not Authenticode-signed; its hash and provenance attestation are published with each release. Nothing is built or signed from a personal computer, and no third-party binary is signed.

## Team and roles

| Role | Members |
| --- | --- |
| Authors / committers | [michoko999](https://github.com/michoko999) |
| Reviewers (for outside contributions) | [michoko999](https://github.com/michoko999) |
| Approvers (each signing request) | [michoko999](https://github.com/michoko999) |

All team members use multi-factor authentication on GitHub (and on SignPath once set up). Outside contributions go through pull requests and are reviewed before being merged.

## Privacy policy

This program does not collect or send any personal data or telemetry, and has no account or ads. It only connects to other systems to do what the user asked it to do:

| Address | Why |
| --- | --- |
| `instant-gaming.com` | check giveaway pages and, in the built-in window, enter giveaways with the user's own session |
| `gaming-cdn.com` | download the profile picture of the user's Instant Gaming account |
| `raw.githubusercontent.com`, `jv-actu.com`, `ledemondujeu.com` | download the giveaway lists enabled in the settings (each source can be turned off) |
| `api.github.com`, `github.com` | check whether a newer version exists and, only when the user clicks **Update**, download it (can be turned off in the settings) |

Lists added by the user are downloaded from the addresses the user typed. The user's Instant Gaming password is typed on the Instant Gaming website and is never read or stored by the program. Settings, history and the browser session are stored locally, next to the executable in `GiveawayIG-data` (or in the user's local application data folder if that folder is read-only); deleting that folder removes everything. The program makes no system changes, does not start with the system and needs no installation, so there is nothing to uninstall.
