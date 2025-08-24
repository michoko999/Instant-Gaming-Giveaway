# -*- coding: utf-8 -*-
# Version: 1.2.0
# Auteur: Michoko
# Date de création 18/02/2023
# https://github.com/michoko999

import random
import os
import json
import csv
import time
import webbrowser
import keyboard
import requests
import re
import asyncio
import aiohttp
from datetime import datetime
from colorama import init, Fore, Style
from tqdm import tqdm
import platform
import configparser
import logging

# Initialisation de colorama pour les couleurs dans le terminal
init()

# Configuration du logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    filename='giveaway_log.txt',
    filemode='a'
)

# Variables globales
TRADPATH = "traduction.json"
CONFIG_FILE = "config.ini"
HISTORY_FILE = "history.json"
LOG_FILE = "giveaway_log.txt"
translations = None
VERSION = "1.2.0"

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
        with open(CONFIG_FILE, 'w') as configfile:
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
        if not self.config.has_section(section):
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
                with open(HISTORY_FILE, 'r') as f:
                    return json.load(f)
            except json.JSONDecodeError:
                return {"participations": []}
        else:
            return {"participations": []}
    
    def save_history(self):
        """Sauvegarde l'historique"""
        with open(HISTORY_FILE, 'w') as f:
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
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3'
        }
        lang_params = {'fr': 'fr', 'en': 'en', 'es': 'es', 'de': 'de', 'pt': 'pt', 'it': 'it', 'pl': 'pl'}
        lang_param = lang_params.get(lang, 'en')
        
        if 'instant-gaming.com' in url:
            url = re.sub(r'/[a-z]{2}/', f'/{lang_param}/', url)

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
            async with session.get(url, headers=headers) as response:
                if response.status == 404:
                    return url, 'invalid'
                
                text = await response.text()
                
                if messages[lang]['giveaway_ended'] in text:
                    return url, 'invalid'
                if messages[lang]['win_game'] in text:
                    return url, 'valid'
                
                return url, 'unknown'
        except Exception as e:
            logging.error(f"Erreur lors de la vérification de l'URL {url}: {e}")
            return url, 'invalid'
    
    async def update_giveaways_async(self, csv_path, lang):
        """Met à jour les concours en utilisant des requêtes asynchrones"""
        try:
            with open(csv_path, newline='', encoding='utf-8') as csvfile:
                reader = csv.reader(csvfile)
                urls = [row[0] for row in reader if row]
        except Exception as e:
            print(f"{Fore.RED}Erreur lors de la lecture du fichier CSV: {e}{Style.RESET_ALL}")
            logging.error(f"Erreur lors de la lecture du fichier CSV: {e}")
            return
        
        valid_urls = []
        invalid_urls = []
        unknown_urls = []
        
        print(f"{Fore.CYAN}Vérification des concours en cours...{Style.RESET_ALL}")
        
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
        
        print(f"\n{Fore.GREEN}Concours valides: {len(valid_urls)}{Style.RESET_ALL}")
        print(f"{Fore.RED}Concours invalides: {len(invalid_urls)}{Style.RESET_ALL}")
        print(f"{Fore.YELLOW}Concours inconnus: {len(unknown_urls)}{Style.RESET_ALL}")
        
        self._save_urls_to_csv('valid_urls.csv', valid_urls)
        self._save_urls_to_csv('invalid_urls.csv', invalid_urls)
        self._save_urls_to_csv('unknown_urls.csv', unknown_urls)
        
        return valid_urls
    
    def _save_urls_to_csv(self, filename, urls):
        """Sauvegarde une liste d'URLs dans un fichier CSV"""
        try:
            with open(filename, 'w', newline='', encoding='utf-8') as csvfile:
                writer = csv.writer(csvfile)
                for url in urls:
                    writer.writerow([url])
        except Exception as e:
            print(f"{Fore.RED}Erreur lors de la sauvegarde du fichier {filename}: {e}{Style.RESET_ALL}")
            logging.error(f"Erreur lors de la sauvegarde du fichier {filename}: {e}")
    
    def get_urls(self, csv_path: str) -> list:
        """Récupère les URLs depuis un fichier CSV"""
        try:
            abs_path = os.path.abspath(__file__)
            full_path = os.path.join(os.path.dirname(abs_path), csv_path)
            urls = []
            with open(full_path, "r", encoding="utf-8") as f:
                reader = csv.reader(f)
                for row in reader:
                    if row:  # Vérifier que la ligne n'est pas vide
                        urls.append(row[0])
            return urls
        except FileNotFoundError:
            print(f"{Fore.RED}{self.translations[self.lang]['errorfile']}{Style.RESET_ALL}")
            logging.error(f"Fichier non trouvé: {csv_path}")
            return None
    
    def open_urls(self, urls, seconds_per_url):
        """Ouvre les URLs avec un délai entre chaque"""
        lang_params = {
            'fr': 'fr', 'en': 'en', 'es': 'es', 'de': 'de', 'pt': 'pt', 'it': 'it', 'pl': 'pl'
        }
        lang_param = lang_params.get(self.lang, 'en')
        
        success_count = 0
        error_count = 0
        
        print(f"\n{Fore.CYAN}Début d'ouverture des URLs...{Style.RESET_ALL}")
        
        for i, url in enumerate(urls, 1):
            try:
                # Ajuster l'URL en fonction de la langue
                if 'instant-gaming.com' in url:
                    url_with_lang = re.sub(r'/[a-z]{2}/', f'/{lang_param}/', url)
                else:
                    url_with_lang = url
                
                # Afficher la progression
                print(f"{Fore.GREEN}[{i}/{len(urls)}] Ouverture de: {url_with_lang}{Style.RESET_ALL}")
                
                # Ouvrir l'URL dans le navigateur
                webbrowser.open(url_with_lang)
                
                # Ajouter à l'historique
                self.history.add_participation(url_with_lang)
                
                success_count += 1
                
                # Attendre le temps spécifié
                for remaining in range(int(seconds_per_url), 0, -1):
                    print(f"\r{Fore.YELLOW}Attente: {remaining}s{Style.RESET_ALL}", end='')
                    time.sleep(1)
                print("\r" + " " * 20 + "\r", end='')  # Effacer la ligne d'attente
                
                # Fermer l'onglet
                keyboard.press_and_release('ctrl+w')
                
            except Exception as e:
                print(f"{Fore.RED}Erreur avec l'URL {url}: {str(e)}{Style.RESET_ALL}")
                self.history.add_participation(url, "error")
                error_count += 1
                logging.error(f"Erreur lors de l'ouverture de l'URL {url}: {e}")
        
        print(f"\n{Fore.GREEN}Terminé! {success_count} URLs ouvertes avec succès, {error_count} erreurs.{Style.RESET_ALL}")
        print(f"{Fore.CYAN}Durée totale: environ {len(urls) * seconds_per_url} secondes{Style.RESET_ALL}")
        
        return success_count, error_count


def show_welcome_message(version):
    """Affiche un message de bienvenue avec la version et des infos système"""
    system_info = platform.system() + " " + platform.release()
    python_version = platform.python_version()
    
    ascii_art = random.choice(ASCII_LOGO)
    
    print(f"{Fore.CYAN}{ascii_art}{Style.RESET_ALL}")
    print(f"\n{Fore.GREEN}Instant Gaming Giveaway Tool v{version}{Style.RESET_ALL}")
    print(f"{Fore.YELLOW}Système: {system_info} | Python: {python_version}{Style.RESET_ALL}")
    print(f"{Fore.YELLOW}Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}{Style.RESET_ALL}")
    print(f"{Fore.CYAN}{'=' * 60}{Style.RESET_ALL}\n")


def get_lang(translations, config):
    """Demande la langue à l'utilisateur ou utilise celle en configuration"""
    try:
        default_lang = config.get('DEFAULT', 'language', fallback='en')
        print(f"{Fore.YELLOW}Langue actuelle: {default_lang}{Style.RESET_ALL}")
        
        change_lang = input(f"{Fore.CYAN}Voulez-vous changer la langue? (o/n): {Style.RESET_ALL}").lower()
        if change_lang in ['o', 'y', 'oui', 'yes']:
            langchoice = input(f"\n{Fore.CYAN}Language (fr/en/es/de/pt/it/pl): {Style.RESET_ALL}")
            if langchoice not in list(translations.keys()):
                langchoice = "en"
                print(f"{Fore.YELLOW}{translations[langchoice]['errorlang']}{Style.RESET_ALL}")
            
            # S'assurer que la section DEFAULT existe
            if 'DEFAULT' not in config.config.sections() and 'DEFAULT' != config.config.default_section:
                config.create_default_config()
            
            config.set('DEFAULT', 'language', langchoice)
            return langchoice
        return default_lang
    except Exception as e:
        logging.error(f"Erreur lors de la gestion de la langue: {e}")
        print(f"{Fore.RED}Erreur lors de la gestion de la langue: {e}{Style.RESET_ALL}")
        return 'en'


def get_seconds_per_url(lang, translations, config):
    """Demande le temps entre chaque URL ou utilise celui en configuration"""
    default_seconds = float(config.get('DEFAULT', 'seconds_per_url', fallback='3.0'))
    print(f"{Fore.YELLOW}Temps actuel entre les URLs: {default_seconds}s{Style.RESET_ALL}")
    
    change_seconds = input(f"{Fore.CYAN}Voulez-vous changer ce temps? (o/n): {Style.RESET_ALL}").lower()
    if change_seconds in ['o', 'y', 'oui', 'yes']:
        while True:
            try:
                seconds_per_url = float(input(f"\n{Fore.CYAN}{translations[lang]['seconds_per_url']}{Style.RESET_ALL} "))
                config.set('DEFAULT', 'seconds_per_url', str(seconds_per_url))
                return seconds_per_url
            except ValueError:
                print(f"{Fore.RED}{translations[lang]['errortype']}{Style.RESET_ALL}\n")
    return default_seconds


def get_ready(lang, translations):
    """Demande si l'utilisateur est prêt à commencer"""
    ready = input(f"\n{Fore.GREEN}{translations[lang]['reponse']}{Style.RESET_ALL}").lower()
    if ready in (translations[lang]["yes"].lower(), translations[lang]["yes"][0].lower()):
        return True
    return False


async def main():
    """Fonction principale asynchrone"""
    try:
        # Effacer l'écran
        os.system('cls' if os.name == 'nt' else 'clear')
        
        # Charger la configuration
        config = Config()
        
        # Afficher le message de bienvenue
        show_welcome_message(VERSION)
        
        # Charger les traductions
        global translations
        try:
            with open('traduction.json', "r", encoding="utf-8") as f:
                translations = json.load(f)
        except Exception as e:
            print(f"{Fore.RED}Erreur lors du chargement des traductions: {e}{Style.RESET_ALL}")
            logging.error(f"Erreur lors du chargement des traductions: {e}")
            return
        
        # Obtenir la langue
        lang = get_lang(translations, config)
        
        # Créer le vérificateur de concours
        checker = GiveawayChecker(lang, config)
        
        # Demander si mise à jour des concours
        update_choice = input(f"\n{Fore.CYAN}{translations[lang]['update_choice']}{Style.RESET_ALL} ")
        if update_choice.lower() in ['yes', 'oui', 'y', 'o', 'sí', 'si', 'ja']:
            valid_urls = await checker.update_giveaways_async('List-Uncheck.csv', lang)
            if valid_urls:
                use_valid = input(f"\n{Fore.CYAN}Voulez-vous utiliser les URLs valides trouvées? (o/n): {Style.RESET_ALL}").lower()
                if use_valid in ['o', 'y', 'oui', 'yes']:
                    urls = valid_urls
                    file_name = None
                else:
                    file_name = input(f"\n{Fore.CYAN}{translations[lang]['csv_file_name']}{Style.RESET_ALL} ")
            else:
                file_name = input(f"\n{Fore.CYAN}{translations[lang]['csv_file_name']}{Style.RESET_ALL} ")
        else:
            file_name = input(f"\n{Fore.CYAN}{translations[lang]['csv_file_name']}{Style.RESET_ALL} ")
        
        if file_name:
            urls = checker.get_urls(file_name)
            if urls is None:
                return
        
        # Obtenir le temps entre chaque URL
        seconds_between_urls = get_seconds_per_url(lang, translations, config)
        
        # Afficher les explications
        print(f"\n{Fore.YELLOW}{translations[lang]['explications']}{Style.RESET_ALL}")
        
        # Demander si prêt
        ready = get_ready(lang, translations)
        if not ready:
            print(f"\n{Fore.RED}Programme annulé par l'utilisateur.{Style.RESET_ALL}")
            return
        
        # Ouvrir les URLs
        success, errors = checker.open_urls(urls, seconds_between_urls)
        
        # Message de fin
        print(f"\n{Fore.GREEN}{translations[lang]['endmessage'].format(success, len(urls) * seconds_between_urls)}{Style.RESET_ALL}")
        
        # Afficher un résumé
        print(f"\n{Fore.CYAN}Résumé de la session:{Style.RESET_ALL}")
        print(f"- {Fore.GREEN}URLs traitées: {len(urls)}{Style.RESET_ALL}")
        print(f"- {Fore.GREEN}Succès: {success}{Style.RESET_ALL}")
        print(f"- {Fore.RED}Erreurs: {errors}{Style.RESET_ALL}")
        
        input(f"\n{Fore.CYAN}Appuyez sur Entrée pour quitter...{Style.RESET_ALL}")
        
    except Exception as e:
        print(f"{Fore.RED}Une erreur inattendue s'est produite: {str(e)}{Style.RESET_ALL}")
        logging.error(f"Erreur inattendue dans la fonction principale: {e}")
        input(f"\n{Fore.RED}Appuyez sur Entrée pour quitter...{Style.RESET_ALL}")


if __name__ == "__main__":
    try:
        # Exécuter la fonction principale asynchrone
        if platform.system() == 'Windows':
            asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n{Fore.YELLOW}Programme interrompu par l'utilisateur.{Style.RESET_ALL}")
    except Exception as e:
        print(f"\n{Fore.RED}Erreur critique: {str(e)}{Style.RESET_ALL}")
        logging.critical(f"Erreur critique: {e}")
