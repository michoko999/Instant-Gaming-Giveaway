# Giveaway IG

<div align="center">

<img src="Magic.ico" width="96" alt="" />

**Participe aux concours Instant Gaming sans y passer ta soirée.**

[English](README.md) · Français

</div>

Giveaway IG est une application Windows **portable** (aucune installation) qui récupère les concours Instant Gaming encore ouverts, vérifie lesquels sont actifs et y participe pour toi, avec une interface « ciel étoilé ».

> [!NOTE]
> Projet open source, **non affilié à Instant Gaming**, sans lien d'affiliation ni de parrainage.

## ✨ Fonctionnalités

- **Mode automatique** : tu te connectes une seule fois à ton compte dans la fenêtre Instant Gaming intégrée. Ensuite, l'application ouvre chaque concours et clique sur « Participer » à ta place.
  - Si le site demande une vérification humaine (captcha), le concours est mis de côté et tu le fais toi-même à la fin. **L'application ne résout ni ne contourne jamais de captcha.**
  - Si ton compte est déconnecté, la session se met en pause le temps que tu te reconnectes.
  - La fenêtre Instant Gaming reste cachée pendant qu'elle travaille ; « Voir la page en direct » l'affiche quand tu veux (la fermer la cache simplement).
- **Mode manuel** : les concours s'ouvrent un par un ; tu cliques sur « Participer » et l'application passe au suivant dès qu'elle détecte ta participation.
- **Listes toujours à jour** : deux listes communautaires intégrées, plus tes propres fichiers (CSV, TXT, JSON) ou listes en ligne.
- **Vérification rapide** : des centaines de concours testés en parallèle en moins d'une minute.
- **Suivi du mois** : chaque concours accepte une participation par mois ; l'application passe ceux déjà faits ce mois-ci et garde un historique exportable en CSV.
- **7 langues** : français, English, Español, Deutsch, Português, Italiano, Polski.
- **Portable** : tes réglages et ta session sont rangés dans un dossier `GiveawayIG-data` à côté de l'exécutable.
- **Mises à jour** : l'application te prévient quand une nouvelle version sort (elle ne télécharge rien toute seule).

## 📥 Télécharger

1. Va sur la page [Releases](https://github.com/michoko999/Instant-Gaming-Giveaway/releases) et télécharge `GiveawayIG.exe`.
2. Range-le dans un dossier à toi, par exemple `Documents\GiveawayIG`.
3. Lance-le. Au premier démarrage, l'assistant te guide : connexion à ton compte, récupération des listes, choix du mode.

Prérequis : Windows 10 ou 11 avec Microsoft Edge WebView2 (déjà présent sur Windows 11 et sur Windows 10 à jour).

### Vérifier que le fichier est authentique

Chaque release affiche l'empreinte SHA-256 de l'exécutable et une **attestation de provenance** GitHub, qui prouve qu'il a été compilé par GitHub Actions à partir du code public de ce dépôt :

```powershell
Get-FileHash .\GiveawayIG.exe -Algorithm SHA256
gh attestation verify .\GiveawayIG.exe --repo michoko999/Instant-Gaming-Giveaway
```

## 🛡️ Antivirus et « Windows a protégé votre ordinateur »

La version 2 a été pensée pour ne plus ressembler à un programme malveillant :

- un vrai exécutable natif (Rust + Tauri), qui **ne se décompresse pas** dans un dossier temporaire comme le faisait l'ancienne version PyInstaller ;
- **aucune simulation de touches** ni écoute du clavier : les clics se font à l'intérieur de la fenêtre de l'application ;
- aucune écriture dans `%TEMP%`, aucun démarrage automatique avec Windows, aucune modification du système.

Tant que l'exécutable n'est pas signé (demande de signature gratuite en cours auprès de SignPath Foundation, voir la [politique de signature](docs/CODE_SIGNING_POLICY.md)), Windows SmartScreen peut afficher « Windows a protégé votre ordinateur » pour une nouvelle version : c'est un avertissement de réputation, pas une détection de virus. Si tu as téléchargé le fichier depuis la page Releases et que l'empreinte correspond, clique sur « Informations complémentaires » puis « Exécuter quand même ».

Si un antivirus signale quand même l'application, merci d'ouvrir une [issue](https://github.com/michoko999/Instant-Gaming-Giveaway/issues) avec le nom de la détection : on la signalera comme faux positif à l'éditeur (pour Microsoft Defender : [portail de soumission](https://www.microsoft.com/wdsi/filesubmission)).

## ⚠️ À savoir avant d'utiliser le mode automatique

Les règlements de concours interdisent souvent les participations faites par un « robot ». Utiliser le mode automatique peut donc entraîner l'annulation de participations, voire une sanction sur ton compte Instant Gaming. **C'est à toi de décider** ; en cas de doute, utilise le mode manuel.

Le **mode prudent**, activé par défaut, limite les risques sans les supprimer :

- au moins 8 secondes entre deux concours, et un nombre limité de concours par session (40 par défaut) ;
- arrêt immédiat de la session si le site affiche un avertissement ou demande plusieurs vérifications humaines ;
- arrêt après plusieurs erreurs d'affilée ;
- les concours déjà faits ce mois-ci ne sont pas rouverts ;
- la vérification des listes se fait sans ton compte, à rythme modéré.

L'application ne cherche pas à se faire passer pour un humain (pas de faux mouvements de souris, pas de contournement de captcha).

## 🔒 Confidentialité

- Ton mot de passe est tapé directement sur le site d'Instant Gaming, dans la fenêtre intégrée. L'application ne le voit pas et ne le stocke pas.
- La session (cookies) reste sur ton PC, dans `GiveawayIG-data\webview`. « Se déconnecter » dans les paramètres l'efface.
- Aucune télémétrie. L'application se connecte uniquement à :
  - `raw.githubusercontent.com` pour télécharger les listes de concours ;
  - `instant-gaming.com` pour vérifier les concours et y participer ;
  - `api.github.com` pour savoir si une nouvelle version existe (désactivable).

## 🛠️ Compiler soi-même

Prérequis : [Node.js 22](https://nodejs.org), [Rust](https://rustup.rs) stable et, sous Windows, les outils de compilation C++ de Visual Studio ([prérequis Tauri](https://tauri.app/start/prerequisites/)).

```bash
npm ci
npm run tauri dev                 # lancer en développement
npx tauri build --no-bundle       # exécutable : src-tauri/target/release/giveaway-ig.exe
```

Vérifications :

```bash
npm run check                                        # types TypeScript / Svelte
cargo test --manifest-path src-tauri/Cargo.toml      # tests du cœur Rust
```

## 📁 Structure

```
src/                 Interface (Svelte 5 + TypeScript)
  screens/           Écrans : assistant, accueil, session, récap, concours, historique, paramètres
  lib/               Ciel étoilé, anneau, état global, traductions (lib/i18n)
src-tauri/           Application Rust (Tauri 2)
  src/sources.rs     Lecture des listes (JSON communautaire, CSV, imports)
  src/checker.rs     Vérification des concours
  src/automation.rs  Fenêtre Instant Gaming et script de participation
  src/session.rs     Déroulement des sessions automatique et manuelle
  src/store.rs       Stockage local, historique, statistiques
scripts/             Préparation des icônes à partir de Magic.ico
legacy/              Ancienne version Python (v1.3)
List-Uncheck.csv     Liste communautaire de concours (utilisée par l'application)
```

## 📜 Licence

[MIT](LICENSE) © 2023-2026 Michoko.
