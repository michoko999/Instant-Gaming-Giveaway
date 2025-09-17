# Outil de Giveaway Instant Gaming

<## 🛠️ Compilation en exécutable
> [!NOTE]
> Pour obtenir un format exécutable portable, l'application est empaquetée avec PyInstaller dans un fichier `EXE`. Certains moteurs antivirus (y compris Windows Defender) peuvent signaler l'exécutable empaqueté comme un cheval de Troie, car PyInstaller a déjà été utilisé par d'autres pour empaqueter du code Python malveillant. Ces alertes peuvent être ignorées sans danger. Si vous ne faites absolument pas confiance à l'exécutable, vous devrez installer Python vous-même et tout exécuter depuis le code source.

> [!IMPORTANT]
> L'exécutable sera créé pour votre plateforme actuelle :
> - **Windows** : Crée `Giveaway_IG.exe` avec support d'icône
> - **Linux/macOS** : Crée `Giveaway_IG` (sans extension) - l'icône et le fichier de version ne sont pas supportés sur ces plateformes

Si vous souhaitez créer une version exécutable de l'application :lign="center">

📖 Ce README est disponible en :
[<img src="https://upload.wikimedia.org/wikipedia/en/a/a4/Flag_of_the_United_States.svg" height="15"> English](README.md) |
[<img src="https://upload.wikimedia.org/wikipedia/en/c/c3/Flag_of_France.svg" height="15"> Français](README-FR.md)

</div>

## Présentation
L'outil de Giveaway Instant Gaming est un script Python automatisé qui vous aide à participer aux concours Instant Gaming en vérifiant périodiquement la liste des concours disponibles. Parfait pour les joueurs qui veulent augmenter leurs chances de gagner des jeux !

## ✨ Fonctionnalités

- 🌐 **Prend en charge plusieurs langues**:
  🇬🇧 English | 🇫🇷 Français | 🇪🇸 Español | 🇩🇪 Deutsch | 🇵🇹 Português | 🇮🇹 Italiano | 🇵🇱 Polski
- ⚡ **Vérification asynchrone des concours** - Plus rapide que jamais !
- 🎨 **Interface colorée** - Visuellement plus agréable et informative
- 📊 **Barre de progression** - Suivez l'avancement du processus
- ⏰ **Intervalles de vérification personnalisables**
- 🔄 **Sauvegarde des préférences** - Configuration persistante
- 📝 **Historique des participations** - Gardez une trace de vos concours
- 💪 **Robuste** - Meilleure gestion des erreurs
- ✅ **Validation intelligente des concours**:
  - Détecte automatiquement les concours actifs
  - Vérifie la disponibilité des concours
  - Ignore les concours expirés ou invalides

## 📋 Prérequis
- Python 3.8 ou supérieur
- Navigateur web (Chrome recommandé)
- Connexion internet

## 📥 Installation
1. Assurez-vous d'avoir Python installé sur votre ordinateur
2. Exécutez `install library.bat` pour installer les dépendances requises
3. Ou installez manuellement les dépendances avec : `pip install -r requirements.txt`

## 🛠️ Compilation en exécutable
> [!NOTE]
> Pour obtenir un format exécutable portable, l’application est empaquetée avec PyInstaller dans un fichier `EXE`. Certains moteurs antivirus (y compris Windows Defender) peuvent signaler l’exécutable empaqueté comme un cheval de Troie, car PyInstaller a déjà été utilisé par d’autres pour empaqueter du code Python malveillant. Ces alertes peuvent être ignorées sans danger. Si vous ne faites absolument pas confiance à l’exécutable, vous devrez installer Python vous-même et tout exécuter depuis le code source.

Si vous souhaitez créer une version exécutable (.exe) de l'application :

1. Assurez-vous d'avoir PyInstaller installé :
   ```
   pip install pyinstaller
   ```

2. Compilez l'application en utilisant :
   ```
   pyinstaller --onefile --icon=Magic.ico --name="Giveaway_IG" --add-data "config.ini;." --add-data "traduction.json;." --add-data "List-Uncheck.csv;." --version-file=file_version_info.txt giveaway_ig.py
   ```

3. Trouvez l'exécutable compilé dans le dossier `dist`

## 🚀 Utilisation
> [!NOTE]
> Ce programme ne contient aucun lien d'affiliation avec Instant Gaming, contrairement à d'autres outils/listes de concours.

1. Lancez le programme en utilisant `run.bat` ou en exécutant `python giveaway_ig.py`
2. Sélectionnez votre langue préférée
3. Choisissez un fichier CSV contenant les URLs des concours (par défaut : List-Uncheck.csv)
4. Définissez l'intervalle de temps entre chaque URL
5. Le programme ouvrira chaque URL de concours valide dans votre navigateur
6. Cliquez sur le bouton orange "Participer" au centre de chaque page

## 📁 Structure des fichiers
- `giveaway_ig.py` : Fichier principal du programme
- `traduction.json` : Contient les traductions pour plusieurs langues
- `List-Uncheck.csv` : Liste par défaut des URLs à vérifier
- `valid_urls.csv` : Liste des URLs de concours valides
- `invalid_urls.csv` : Liste des URLs expirées ou invalides
- `unknown_urls.csv` : Liste des URLs avec un statut inconnu

## ❓ Dépannage
- Si vous rencontrez des problèmes avec le chargement des traductions, vérifiez le format JSON dans les fichiers de traduction
- Assurez-vous que vos fichiers CSV sont correctement formatés avec une URL par ligne
- Vérifiez que vous disposez d'une connexion internet stable

## 📄 Licence
Ce projet est sous licence MIT. Consultez le fichier LICENSE pour plus de détails.

---
Pour la version anglaise de cette documentation, cliquez sur le badge English en haut de ce document ou accédez à la [documentation en anglais (README.md)](README.md).
