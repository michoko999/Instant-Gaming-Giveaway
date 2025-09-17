# -*- coding: utf-8 -*-
"""
Automated tool to participate in Instant Gaming giveaways.

This tool helps users automatically check and participate in Instant Gaming contests
by validating URLs, managing participation history, and opening valid giveaway links
in the browser with customizable intervals.
"""
# Version: 1.3.0
# Auteur: Michoko
# Date de création 18/02/2023
# https://github.com/michoko999

# Standard library imports
import asyncio
import configparser
import csv
import json
import logging
import os
import platform
import random
import re
import shutil
import sys
import tempfile
import time
import webbrowser
from datetime import datetime
from urllib.parse import urlparse

# Third-party imports
import aiohttp
import keyboard
import requests
from colorama import init, Fore, Style
from tqdm import tqdm


# Initialisation de colorama pour les couleurs dans le terminal
init()

# La configuration du logging sera définie plus tard avec le chemin du fichier de log

# Variables globales
VERSION = "1.3.0"
TRANSLATIONS = None

def is_valid_instant_gaming_url(url):
    """Valide de manière sécurisée si une URL appartient à Instant Gaming"""
    try:
        parsed_url = urlparse(url)
        # Vérifier que le scheme est https ou http
        if parsed_url.scheme not in ['http', 'https']:
            return False

        # Vérifier que le hostname est exactement instant-gaming.com ou ses sous-domaines légitimes
        hostname = parsed_url.hostname
        if not hostname:
            return False

        # Accepter instant-gaming.com et ses sous-domaines (comme www.instant-gaming.com)
        return hostname == 'instant-gaming.com' or hostname.endswith('.instant-gaming.com')
    except Exception:
        return False

# Fonction pour obtenir les chemins corrects des ressources, que ce soit en mode développement ou en mode exécutable
def resource_path(relative_path):
    """ Récupère le chemin absolu vers une ressource, fonctionne pour le développement et pour PyInstaller """
    try:
        # PyInstaller crée un dossier temporaire et stocke le chemin dans _MEIPASS
        base_path = getattr(sys, '_MEIPASS',
                          os.path.dirname(os.path.abspath(__file__)))
        path = os.path.join(base_path, relative_path)
        logging.debug("Chemin de ressource calculé: %s (base_path: %s, relative_path: %s)",
                     path, base_path, relative_path)
        return path
    except Exception as exc:
        logging.debug("Erreur lors du calcul du chemin de ressource: %s", str(exc))
        base_path = os.path.dirname(os.path.abspath(__file__))
        path = os.path.join(base_path, relative_path)
        logging.debug("Chemin de ressource alternatif: %s", path)
        return path

def get_writable_path(filename):
    """Récupère un chemin accessible en écriture pour les fichiers de configuration et logs"""
    # En mode exécutable, utiliser le dossier TEMP du système
    if getattr(sys, 'frozen', False):
        # Créer un sous-dossier spécifique dans le dossier TEMP
        app_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
        os.makedirs(app_dir, exist_ok=True)
        return os.path.join(app_dir, filename)
    # En mode développement, utiliser le répertoire courant
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), filename)

# Chemin pour les ressources en lecture seule
TRADPATH = resource_path("traduction.json")

# Chemins pour les fichiers qui doivent être modifiables
CONFIG_FILE = get_writable_path("config.ini")
HISTORY_FILE = get_writable_path("history.json")
LOG_FILE = get_writable_path("giveaway_log.txt")

# Copier le fichier config.ini de ressource vers le dossier utilisateur si nécessaire
if getattr(sys, 'frozen', False) and not os.path.exists(CONFIG_FILE):
    try:
        config_template = resource_path("config.ini")
        shutil.copy2(config_template, CONFIG_FILE)
    except Exception as exc:
        print(f"Erreur lors de la copie du fichier de configuration: {exc}")

ASCII_LOGO = [
    r"""


     _/      _/       _/                    _/                       _/
    _/_/  _/_/                 _/_/_/      _/_/_/        _/_/       _/  _/       _/_/
   _/  _/  _/       _/      _/            _/    _/    _/    _/     _/_/       _/    _/
  _/      _/       _/      _/            _/    _/    _/    _/     _/  _/     _/    _/
 _/      _/       _/        _/_/_/      _/    _/      _/_/       _/    _/     _/_/""",
    r"""
 ::::    ::::  :::::::::::  ::::::::  :::    :::  ::::::::  :::    :::  ::::::::
 +:+:+: :+:+:+     :+:     :+:    :+: :+:    :+: :+:    :+: :+:   :+:  :+:    :+:
 +:+ +:+:+ +:+     +:+     +:+        +:+    +:+ +:+    +:+ +:+  +:+   +:+    +:+
 +#+  +:+  +#+     +#+     +#+        +#++:++#++ +#+    +:+ +#++:++    +#+    +:+
 +#+       +#+     +#+     +#+        +#+    +#+ +#+    +#+ +#+  +#+   +#+    +#+
 #+#       #+#     #+#     #+#    #+# #+#    #+# #+#    #+# #+#   #+#  #+#    #+#
 ###       ### ###########  ########  ###    ###  ########  ###    ###  ######## """,
    r"""
 .::       .::                            .::
 .: .::   .::: .:       .::               .::
 .:: .:: . .::      .:::.::        .::    .::  .::   .::
 .::  .::  .::.:: .::   .: .:    .::  .:: .:: .::  .::  .::
 .::   .:  .::.::.::    .::  .::.::    .::.:.::   .::    .::
 .::       .::.:: .::   .:   .:: .::  .:: .:: .::  .::  .::
 .::       .::.::   .:::.::  .::   .::    .::  .::   .::  """,
    r"""
 888b     d888 d8b          888               888
 8888b   d8888 Y8P          888               888
 88888b.d88888              888               888
 888Y88888P888 888  .d8888b 88888b.   .d88b.  888  888  .d88b.
 888 Y888P 888 888 d88P     888  88b d88  88b 888 .88P d88  88b
 888  Y8P  888 888 888      888  888 888  888 888888K  888  888
 888       888 888 Y88b.    888  888 Y88..88P 888  88b Y88..88P
 888       888 888   Y8888P 888  888   Y88P   888  888   Y88P  """,
r"""
 ███▄ ▄███▓ ██▓ ▄████▄   ██░ ██  ▒█████   ██ ▄█▀ ▒█████
▓██▒▀█▀ ██▒▓██▒▒██▀ ▀█  ▓██░ ██▒▒██▒  ██▒ ██▄█▒ ▒██▒  ██▒
▓██    ▓██░▒██▒▒▓█    ▄ ▒██▀▀██░▒██░  ██▒▓███▄░ ▒██░  ██▒
▒██    ▒██ ░██░▒▓▓▄ ▄██▒░▓█ ░██ ▒██   ██░▓██ █▄ ▒██   ██░
▒██▒   ░██▒░██░▒ ▓███▀ ░░▓█▒░██▓░ ████▓▒░▒██▒ █▄░ ████▓▒░
░ ▒░   ░  ░░▓  ░ ░▒ ▒  ░ ▒ ░░▒░▒░ ▒░▒░▒░ ▒ ▒▒ ▓▒░ ▒░▒░▒░
░  ░      ░ ▒ ░  ░  ▒    ▒ ░▒░ ░  ░ ▒ ▒░ ░ ░▒ ▒░  ░ ▒ ▒░
░      ░    ▒ ░░         ░  ░░ ░░ ░ ░ ▒  ░ ░░ ░ ░ ░ ░ ▒
       ░    ░  ░ ░       ░  ░  ░    ░ ░  ░  ░       ░ ░
               ░
"""]


class Config:
    """Gestion des configurations utilisateur"""
    def __init__(self):
        self.config = configparser.ConfigParser()
        self.load_config()

    def load_config(self):
        """Charge ou crée le fichier de configuration"""
        if os.path.exists(CONFIG_FILE):
            self.config.read(CONFIG_FILE)
        else:
            self.create_default_config()

    def create_default_config(self):
        """Crée un fichier de configuration par défaut"""
        self.config['DEFAULT'] = {
            'language': 'en',
            'seconds_per_url': '3.0',
            'default_file': 'valid_urls.csv',
            'browser_path': ''
        }
        self.save_config()

    def save_config(self):
        """Sauvegarde la configuration dans le fichier"""
        with open(CONFIG_FILE, 'w', encoding='utf-8') as configfile:
            self.config.write(configfile)

    def get(self, section, key, fallback=None):
        """Récupère une valeur de configuration"""
        if section != 'DEFAULT' and not self.config.has_section(section):
            return fallback
        try:
            return self.config.get(section, key, fallback=fallback)
        except configparser.NoSectionError:
            return fallback

    def set(self, section, key, value):
        """Définit une valeur de configuration"""
        if section != 'DEFAULT' and not self.config.has_section(section):
            self.config.add_section(section)
        self.config[section][key] = value
        self.save_config()


class HistoryManager:
    """Gestionnaire d'historique des participations"""
    def __init__(self):
        self.history = self.load_history()

    def load_history(self):
        """Charge l'historique ou en crée un nouveau"""
        if os.path.exists(HISTORY_FILE):
            try:
                with open(HISTORY_FILE, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except json.JSONDecodeError:
                return {"participations": []}
        else:
            return {"participations": []}

    def save_history(self):
        """Sauvegarde l'historique"""
        with open(HISTORY_FILE, 'w', encoding='utf-8') as f:
            json.dump(self.history, f, indent=4)

    def add_participation(self, url, status="success"):
        """Ajoute une participation à l'historique"""
        now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        self.history["participations"].append({
            "url": url,
            "date": now,
            "status": status
        })
        self.save_history()

    def get_last_participations(self, count=10):
        """Récupère les dernières participations"""
        participations = self.history["participations"]
        return participations[-count:] if participations else []


class GiveawayChecker:
    """Classe pour vérifier les concours Instant Gaming"""
    def __init__(self, lang="en", config=None):
        self.lang = lang
        self.config = config or Config()
        self.translations = self.load_translations()
        self.history = HistoryManager()

    def load_translations(self):
        """Charge les traductions depuis le fichier"""
        try:
            with open(TRADPATH, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception as e:
            logging.error(f"Erreur lors du chargement des traductions: {e}")
            return {"en": {"errorfile": "Translation file not found."}}

    async def check_url_async(self, url, lang, session):
        """Vérifie de manière asynchrone si un concours est valide"""
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        }
        lang_params = {'fr': 'fr', 'en': 'en', 'es': 'es', 'de': 'de', 'pt': 'pt', 'it': 'it', 'pl': 'pl'}
        lang_param = lang_params.get(lang, 'en')

        # Ajustement de l'URL pour correspondre à la langue
        if is_valid_instant_gaming_url(url):
            url = re.sub(r'/[a-z]{2}/', f'/{lang_param}/', url)
            logging.debug("URL ajustée pour la langue: %s", url)

        messages = {
            'fr': {'giveaway_ended': 'Ce Giveaway est terminé', 'win_game': 'Gagne le jeu de ton choix'},
            'en': {'giveaway_ended': 'This giveaway is over', 'win_game': 'Win the game of your choice'},
            'es': {'giveaway_ended': 'Este sorteo ya ha terminado', 'win_game': 'Gana el juego que quieras'},
            'de': {'giveaway_ended': 'Dieses Giveaway ist vorüber', 'win_game': 'Gewinne ein Spiel deiner Wahl'},
            'pt': {'giveaway_ended': 'O giveaway terminou', 'win_game': 'Ganha um jogo à tua escolha'},
            'it': {'giveaway_ended': 'Questo giveaway è finito', 'win_game': 'Vinci un gioco a tua scelta'},
            'pl': {'giveaway_ended': 'To giveaway zakończone', 'win_game': 'Wygraj wybraną przez siebie grę'}
        }

        try:
            # Ajouter un timeout pour éviter les attentes infinies
            logging.debug("Tentative de vérification de l'URL: %s", url)
            async with session.get(url, headers=headers, timeout=10) as response:
                status = response.status
                logging.debug("Statut HTTP pour %s: %s", url, status)

                if status == 404:
                    logging.debug("URL non trouvée (404): %s", url)
                    return url, 'invalid'

                text = await response.text()
                text_length = len(text)
                logging.debug("Contenu reçu pour %s, longueur: %s caractères",
                            url, text_length)

                # Vérifier si le concours est terminé
                if messages[lang]['giveaway_ended'] in text:
                    logging.debug("Concours terminé trouvé: %s", url)
                    return url, 'invalid'

                # Vérifier si c'est un concours valide
                if messages[lang]['win_game'] in text:
                    logging.debug("Concours valide trouvé: %s", url)
                    return url, 'valid'

                logging.debug("Statut inconnu pour l'URL: %s", url)
                return url, 'unknown'

        except asyncio.TimeoutError:
            logging.error("Timeout lors de la vérification de l'URL %s", url)
            return url, 'invalid'
        except Exception as exc:
            logging.error("Erreur lors de la vérification de l'URL %s: %s", url, exc)
            return url, 'invalid'

    async def update_giveaways_async(self, csv_path, lang):
        """Met à jour les concours en utilisant des requêtes asynchrones"""
        try:
            urls = []
            # Gérer différemment selon qu'on est en mode exécutable ou développement
            if getattr(sys, 'frozen', False):
                # En mode exécutable, vérifier si le fichier est dans le dossier temp
                temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                os.makedirs(temp_dir, exist_ok=True)
                temp_path = os.path.join(temp_dir, os.path.basename(csv_path))

                # Vérifier d'abord dans le dossier temporaire
                if os.path.exists(temp_path):
                    full_path = temp_path
                    logging.debug(f"Utilisation du fichier trouvé dans le dossier temp: {full_path}")
                else:
                    # Si non trouvé, chercher dans les ressources
                    resource_full_path = resource_path(csv_path)
                    logging.debug(f"Tentative d'utilisation du fichier depuis les ressources: {resource_full_path}")

                    if os.path.exists(resource_full_path):
                        try:
                            # Copier le fichier des ressources vers le dossier temp pour le rendre modifiable
                            shutil.copy2(resource_full_path, temp_path)
                            full_path = temp_path
                            logging.debug(f"Fichier copié des ressources vers le dossier temp: {full_path}")
                        except Exception as e:
                            logging.error(f"Erreur lors de la copie du fichier: {e}")
                            full_path = resource_full_path
                    else:
                        # Utiliser le chemin de ressource comme dernier recours
                        full_path = resource_full_path
            else:
                # En mode développement
                full_path = resource_path(csv_path)
                logging.debug(f"Mode développement, chemin du fichier: {full_path}")

            logging.debug(f"Lecture du fichier CSV pour vérification: {full_path}")

            # Vérifier si le fichier existe
            if not os.path.exists(full_path):
                print(f"{Fore.RED}Fichier {csv_path} non trouvé!{Style.RESET_ALL}")
                logging.error(f"Fichier {csv_path} non trouvé au chemin {full_path}")
                return []

            # Vérifier si le fichier est vide
            if os.path.getsize(full_path) == 0:
                print(f"{Fore.YELLOW}Le fichier {csv_path} est vide.{Style.RESET_ALL}")
                logging.warning(f"Fichier {csv_path} est vide")
                return []

            with open(full_path, newline='', encoding='utf-8') as csvfile:
                reader = csv.reader(csvfile)
                all_urls = [row[0] for row in reader if row]

            if not all_urls:
                print(f"{Fore.YELLOW}Aucune URL trouvée dans le fichier {csv_path}.{Style.RESET_ALL}")
                return []

            # Éliminer les doublons tout en préservant l'ordre
            seen_urls = set()
            urls = []
            for url in all_urls:
                url_lower = url.strip().lower()  # Normaliser l'URL pour la comparaison
                if url_lower not in seen_urls:
                    seen_urls.add(url_lower)
                    urls.append(url)

            print(f"{Fore.CYAN}Nombre d'URLs à vérifier: {len(urls)}{Style.RESET_ALL}")
            if len(urls) < len(all_urls):
                logging.debug(f"{len(all_urls) - len(urls)} doublons ont été éliminés.")

        except Exception as e:
            print(f"{Fore.RED}Erreur lors de la lecture du fichier CSV: {e}{Style.RESET_ALL}")
            logging.error(f"Erreur lors de la lecture du fichier CSV: {e}")
            return []

        valid_urls = []
        invalid_urls = []
        unknown_urls = []

        print(f"{Fore.CYAN}{self.translations[lang]['checking_giveaways']}{Style.RESET_ALL}")

        async with aiohttp.ClientSession() as session:
            tasks = [self.check_url_async(url, lang, session) for url in urls]

            results = []
            for f in tqdm(asyncio.as_completed(tasks), total=len(tasks), desc="Progression",
                          bar_format="{l_bar}%s{bar}%s{r_bar}" % (Fore.GREEN, Style.RESET_ALL)):
                result = await f
                results.append(result)

            for url, status in results:
                if status == 'valid':
                    valid_urls.append(url)
                elif status == 'invalid':
                    invalid_urls.append(url)
                else:
                    unknown_urls.append(url)

        print(f"\n{Fore.GREEN}Concours valides: {len(valid_urls)}{' (Fichier valid_urls.csv sauvegardé)' if valid_urls else ''}{Style.RESET_ALL}")
        print(f"{Fore.RED}Concours invalides: {len(invalid_urls)}{' (Fichier invalid_urls.csv sauvegardé)' if invalid_urls else ''}{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}Concours inconnus: {len(unknown_urls)}{' (Fichier unknown_urls.csv sauvegardé)' if unknown_urls else ''}{Style.RESET_ALL}")

        # Sauvegarder dans le dossier temporaire si en mode exécutable
        if getattr(sys, 'frozen', False):
            temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
            os.makedirs(temp_dir, exist_ok=True)

            valid_urls_path = os.path.join(temp_dir, 'valid_urls.csv')
            invalid_urls_path = os.path.join(temp_dir, 'invalid_urls.csv')
            unknown_urls_path = os.path.join(temp_dir, 'unknown_urls.csv')

            if valid_urls:
                self._save_urls_to_csv(valid_urls_path, valid_urls)
            if invalid_urls:
                self._save_urls_to_csv(invalid_urls_path, invalid_urls)
            if unknown_urls:  # Ne créer le fichier que s'il y a des URLs inconnues
                self._save_urls_to_csv(unknown_urls_path, unknown_urls)

            # Important: Afficher les chemins des fichiers sauvegardés pour le débogage
            logging.debug(f"Fichiers sauvegardés dans le dossier temporaire: {temp_dir}")
            logging.debug(f"valid_urls.csv: {valid_urls_path}")
            logging.debug(f"invalid_urls.csv: {invalid_urls_path}")
            if unknown_urls:
                logging.debug(f"unknown_urls.csv: {unknown_urls_path}")
            print(f"{Fore.GREEN}Fichiers sauvegardés dans: {temp_dir}{Style.RESET_ALL}")
        else:
            # En mode développement, utiliser le répertoire courant
            if valid_urls:
                self._save_urls_to_csv('valid_urls.csv', valid_urls)
            if invalid_urls:
                self._save_urls_to_csv('invalid_urls.csv', invalid_urls)
            if unknown_urls:  # Ne créer le fichier que s'il y a des URLs inconnues
                self._save_urls_to_csv('unknown_urls.csv', unknown_urls)

        return valid_urls

    def _save_urls_to_csv(self, filename, urls):
        """Sauvegarde une liste d'URLs dans un fichier CSV"""
        if not urls:  # Ne pas créer le fichier si la liste est vide
            return

        try:
            # Assurer que le dossier parent existe
            parent_dir = os.path.dirname(filename)
            if parent_dir and not os.path.exists(parent_dir):
                os.makedirs(parent_dir, exist_ok=True)
                logging.debug(f"Création du dossier: {parent_dir}")

            # Éliminer les doublons tout en préservant l'ordre
            seen_urls = set()
            unique_urls = []
            for url in urls:
                url_lower = url.strip().lower()  # Normaliser l'URL pour la comparaison
                if url_lower not in seen_urls:
                    seen_urls.add(url_lower)
                    unique_urls.append(url)

            # Si des doublons ont été éliminés, le signaler dans le log
            if len(unique_urls) < len(urls):
                logging.debug(f"Doublons éliminés dans {filename}: {len(urls) - len(unique_urls)} URLs en double supprimées")

            # Sauvegarder le fichier avec les URLs uniques
            with open(filename, 'w', newline='', encoding='utf-8') as csvfile:
                writer = csv.writer(csvfile)
                for url in unique_urls:
                    writer.writerow([url])

            logging.debug(f"Fichier sauvegardé avec succès: {filename} ({len(urls)} URLs)")

        except Exception as e:
            print(f"{Fore.RED}Erreur lors de la sauvegarde du fichier {filename}: {e}{Style.RESET_ALL}")
            logging.error(f"Erreur lors de la sauvegarde du fichier {filename}: {e}")

    def get_urls(self, csv_path: str) -> list:
        """Récupère les URLs depuis un fichier CSV"""
        try:
            # Gérer différemment selon qu'on est en mode exécutable ou développement
            if getattr(sys, 'frozen', False):
                # En mode exécutable, vérifier si le fichier est dans le dossier temp
                temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                os.makedirs(temp_dir, exist_ok=True)
                temp_path = os.path.join(temp_dir, os.path.basename(csv_path))

                # Vérifier d'abord dans le dossier temporaire
                if os.path.exists(temp_path):
                    full_path = temp_path
                    logging.debug(f"Utilisation du fichier trouvé dans le dossier temp: {full_path}")
                else:
                    # Si non trouvé, chercher dans les ressources et le copier dans le dossier temp
                    resource_full_path = resource_path(csv_path)
                    logging.debug(f"Tentative d'utilisation du fichier depuis les ressources: {resource_full_path}")

                    if os.path.exists(resource_full_path):
                        try:
                            # Copier le fichier des ressources vers le dossier temp pour le rendre modifiable
                            shutil.copy2(resource_full_path, temp_path)
                            full_path = temp_path
                            logging.debug(f"Fichier copié des ressources vers le dossier temp: {full_path}")
                        except Exception as e:
                            logging.error(f"Erreur lors de la copie du fichier: {e}")
                            full_path = resource_full_path
                    else:
                        # Utiliser le chemin de ressource comme dernier recours
                        full_path = resource_full_path
            else:
                # En mode développement
                full_path = resource_path(csv_path)
                logging.debug(f"Mode développement, chemin du fichier: {full_path}")

            if not os.path.exists(full_path):
                raise FileNotFoundError(f"Fichier non trouvé: {full_path}")

            logging.debug(f"Lecture du fichier CSV: {full_path}")
            urls = []
            with open(full_path, "r", encoding="utf-8") as f:
                reader = csv.reader(f)
                for row in reader:
                    if row:  # Vérifier que la ligne n'est pas vide
                        urls.append(row[0])
            logging.debug(f"{len(urls)} URLs trouvées dans le fichier")
            return urls
        except FileNotFoundError as e:
            print(f"{Fore.RED}{self.translations[self.lang]['errorfile']}{Style.RESET_ALL}")
            logging.error(f"Fichier non trouvé: {csv_path}, erreur: {str(e)}")
            return None
        except Exception as e:
            error_message = f"Erreur lors de la lecture du fichier {csv_path}: {str(e)}"
            print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
            logging.error(error_message)
            logging.exception("Détails de l'erreur:")
            return None

    def open_urls(self, urls, seconds_per_url):
        """Ouvre les URLs avec un délai entre chaque"""
        lang_params = {
            'fr': 'fr', 'en': 'en', 'es': 'es', 'de': 'de', 'pt': 'pt', 'it': 'it', 'pl': 'pl'
        }
        lang_param = lang_params.get(self.lang, 'en')

        success_count = 0
        error_count = 0

        print(f"\n{Fore.CYAN}{self.translations[self.lang]['opening_urls']}{Style.RESET_ALL}")

        for i, url in enumerate(urls, 1):
            try:
                # Ajuster l'URL en fonction de la langue
                if is_valid_instant_gaming_url(url):
                    url_with_lang = re.sub(r'/[a-z]{2}/', f'/{lang_param}/', url)
                else:
                    url_with_lang = url

                # Afficher la progression
                print(f"{Fore.GREEN}[{i}/{len(urls)}] {self.translations[self.lang]['opening'].format(url_with_lang)}{Style.RESET_ALL}")

                # Ouvrir l'URL dans le navigateur
                webbrowser.open(url_with_lang)

                # Ajouter à l'historique
                self.history.add_participation(url_with_lang)

                success_count += 1

                # Attendre le temps spécifié
                for remaining in range(int(seconds_per_url), 0, -1):
                    print(f"\r{Fore.YELLOW}{self.translations[self.lang]['waiting'].format(remaining)}{Style.RESET_ALL}", end='')
                    time.sleep(1)
                print("\r" + " " * 20 + "\r", end='')  # Effacer la ligne d'attente

                # Fermer l'onglet
                keyboard.press_and_release('ctrl+w')

            except Exception as e:
                print(f"{Fore.RED}Erreur avec l'URL {url}: {str(e)}{Style.RESET_ALL}")
                self.history.add_participation(url, "error")
                error_count += 1
                logging.error(f"Erreur lors de l'ouverture de l'URL {url}: {e}")

        print(f"\n{Fore.GREEN}{self.translations[self.lang]['finished'].format(success_count, error_count)}{Style.RESET_ALL}")
        print(f"{Fore.CYAN}{self.translations[self.lang]['total_duration'].format(len(urls) * seconds_per_url)}{Style.RESET_ALL}")

        return success_count, error_count


def show_welcome_message(version):
    """Affiche un message de bienvenue avec la version et des infos système"""
    system = platform.system()
    system_version = platform.release()
    system_info = f"{system} {system_version}"
    python_version = platform.python_version()

    ascii_art = random.choice(ASCII_LOGO)

    print(f"{Fore.CYAN}{ascii_art}{Style.RESET_ALL}")
    print(f"\n{Fore.GREEN}Instant Gaming Giveaway Tool v{version}{Style.RESET_ALL}")
    print(f"{Fore.YELLOW}Système: {system_info} | Python: {python_version}{Style.RESET_ALL}")
    print(f"{Fore.YELLOW}Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}{Style.RESET_ALL}")
    print(f"{Fore.CYAN}{'=' * 60}{Style.RESET_ALL}\n")


def get_lang(translations_dict, config):
    """Demande la langue à l'utilisateur ou utilise celle en configuration"""
    try:
        default_lang = config.get('DEFAULT', 'language', fallback='en')
        # Initialize with English translations for this initial prompt
        current_lang_message = translations_dict['en']['current_language'].format(default_lang)
        change_lang_message = translations_dict['en']['change_language']

        print(f"{Fore.YELLOW}{current_lang_message}{Style.RESET_ALL}")

        change_lang = input(f"{Fore.CYAN}{change_lang_message}{Style.RESET_ALL}").lower()
        if change_lang in [translations_dict['en']['yes'].lower(), translations_dict['en']['yes'][0].lower()]:
            langchoice = input(f"\n{Fore.CYAN}Language (fr/en/es/de/pt/it/pl): {Style.RESET_ALL}")
            if langchoice not in list(translations_dict.keys()):
                langchoice = "en"
                print(f"{Fore.YELLOW}{translations_dict[langchoice]['errorlang']}{Style.RESET_ALL}")

            # Définir la langue dans la configuration
            config.set('DEFAULT', 'language', langchoice)
            return langchoice
        return default_lang
    except Exception as e:
        logging.error(f"Erreur lors de la gestion de la langue: {e}")
        print(f"{Fore.RED}Erreur lors de la gestion de la langue: {e}{Style.RESET_ALL}")
        return 'en'
def get_seconds_per_url(lang, translations_dict, config):
    """Demande le temps entre chaque URL ou utilise celui en configuration"""
    default_seconds = float(config.get('DEFAULT', 'seconds_per_url', fallback='3.0'))
    print(f"{Fore.YELLOW}{translations_dict[lang]['current_time'].format(default_seconds)}{Style.RESET_ALL}")

    try:
        change_seconds = input(f"{Fore.CYAN}{translations_dict[lang]['change_time']}{Style.RESET_ALL} ").lower()
        if change_seconds in ["yes", "y", "oui", "o", translations_dict[lang]["yes"].lower(), translations_dict[lang]["yes"][0].lower()]:
            while True:
                try:
                    seconds_per_url = float(input(f"\n{Fore.CYAN}{translations_dict[lang]['seconds_per_url']}{Style.RESET_ALL} "))
                    config.set('DEFAULT', 'seconds_per_url', str(seconds_per_url))
                    return seconds_per_url
                except ValueError:
                    print(f"{Fore.RED}{translations_dict[lang]['errortype']}{Style.RESET_ALL}\n")
    except Exception as e:
        logging.error(f"Erreur lors de la saisie du temps entre les URLs: {str(e)}")
        print(f"{Fore.YELLOW}Utilisation du temps par défaut: {default_seconds}s{Style.RESET_ALL}")
    return default_seconds
def get_ready(lang, translations_dict):
    """Demande si l'utilisateur est prêt à commencer"""
    try:
        ready = input(f"\n{Fore.GREEN}{translations_dict[lang]['reponse']}{Style.RESET_ALL} ").lower()
        if ready in ("yes", "y", "oui", "o", translations_dict[lang]["yes"].lower(), translations_dict[lang]["yes"][0].lower()):
            return True
    except Exception as e:
        logging.error(f"Erreur lors de la demande si prêt: {str(e)}")
        print(f"{Fore.YELLOW}Supposant que l'utilisateur est prêt...{Style.RESET_ALL}")
        return True
    return False


async def main():
    """Fonction principale asynchrone"""
    try:
        # Affichage des informations de version au démarrage
        print(f"\nInstant Gaming Giveaway Tool v{VERSION}")
        print(f"Date: {datetime.now().strftime('%Y-%m-%d %H:%M')} | Exécutant les vérifications...")
        
        # Configurer le logging avec le bon chemin du fichier de log
        logging.basicConfig(
            level=logging.DEBUG,  # Augmenter le niveau de logging pour voir tous les messages
            format='%(asctime)s - %(levelname)s - %(message)s',
            filename=LOG_FILE,
            filemode='a'
        )

        logging.debug("Programme démarré")

        # Effacer l'écran
        os.system('cls' if os.name == 'nt' else 'clear')

        # Charger la configuration
        config = Config()

        # Afficher le message de bienvenue
        show_welcome_message(VERSION)

        # Charger les traductions
        global TRANSLATIONS
        try:
            with open(TRADPATH, "r", encoding="utf-8") as f:
                TRANSLATIONS = json.load(f)
        except Exception as e:
            print(f"{Fore.RED}Erreur lors du chargement des traductions: {e}{Style.RESET_ALL}")
            logging.error(f"Erreur lors du chargement des traductions: {e}")
            return

        # Ajouter une information sur la non-affiliation
        print(f"\n{Fore.YELLOW}Note: This program has no affiliation with Instant Gaming, unlike other tools/contest lists.{Style.RESET_ALL}")

        # Obtenir la langue
        lang = get_lang(TRANSLATIONS, config)

        # Créer le vérificateur de concours
        checker = GiveawayChecker(lang, config)

        # Initialiser les variables par défaut
        urls = []
        file_name = "valid_urls.csv"  # Valeur par défaut

        # Demander si mise à jour des concours
        update_choice = input(f"\n{Fore.CYAN}{TRANSLATIONS[lang]['update_choice']}{Style.RESET_ALL} ")
        if update_choice.lower() in ["yes", "y", TRANSLATIONS[lang]["yes"].lower(), TRANSLATIONS[lang]["yes"][0].lower()]:
            list_uncheck_path = 'List-Uncheck.csv'
            # Extraire List-Uncheck.csv si en mode exécutable et le placer dans le dossier temporaire
            if getattr(sys, 'frozen', False):
                try:
                    # Utiliser le même dossier temporaire que pour les autres fichiers
                    temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                    os.makedirs(temp_dir, exist_ok=True)
                    list_uncheck_path = os.path.join(temp_dir, 'List-Uncheck.csv')

                    logging.debug(f"Dossier temporaire créé: {temp_dir}")
                    logging.debug(f"Chemin du fichier List-Uncheck.csv: {list_uncheck_path}")

                    # Trouver le chemin du fichier dans l'exécutable
                    src_file = resource_path('List-Uncheck.csv')
                    logging.debug(f"Chemin source pour List-Uncheck.csv: {src_file}")

                    # Vérifier si le fichier existe dans l'exécutable
                    if os.path.exists(src_file):
                        shutil.copy2(src_file, list_uncheck_path)
                        print(f"{Fore.GREEN}Fichier de liste chargé avec succès.{Style.RESET_ALL}")
                        logging.debug(f"Fichier copié avec succès de {src_file} vers {list_uncheck_path}")
                    else:
                        # Si le fichier n'est pas trouvé, créer un fichier vide
                        with open(list_uncheck_path, 'w', encoding='utf-8') as f:
                            pass  # Créer un fichier vide
                        print(f"{Fore.YELLOW}Fichier de liste non trouvé dans l'exécutable. Un nouveau fichier vide a été créé.{Style.RESET_ALL}")
                        logging.debug(f"Fichier source non trouvé, création d'un fichier vide: {list_uncheck_path}")
                except Exception as e:
                    error_message = f"Erreur lors du chargement du fichier de liste: {str(e)}"
                    print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
                    logging.error(error_message)
                    logging.exception("Détails de l'erreur:")

            # Toujours récupérer les liens depuis GitHub (sources multiples)
            print(f"\n{Fore.CYAN}Récupération des liens depuis GitHub...{Style.RESET_ALL}")
            github_url_json = "https://raw.githubusercontent.com/enzomtpYT/InstantGamingGiveawayList/refs/heads/main/json.json"
            github_url_csv = "https://raw.githubusercontent.com/michoko999/Instant-Gaming-Giveaway/refs/heads/main/List-Uncheck.csv"
            logging.debug(f"Tentative de récupération des données JSON depuis: {github_url_json}")
            logging.debug(f"Tentative de récupération des données CSV depuis: {github_url_csv}")

            use_remote = True
            use_local = False
            json_path = ""

            if use_remote or use_local:
                try:

                    # Charger les URLs du fichier existant
                    existing_urls = []
                    try:
                        with open(list_uncheck_path, "r", encoding="utf-8") as csvfile:
                            reader = csv.reader(csvfile)
                            existing_urls = [row[0] for row in reader if row]
                        logging.debug(f"URLs existantes chargées: {len(existing_urls)}")
                    except Exception as e:
                        print(f"{Fore.YELLOW}Pas de liste existante trouvée. Création d'une nouvelle liste.{Style.RESET_ALL}")
                        logging.debug(f"Erreur de lecture du fichier existant: {str(e)}")

                    all_github_urls = []

                    # 1. Récupérer et traiter les données JSON (enzomtpYT)
                    github_data = {}
                    if use_remote:
                        try:
                            response = requests.get(github_url_json, timeout=30)
                            response.raise_for_status()
                            github_data = response.json()
                            print(f"{Fore.GREEN}Données JSON récupérées depuis enzomtpYT "
                                 f"avec succès.{Style.RESET_ALL}")
                        except Exception as exc:
                            print(f"{Fore.YELLOW}Erreur lors de la récupération "
                                 f"des données JSON: {exc}{Style.RESET_ALL}")
                            logging.error("Erreur lors de la récupération des données JSON: %s",
                                        exc)
                    elif use_local:
                        with open(json_path, "r", encoding="utf-8") as json_file:
                            github_data = json.load(json_file)
                        print(f"{Fore.GREEN}Données chargées depuis le fichier local avec succès.{Style.RESET_ALL}")

                    # Traiter les concours actifs (alive) du JSON
                    if "alive" in github_data and isinstance(github_data["alive"], list):
                        alive_usernames = github_data["alive"]
                        for username in alive_usernames:
                            # Créer l'URL pour chaque nom d'utilisateur
                            all_github_urls.append(
                                f"https://www.instant-gaming.com/{lang}/giveaway/"
                                f"{username.upper()}"
                            )

                        logging.debug("Concours actifs trouvés (enzomtpYT): %s",
                                    len(alive_usernames))
                    else:
                        print(f"{Fore.YELLOW}Aucun concours actif trouvé dans les "
                             f"données JSON enzomtpYT{Style.RESET_ALL}")
                        logging.warning("Aucun concours actif trouvé dans les données JSON enzomtpYT")

                    # 2. Récupérer les données CSV (michoko999)
                    michoko_urls = []
                    if use_remote:
                        try:
                            response = requests.get(github_url_csv, timeout=30)
                            response.raise_for_status()
                            csv_content = response.text

                            # Parser le contenu CSV
                            csv_reader = csv.reader(csv_content.splitlines())
                            michoko_urls = [row[0] for row in csv_reader if row and row[0].strip()]

                            print(f"{Fore.GREEN}Données CSV récupérées depuis michoko999 "
                                 f"avec succès.{Style.RESET_ALL}")
                            logging.debug("Concours trouvés (michoko999): %s", len(michoko_urls))

                            # Ajouter les URLs de michoko999 à la liste totale
                            all_github_urls.extend(michoko_urls)

                        except Exception as exc:
                            print(f"{Fore.YELLOW}Erreur lors de la récupération "
                                 f"des données CSV michoko999: {exc}{Style.RESET_ALL}")
                            logging.error("Erreur lors de la récupération des données CSV "
                                        "michoko999: %s", exc)

                    # Optionnel: traiter également les concours terminés pour référence
                    if "dead" in github_data and isinstance(github_data["dead"], list):
                        dead_usernames = github_data["dead"]
                        logging.debug(f"Concours terminés trouvés (enzomtpYT): {len(dead_usernames)}")

                    total_github_urls = len(all_github_urls)
                    logging.debug(f"URLs totales générées: {total_github_urls}")

                    # Compter combien de nouvelles URLs seront ajoutées
                    existing_urls_set = set(existing_urls)
                    github_urls_set = set(all_github_urls)
                    new_urls = github_urls_set - existing_urls_set
                    new_urls_count = len(new_urls)

                    # S'assurer que les URL sont normalisées pour éviter les doublons
                    existing_urls_normalized = [url.strip().lower() for url in existing_urls]
                    github_urls_normalized = [url.strip().lower() for url in all_github_urls]

                    # Éliminer les doublons en utilisant des ensembles
                    existing_urls_set = set(existing_urls_normalized)
                    github_urls_set = set(github_urls_normalized)
                    new_urls = github_urls_set - existing_urls_set
                    new_urls_count = len(new_urls)

                    # Afficher les statistiques
                    print(f"{Fore.CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Style.RESET_ALL}")
                    print(f"{Fore.YELLOW}Total des concours sur GitHub: {total_github_urls}{Style.RESET_ALL}")
                    print(f"{Fore.YELLOW}Concours dans votre liste actuelle: {len(existing_urls)}{Style.RESET_ALL}")
                    print(f"{Fore.CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Style.RESET_ALL}")

                    # Fusionner les listes, éliminer les doublons tout en préservant la casse originale des URL
                    # Créer un dictionnaire de correspondance entre les URLs normalisées et les URLs originales
                    url_mapping = {}
                    for url in existing_urls + all_github_urls:
                        url_mapping[url.strip().lower()] = url

                    # Obtenir la liste finale sans doublons, en préservant la casse originale
                    all_urls = [url_mapping[url_norm] for url_norm in set(existing_urls_normalized + github_urls_normalized)]

                    # Écrire la liste fusionnée
                    with open(list_uncheck_path, "w", newline='', encoding="utf-8") as csvfile:
                        writer = csv.writer(csvfile)
                        for url in all_urls:
                            writer.writerow([url])

                    if new_urls_count > 0:
                        print(f"{Fore.GREEN}✓ {new_urls_count} nouveaux concours ajoutés à votre liste!{Style.RESET_ALL}")
                    else:
                        print(f"{Fore.YELLOW}Votre liste est déjà à jour. Aucun nouveau concours trouvé.{Style.RESET_ALL}")

                    logging.debug(f"Liste fusionnée écrite avec succès. Total: {len(all_urls)} URLs")
                except Exception as e:
                    error_message = f"Erreur lors de la récupération ou du traitement des liens: {str(e)}"
                    print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
                    logging.error(error_message)
                    logging.exception("Détails de l'erreur:")

            try:
                print(f"{Fore.CYAN}Vérification des concours en cours, veuillez patienter...{Style.RESET_ALL}")
                logging.debug(f"Démarrage de update_giveaways_async avec {list_uncheck_path}")

                # Vérifier que le fichier existe avant de continuer
                if not os.path.exists(list_uncheck_path):
                    print(f"{Fore.YELLOW}Le fichier {list_uncheck_path} n'existe pas. Utilisation d'un fichier vide.{Style.RESET_ALL}")
                    # Créer un fichier vide
                    with open(list_uncheck_path, 'w', encoding='utf-8') as f:
                        pass

                # Vérifier que le fichier n'est pas vide
                file_size = os.path.getsize(list_uncheck_path)
                if file_size == 0:
                    print(f"{Fore.YELLOW}Le fichier {list_uncheck_path} est vide. Récupération des liens depuis GitHub recommandée.{Style.RESET_ALL}")

                # Ajouter un délai visible pour montrer que le processus commence
                time.sleep(2)

                try:
                    valid_urls = await checker.update_giveaways_async(list_uncheck_path, lang)

                    if valid_urls and len(valid_urls) > 0:
                        print(f"{Fore.GREEN}Vérification terminée avec succès ! {len(valid_urls)} concours valides trouvés.{Style.RESET_ALL}")
                    else:
                        print(f"{Fore.RED}Aucun concours valide trouvé. Veuillez vérifier votre connexion internet ou essayer la récupération depuis GitHub.{Style.RESET_ALL}")
                except Exception as e:
                    print(f"{Fore.RED}Erreur lors de la vérification asynchrone: {e}{Style.RESET_ALL}")
                    logging.error(f"Erreur lors de la vérification asynchrone: {e}")
                    logging.exception("Détails de l'erreur:")
                    valid_urls = []

                logging.debug("update_giveaways_async terminé avec %s URLs valides",
                            len(valid_urls) if valid_urls else 0)
            except Exception as exc:
                error_message = f"Erreur lors de la mise à jour des concours: {str(exc)}"
                print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
                logging.error(error_message)
                logging.exception("Détails de l'erreur:")
                valid_urls = []

            # Utiliser des URLs valides ou un fichier
            if valid_urls and len(valid_urls) > 0:
                    try:
                        use_valid = input(
                            f"{Fore.CYAN}{TRANSLATIONS[lang]['use_valid_urls']}{Style.RESET_ALL} "
                        ).lower()
                        logging.debug("Réponse pour utiliser les URLs valides: %s", use_valid)

                        if use_valid in ["yes", "y", "oui", "o",
                                       TRANSLATIONS[lang]["yes"].lower(),
                                       TRANSLATIONS[lang]["yes"][0].lower()]:
                            urls = valid_urls
                            file_name = None  # Pas besoin de fichier, on utilise les URLs en mémoire
                            logging.debug("Utilisation des URLs valides récupérées: %s URLs",
                                        len(urls))
                        else:
                            # Si l'utilisateur ne veut pas utiliser les URLs valides,
                            # proposer un choix parmi les fichiers existants
                            if getattr(sys, 'frozen', False):
                                temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                                print(f"\n{Fore.YELLOW}Fichiers disponibles dans {temp_dir}: "
                                     f"valid_urls.csv, List-Uncheck.csv{Style.RESET_ALL}")
                                print(f"{Fore.YELLOW}Entrez uniquement le nom du fichier, "
                                     f"pas le chemin complet.{Style.RESET_ALL}")
                            else:
                                print(f"\n{Fore.YELLOW}Fichiers disponibles: "
                                     f"valid_urls.csv, List-Uncheck.csv{Style.RESET_ALL}")

                            file_name = input(
                                f"{Fore.CYAN}{TRANSLATIONS[lang]['csv_file_name']}{Style.RESET_ALL} "
                            )
                            if not file_name:
                                file_name = "valid_urls.csv"  # Valeur par défaut
                            logging.debug("Fichier à utiliser spécifié par l'utilisateur: %s",
                                        file_name)
                    except Exception as exc:
                        error_message = f"Erreur lors de la sélection des URLs: {str(exc)}"
                        print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
                        logging.error(error_message)
                        urls = valid_urls  # Utiliser les URLs valides en cas d'erreur
                        file_name = None
            else:
                # Aucune URL valide trouvée, demander un fichier
                if getattr(sys, 'frozen', False):
                    temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                    print(f"\n{Fore.YELLOW}Aucune URL valide trouvée. "
                         f"Fichiers disponibles dans {temp_dir}: "
                         f"valid_urls.csv, List-Uncheck.csv{Style.RESET_ALL}")
                    print(f"{Fore.YELLOW}Entrez uniquement le nom du fichier, "
                         f"pas le chemin complet.{Style.RESET_ALL}")
                else:
                    print(f"\n{Fore.YELLOW}Aucune URL valide trouvée. "
                         f"Fichiers disponibles: "
                         f"valid_urls.csv, List-Uncheck.csv{Style.RESET_ALL}")

                try:
                    file_name = input(
                        f"{Fore.CYAN}{TRANSLATIONS[lang]['csv_file_name']}{Style.RESET_ALL} "
                    )
                    if not file_name:
                        file_name = "valid_urls.csv"  # Valeur par défaut
                    logging.debug("Fichier à utiliser spécifié: %s", file_name)
                except Exception as exc:
                    error_message = f"Erreur lors de la demande du nom de fichier: {str(exc)}"
                    print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
                    logging.error(error_message)
                    file_name = "valid_urls.csv"  # Valeur par défaut
        else:
            # Pas de mise à jour demandée, utiliser un fichier existant
            if getattr(sys, 'frozen', False):
                temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                print(f"\n{Fore.YELLOW}Fichiers disponibles dans {temp_dir}: "
                     f"valid_urls.csv, List-Uncheck.csv{Style.RESET_ALL}")
                print(f"{Fore.YELLOW}Entrez uniquement le nom du fichier, "
                     f"pas le chemin complet.{Style.RESET_ALL}")
            else:
                print(f"\n{Fore.YELLOW}Fichiers disponibles: "
                     f"valid_urls.csv, List-Uncheck.csv{Style.RESET_ALL}")

            try:
                file_name = input(
                    f"{Fore.CYAN}{TRANSLATIONS[lang]['csv_file_name']}{Style.RESET_ALL} "
                )
                if not file_name:
                    file_name = "valid_urls.csv"  # Valeur par défaut
                logging.debug("Fichier à utiliser spécifié: %s", file_name)
            except Exception as exc:
                error_message = f"Erreur lors de la demande du nom de fichier: {str(exc)}"
                print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
                logging.error(error_message)
                file_name = "valid_urls.csv"  # Valeur par défaut

        # Charger les URLs depuis un fichier seulement si nécessaire
        if not urls and file_name:
            try:
                print(f"{Fore.CYAN}Chargement du fichier {file_name}...{Style.RESET_ALL}")
                urls = checker.get_urls(file_name)
                if urls is None or len(urls) == 0:
                    # En cas d'erreur avec le fichier spécifié, utiliser un fichier par défaut
                    print(f"{Fore.YELLOW}Fichier vide ou introuvable. "
                         f"Tentative d'utilisation du fichier par défaut...{Style.RESET_ALL}")
                    urls = checker.get_urls("valid_urls.csv")
                    if urls is None or len(urls) == 0:
                        # Si aucun fichier ne fonctionne, créer un fichier vide
                        temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                        os.makedirs(temp_dir, exist_ok=True)
                        default_file = os.path.join(temp_dir, 'valid_urls.csv')

                        try:
                            with open(default_file, 'w', newline='', encoding='utf-8') as csv_file:
                                writer = csv.writer(csv_file)
                                writer.writerow(["https://www.instant-gaming.com/fr/giveaway/"])
                            urls = [f"https://www.instant-gaming.com/{lang}/giveaway/"]
                            print(f"{Fore.GREEN}Fichier créé avec une URL par défaut.{Style.RESET_ALL}")
                        except Exception as exc:
                            print(f"{Fore.RED}Impossible de créer un fichier par défaut: "
                                 f"{str(exc)}{Style.RESET_ALL}")
                            return
                print(f"{Fore.GREEN}Fichier chargé avec succès. "
                     f"{len(urls)} URLs trouvées.{Style.RESET_ALL}")
            except Exception as exc:
                error_message = f"Erreur lors du chargement du fichier: {str(exc)}"
                print(f"{Fore.RED}{error_message}{Style.RESET_ALL}")
                logging.error(error_message)
                # Créer un fichier par défaut comme solution de secours
                try:
                    temp_dir = os.path.join(tempfile.gettempdir(), 'IG_Giveaway')
                    os.makedirs(temp_dir, exist_ok=True)
                    default_file = os.path.join(temp_dir, 'valid_urls.csv')
                    with open(default_file, 'w', newline='', encoding='utf-8') as csv_file:
                        writer = csv.writer(csv_file)
                        writer.writerow(["https://www.instant-gaming.com/fr/giveaway/"])
                    urls = [f"https://www.instant-gaming.com/{lang}/giveaway/"]
                    print(f"{Fore.GREEN}Fichier de secours créé "
                         f"avec une URL par défaut.{Style.RESET_ALL}")
                except Exception:
                    print(f"{Fore.RED}Impossible de créer un fichier de secours. "
                         f"Le programme va se terminer.{Style.RESET_ALL}")
                    return
        elif not urls:
            # Cas de secours si aucune URL n'est disponible
            urls = [f"https://www.instant-gaming.com/{lang}/giveaway/"]
            print(f"{Fore.YELLOW}Utilisation d'une URL par défaut.{Style.RESET_ALL}")

        try:
            # Obtenir le temps entre chaque URL
            seconds_between_urls = get_seconds_per_url(lang, TRANSLATIONS, config)

            # Afficher les explications
            print(f"\n{Fore.YELLOW}{TRANSLATIONS[lang]['explications']}{Style.RESET_ALL}")

            # Demander si prêt
            ready = get_ready(lang, TRANSLATIONS)
            if not ready:
                print(f"\n{Fore.RED}Programme annulé par l'utilisateur.{Style.RESET_ALL}")
                return
        except Exception as exc:
            logging.error("Erreur lors de la préparation: %s", str(exc))
            print(f"{Fore.YELLOW}Configuration par défaut utilisée.{Style.RESET_ALL}")
            seconds_between_urls = 3.0

        # Ouvrir les URLs
        success, errors = checker.open_urls(urls, seconds_between_urls)

        # Message de fin
        print(f"\n{Fore.GREEN}{TRANSLATIONS[lang]['endmessage'].format(success, len(urls) * seconds_between_urls)}{Style.RESET_ALL}")

        # Afficher un résumé
        print(f"\n{Fore.CYAN}{TRANSLATIONS[lang]['summary']}{Style.RESET_ALL}")
        print(f"- {Fore.GREEN}{TRANSLATIONS[lang]['urls_processed'].format(len(urls))}{Style.RESET_ALL}")
        print(f"- {Fore.GREEN}{TRANSLATIONS[lang]['success'].format(success)}{Style.RESET_ALL}")
        print(f"- {Fore.RED}{TRANSLATIONS[lang]['errors'].format(errors)}{Style.RESET_ALL}")

        input(f"\n{Fore.CYAN}{TRANSLATIONS[lang]['press_enter']}{Style.RESET_ALL}")

    except Exception as exc:
        print(f"{Fore.RED}Une erreur inattendue s'est produite: {str(exc)}{Style.RESET_ALL}")
        logging.error("Erreur inattendue dans la fonction principale: %s", exc)
        input(f"\n{Fore.RED}Appuyez sur Entrée pour quitter...{Style.RESET_ALL}")


if __name__ == "__main__":
    try:
        # Exécuter la fonction principale asynchrone
        if platform.system() == 'Windows':
            asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n{Fore.YELLOW}Programme interrompu par l'utilisateur.{Style.RESET_ALL}")
    except Exception as exc:
        print(f"\n{Fore.RED}Erreur critique: {str(exc)}{Style.RESET_ALL}")
        logging.critical("Erreur critique: %s", exc)
