# -*- coding: utf-8 -*-
# Version: 1.04
# Auteur: Michoko, Ourson
# Date de création 18/02/2023
# https://github.com/michoko999
#
# Modifié par: Mothix
# Date de modification: 13/11/2023


import random
import os
import json
import csv
import time
import webbrowser
import keyboard


# Variables globales
TRADPATH = "traduction.json"
LANGDICT = None

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


def get_trad_dict(chemin_fichier: str) -> dict:
    """
    Renvoie un dictionnaire contenant les traductions des messages du programme
    :param chemin_fichier: Chemin vers le fichier json contenant les traductions
    :return: Dictionnaire contenant les traductions
    """
    abs_path = os.path.abspath(__file__)
    full_path = os.path.join(os.path.dirname(abs_path), chemin_fichier)
    with open(full_path, "r", encoding="utf-8") as f:
        trad_dict = json.load(f)
    return trad_dict


def get_urls(lang: str, csv_path: str) -> list:
    """
    Renvoie une liste contenant les urls contenues dans le fichier csv
    :param csv_path: Chemin vers le fichier csv
    :return: Liste contenant les urls
    """
    try:
        abs_path = os.path.abspath(__file__)
        full_path = os.path.join(os.path.dirname(abs_path), csv_path)
        urls = []
        with open(full_path, "r", encoding="utf-8") as f:
            reader = csv.reader(f)
            for row in reader:
                urls.append(row[0])
        return urls
    except FileNotFoundError:
        print(LANGDICT[lang]["errorfile"])
        return None


def get_lang() -> str:
    """
    Renvoie la langue choisie par l'utilisateur
    :return: Langue choisie par l'utilisateur
    """
    langchoice = input("\n" + "Language : (fr/en/es/de) : ")
    if langchoice not in list(LANGDICT.keys()):
        langchoice = "en"
        print(LANGDICT[langchoice]["errorlang"])
    return langchoice


def get_seconds_per_url(lang: str) -> float:
    """
    Renvoie le nombre de secondes entre chaque ouverture d'URL
    :return: Nombre de secondes entre chaque ouverture d'URL
    """
    while True:
        try:
            seconds_per_url = float(input("\n" + LANGDICT[lang]["seconds_per_url"]))
            break
        except ValueError:
            print(LANGDICT[lang]["errortype"] + "\n")
    return seconds_per_url


def get_ready(lang: str) -> bool:
    """
    Renvoie True si l'utilisateur est prêt à lancer le programme
    :return: True si l'utilisateur est prêt à lancer le programme
    """
    ready = input("\n" + LANGDICT[lang]["reponse"]).lower()
    if ready in (LANGDICT[lang]["yes"].lower(), LANGDICT[lang]["yes"][0].lower()):
        return True
    return False


def open_urls(lang: str, urls: list, seconds_per_url: float):
    """
    Ouvre les urls contenues dans la liste urls
    :param urls: Liste contenant les urls
    :param seconds_per_url: Nombre de secondes entre chaque ouverture d'URL
    """
    start_time = time.time()
    for url in urls:
        webbrowser.open_new_tab(url)

        time.sleep(seconds_per_url)

        keyboard.press_and_release('ctrl+w')

    end_time = time.time()
    time_elapsed = end_time - start_time

    print(LANGDICT[lang]["endmessage"].format(len(urls), time_elapsed))


def main():
    """
    Fonction principale
    """
    os.system('cls' if os.name == 'nt' else 'clear')

    global LANGDICT
    LANGDICT = get_trad_dict(TRADPATH)

    ascii_art = random.choice(ASCII_LOGO)
    print(ascii_art + "\n")

    lang = get_lang()
    file_name = input("\n" + LANGDICT[lang]["csv_file_name"])
    liste_urls = get_urls(lang, file_name)
    if liste_urls is None:
        return
    seconds_between_urls = get_seconds_per_url(lang)
    print("\n" + LANGDICT[lang]["explications"])

    ready = get_ready(lang)
    if not ready:
        return

    open_urls(lang, liste_urls, seconds_between_urls)


if __name__ == "__main__":
    main()
