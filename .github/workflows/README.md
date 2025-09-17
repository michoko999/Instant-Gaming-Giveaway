# GitHub Actions Workflows Documentation

Ce dossier contient les workflows GitHub Actions pour automatiser la compilation, les tests et les releases du projet Instant Gaming Giveaway Tool.

## 📁 Workflows disponibles

### 1. `build-and-test.yml` - Build et Test Principal
**Déclencheurs :**
- Push sur `main` et `develop`
- Pull requests vers `main`
- Déclenchement manuel

**Actions :**
- ✅ Compile l'exécutable Windows (`.exe`)
- ✅ Compile l'exécutable Linux
- ✅ Teste que les exécutables fonctionnent (smoke tests)
- ✅ Upload des artifacts pour téléchargement
- ✅ Rapport de synthèse des builds

### 2. `build-on-commits.yml` - Build Conditionnel
**Déclencheurs :**
- Push sur `main`
- Déclenchement manuel

**Actions :**
- 🔍 Compte les commits depuis le dernier build
- 🚀 Lance un build tous les 5 commits
- 🏷️ Crée des tags pour tracker les builds

### 3. `code-quality.yml` - Qualité du Code
**Déclencheurs :**
- Push sur `main` et `develop` (fichiers `.py`)
- Pull requests vers `main` (fichiers `.py`)
- Déclenchement manuel

**Actions :**
- 📊 Analyse PyLint (score de qualité)
- 🔍 Analyse Flake8 (style de code)
- 🛡️ Analyse de sécurité Bandit
- 🔒 Vérification des dépendances vulnérables
- ⚡ Tests de performance basiques

### 4. `release.yml` - Création de Releases
**Déclencheurs :**
- Tags de version (`v*` ex: `v1.3.0`)
- Déclenchement manuel avec paramètres

**Actions :**
- 🏗️ Compile pour Windows, Linux et macOS
- 📦 Crée une release GitHub automatique
- 📝 Génère des notes de release
- ⬆️ Upload des exécutables dans la release

## 🚀 Comment utiliser ces workflows

### Build automatique à chaque push
Les workflows se déclenchent automatiquement quand vous pushez du code :

```bash
git add .
git commit -m "Amélioration du code"
git push origin main
```

### Build tous les 5 commits
Le workflow `build-on-commits.yml` compte automatiquement les commits et lance un build complet tous les 5 commits.

### Créer une release
Pour créer une nouvelle version :

**Option 1: Avec un tag**
```bash
git tag v1.3.0
git push origin v1.3.0
```

**Option 2: Manuellement**
1. Allez dans l'onglet "Actions" de votre repo GitHub
2. Sélectionnez "Create Release with Executables"
3. Cliquez "Run workflow"
4. Entrez la version (ex: 1.3.0)
5. Lancez le workflow

### Tests de qualité
Les tests de qualité se lancent automatiquement et génèrent des rapports détaillés :
- Score PyLint (objectif : >6.0/10)
- Conformité aux standards Python
- Analyse de sécurité
- Performance

## 📊 Visualisation des résultats

### Dans l'interface GitHub
- **Actions** tab : Voir tous les workflows et leur statut
- **Releases** : Télécharger les exécutables compilés
- **Pull Requests** : Checks automatiques sur les PRs

### Artifacts disponibles
Après chaque build, vous pouvez télécharger :
- `giveaway-ig-windows-exe` : Exécutable Windows
- `giveaway-ig-linux-binary` : Exécutable Linux
- `code-quality-reports` : Rapports d'analyse
- `build-logs-*` : Logs en cas d'erreur

## 🔧 Configuration requise

### Permissions GitHub
Les workflows suivent le principe de **moindre privilège** avec des permissions explicites :

**🔒 Permissions Read-Only (workflows d'analyse) :**
- `contents: read` - Lecture du code source uniquement
- `actions: read` - Lecture des actions et workflows

**✏️ Permissions Write Limitées (workflow de release) :**
- `contents: write` - Nécessaire pour créer des releases et uploader des assets
- `actions: read` - Lecture des actions et workflows

Cette approche sécurisée limite les risques d'exploitation malveillante et respecte les standards de sécurité industriels.

### Secrets (optionnels)
- `GITHUB_TOKEN` : Automatiquement fourni par GitHub

### Variables d'environnement
Aucune configuration spéciale requise.

## 🐛 Dépannage

### Build qui échoue
1. Vérifiez les logs dans l'onglet Actions
2. Téléchargez les build logs depuis les artifacts
3. Vérifiez que `requirements.txt` est à jour

### Tests qui échouent
- PyLint score trop bas : Améliorez la qualité du code
- Tests de sécurité : Corrigez les problèmes détectés
- Smoke tests : Vérifiez que le programme démarre correctement

### Pas de build tous les 5 commits
- Vérifiez que le workflow `build-on-commits.yml` est actif
- Les tags de build sont utilisés pour compter les commits

## 📈 Métriques et monitoring

### Indicateurs de qualité suivis
- **PyLint Score** : Qualité du code (objectif: >6.0/10)
- **Build Success Rate** : Taux de réussite des builds
- **Executable Size** : Taille des exécutables générés
- **Build Time** : Temps de compilation

### Historique
- Tous les builds sont archivés pendant 30-90 jours
- Les releases sont permanentes
- Les rapports de qualité sont conservés 30 jours

## 🔄 Maintenance des workflows

### Mise à jour des versions Python
Modifiez la ligne `python-version: '3.12'` dans les workflows.

### Ajout de nouveaux tests
Modifiez `code-quality.yml` pour ajouter de nouveaux outils d'analyse.

### Changement des déclencheurs
Modifiez les sections `on:` de chaque workflow selon vos besoins.

---

**💡 Conseil :** Surveillez régulièrement l'onglet Actions pour vous assurer que tous les workflows fonctionnent correctement et que la qualité du code reste élevée.
