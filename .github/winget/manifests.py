"""Écrit les fichiers winget (manifestes) d'une version publiée de Giveaway IG.

Utilisé par le workflow « Winget » :
    python manifests.py <tag> <sha256 de GiveawayIG-setup.exe> <date AAAA-MM-JJ> <owner/repo> <dossier>
"""

import pathlib
import sys

IDENTIFIER = "Michoko999.GiveawayIG"
SCHEMA = "1.12.0"


def manifests(tag: str, sha256: str, date: str, repo: str) -> dict[str, str]:
    version = tag.lstrip("v")
    home = f"https://github.com/{repo}"
    head = f"PackageIdentifier: {IDENTIFIER}\nPackageVersion: {version}\n"

    def schema(kind: str) -> str:
        return f"# yaml-language-server: $schema=https://aka.ms/winget-manifest.{kind}.{SCHEMA}.schema.json\n\n"

    return {
        f"{IDENTIFIER}.yaml": schema("version")
        + head
        + f"DefaultLocale: en-US\nManifestType: version\nManifestVersion: {SCHEMA}\n",
        f"{IDENTIFIER}.installer.yaml": schema("installer")
        + head
        + "InstallerType: nullsoft\n"
        + "Scope: user\n"
        + "InstallModes:\n- interactive\n- silent\n- silentWithProgress\n"
        + "UpgradeBehavior: install\n"
        + "ProductCode: Giveaway IG\n"
        + f"ReleaseDate: {date}\n"
        + "Installers:\n"
        + "- Architecture: x64\n"
        + f"  InstallerUrl: {home}/releases/download/{tag}/GiveawayIG-setup.exe\n"
        + f"  InstallerSha256: {sha256.upper()}\n"
        + f"ManifestType: installer\nManifestVersion: {SCHEMA}\n",
        f"{IDENTIFIER}.locale.en-US.yaml": schema("defaultLocale")
        + head
        + "PackageLocale: en-US\n"
        + "Publisher: Michoko999\n"
        + "PublisherUrl: https://github.com/michoko999\n"
        + f"PublisherSupportUrl: {home}/issues\n"
        + "PackageName: Giveaway IG\n"
        + f"PackageUrl: {home}\n"
        + "License: MIT\n"
        + f"LicenseUrl: {home}/blob/main/LICENSE\n"
        + "Copyright: Copyright (c) 2023-2026 Michoko999\n"
        + "ShortDescription: Keeps track of Instant Gaming giveaways and helps you enter them every month.\n"
        + "Description: |-\n"
        + "  Giveaway IG gathers Instant Gaming giveaway links from community lists, checks which ones are still open and keeps track of the ones each account has already entered this month.\n"
        + "  Giveaways open in a built-in window where you sign in on the official site; you can enter them yourself (manual mode) or let the app click the site's own button at a careful pace (automatic mode).\n"
        + "  No password handling, no captcha solving, no telemetry. Available in 7 languages.\n"
        + "Moniker: giveaway-ig\n"
        + "Tags:\n- giveaway\n- instant-gaming\n- games\n- steam\n- tauri\n"
        + f"ReleaseNotesUrl: {home}/releases/tag/{tag}\n"
        + f"ManifestType: defaultLocale\nManifestVersion: {SCHEMA}\n",
        f"{IDENTIFIER}.locale.fr-FR.yaml": schema("locale")
        + head
        + "PackageLocale: fr-FR\n"
        + "Publisher: Michoko999\n"
        + "PackageName: Giveaway IG\n"
        + "License: MIT\n"
        + "ShortDescription: Suit les concours Instant Gaming et t'aide à y participer chaque mois.\n"
        + "Description: |-\n"
        + "  Giveaway IG récupère les liens des concours Instant Gaming dans des listes communautaires, vérifie lesquels sont encore ouverts et retient ceux auxquels chaque compte a déjà participé ce mois-ci.\n"
        + "  Les concours s'ouvrent dans une fenêtre intégrée où tu te connectes sur le site officiel ; tu participes toi-même (mode manuel) ou l'application clique sur le bouton du site à un rythme prudent (mode automatique).\n"
        + "  Aucun mot de passe manipulé, aucun captcha contourné, aucune télémétrie. Disponible en 7 langues.\n"
        + f"ManifestType: locale\nManifestVersion: {SCHEMA}\n",
    }


def main() -> None:
    tag, sha256, date, repo, out = sys.argv[1:6]
    if len(sha256) != 64 or not all(c in "0123456789abcdefABCDEF" for c in sha256):
        sys.exit(f"Empreinte SHA-256 invalide : {sha256!r}")
    folder = pathlib.Path(out)
    folder.mkdir(parents=True, exist_ok=True)
    for name, text in manifests(tag, sha256, date, repo).items():
        (folder / name).write_text(text, encoding="utf-8", newline="\n")
        print(f"{name} écrit")


if __name__ == "__main__":
    main()
