# Code signing policy / Politique de signature

## Status

The Windows executable is **not signed yet**. A free code signing request for open source projects is being prepared with [SignPath Foundation](https://signpath.org/).

Once approved, this page will carry the required statement:
*Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org/).*

Until then, every release provides:

- the SHA-256 hash of `GiveawayIG.exe`;
- a GitHub build provenance attestation (`gh attestation verify GiveawayIG.exe --repo michoko999/Instant-Gaming-Giveaway`), which proves the file was built by GitHub Actions from this repository.

## What gets signed

Only `GiveawayIG.exe`, built by the [Release workflow](../.github/workflows/release.yml) from a `v2.x.y` tag of this repository. Nothing is signed from a personal computer.

## Team and roles

| Role | Members |
| --- | --- |
| Authors / committers | [michoko999](https://github.com/michoko999) |
| Reviewers (for outside contributions) | [michoko999](https://github.com/michoko999) |
| Approvers (each signing request) | [michoko999](https://github.com/michoko999) |

All team members use multi-factor authentication on GitHub (and on SignPath once set up).

## Privacy policy

This program does not collect or send any personal data or telemetry. It only connects to other systems to do what the user asked it to do:

- `raw.githubusercontent.com`: download the giveaway lists enabled in the settings;
- `instant-gaming.com`: check giveaway pages and, in the built-in window, enter giveaways with the user's own session;
- `api.github.com`: check whether a newer version has been published (can be turned off in the settings).

The user's Instant Gaming password is typed on the Instant Gaming website and is never read or stored by the program. Settings, history and the browser session are stored locally, next to the executable (`GiveawayIG-data`) or in the user's local application data folder.
