# -*- coding: utf-8 -*-
# Version: 1.03
# Auteur: Michoko, Ourson
# Date de création 18/02/2023
# https://github.com/michoko999

import random
import os 
import csv
import time
import webbrowser
import keyboard

def clear():
    os.system('cls' if os.name == 'nt' else 'clear')

clear()

ascii_arts = [
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
"""
]                                                               

# Sélectionne un ASCII art aléatoire
ascii_art = random.choice(ascii_arts)

# Affiche l'ASCII art
print(ascii_art)

print(" ")
lang = input("Language : (fr/en/es/de) :")      
print(" ")                                              

if lang == "fr":
    nom_du_fichier = (input("Entrez la liste (avec l'extension du fichier en .csv): "))
    print(" ")
    while True:
        try:
            seconds_per_url = int(input("Entrez le temps en secondes entre chaque ouverture d'URL : "))
            break
        except ValueError:
            print("Veuillez entrer un nombre entier pour le temps en secondes.")
            print(" ")

elif lang == "es":
    nom_du_fichier = (input("Ingrese a la lista (con la extensión de archivo en .csv):"))
    print(" ")
    while True:
        try:
            seconds_per_url = int(input("Enter the time in seconds between each URL opening: "))
            break
        except ValueError:
            print("Introduzca un número entero para el tiempo en segundos.")
            print(" ")

elif lang == "en":
    nom_du_fichier = (input("Enter the list (with the file extension in .csv):"))
    print(" ")
    while True:
        try:
            seconds_per_url = int(input("Enter the time in seconds between each URL opening: "))
            break
        except ValueError:
            print("Please enter an integer for the time in seconds.")
            print(" ")

elif lang == "de":
    nom_du_fichier = (input("Geben Sie die Liste ein (mit der Dateierweiterung in .csv):"))
    print(" ")
    while True:
        try:
            seconds_per_url = int(input("Geben Sie die Zeit in Sekunden zwischen jedem URL-Öffnen ein:"))
            break
        except ValueError:
            print("Bitte geben Sie eine Ganzzahl für die Zeit in Sekunden ein.")
            print(" ")

else:
    print(" ")
    print("Language is not supported.")
    print(" ")

def demander_preparation(lang):
    if lang == "fr":
        while True:
            print(" ")
            reponse = input("Êtes-vous prêt à commencer ? (Oui/Non) ")
            reponse = reponse.lower()
            if reponse == "oui":
                return True
            elif reponse == "non":
                explications(lang)
                continue
            else:
                print("Réponse invalide. Veuillez répondre par 'Oui' ou 'Non'.")

    elif lang == "es":
        while True:
            print(" ")
            reponse = input("¿Estás listo para comenzar? (Sí/No) ")
            reponse = reponse.lower()
            if reponse == "sí":
                return True
            elif reponse == "no":
                explications(lang)
                continue
            else:
                print("Respuesta inválida. Por favor, responde con 'Sí' o 'No'.")

    elif lang == "en":
        while True:
            print(" ")
            reponse = input("Are you ready to start? (Yes/No) ")
            reponse = reponse.lower()
            if reponse == "yes":
                return True
            elif reponse == "no":
                explications(lang)
                continue
            else:
                print("Invalid response. Please answer with 'Yes' or 'No'.")

    elif lang == "de":
        while True:
            print(" ")
            reponse = input("Bist du bereit zu starten? (Ja/Nein) ")
            reponse = reponse.lower()
            if reponse == "ja":
                return True
            elif reponse == "nein":
                explications(lang)
                continue
            else:
                print("Ungültige Antwort. Bitte antworte mit 'Ja' oder 'Nein'.")

    else:
        
        print("Language is not supported.")
        return False

def explications(lang):
    if lang == "fr":
        print(" ")
        print("Voici quelques explications sur la suite du processus :")
        print("Le programme va commencer a ouvrir les pages une par une avec les concours, il faudra juste cliquer sur le bouton orange a centre de l'écran avec écrit 'Participer' ")
        # Ajoutez ici les explications supplémentaires en français
    elif lang == "es":
        print(" ")
        print("Aquí hay algunas explicaciones sobre el resto del proceso:")
        print("El programa comenzará a abrir las páginas una por una con los concursos, solo debes hacer clic en el botón naranja en el centro de la pantalla con la escritura 'Participar'")
        # Ajoutez ici les explications supplémentaires en espagnol
    elif lang == "en":
        print(" ")
        print("Here are some explanations on the rest of the process:")
        print("The program will start to open the pages one by one with the competitions, you just have to click on the orange button in the center of the screen with the writing 'Participate'")
        # Ajoutez ici les explications supplémentaires en anglais
    elif lang == "de":
        print(" ")
        print("Hier einige Erläuterungen zum weiteren Ablauf:")
        print("Das Programm beginnt, die Seiten nacheinander mit den Wettbewerben zu öffnen. Sie müssen lediglich auf die orangefarbene Schaltfläche in der Mitte des Bildschirms mit der Aufschrift „Teilnehmen“ klicken.")
        # Ajoutez ici les explications supplémentaires en allemand

# Appel de la fonction pour demander la préparation
if demander_preparation(lang):
    # L'utilisateur est prêt, continuez avec le reste du programme
    # Ajoutez ici le code que vous souhaitez exécuter lorsque l'utilisateur est prêt
    print(" ")
    print("Début du programme...")
time.sleep(3)
# Ouvrir le fichier CSV et stocker les URLs dans une liste
filename = nom_du_fichier
urls = []
try:
    with open(nom_du_fichier, 'r') as csv_file:
        csv_reader = csv.reader(csv_file)
        for row in csv_reader:
            urls.append(row[0])
except FileNotFoundError:
    print(f"The file {filename} was not found.")
    print(f"Please check the file name, that the file is in the same directory, that you have put the file extension correctly and try again")
    exit()

# Demander à l'utilisateur le temps entre chaque ouverture d'URL
time.sleep(1)

# Ouvrir chaque URL dans la liste
start_time = time.time()
num_urls = len(urls)
for i in range(num_urls):
    # Ouvrir l'URL
    webbrowser.open_new_tab(urls[i])
    
    # Attendre le nombre de secondes spécifié par l'utilisateur
    time.sleep(seconds_per_url)
    
    # Fermer l'onglet actif
    keyboard.press_and_release('ctrl+w')

end_time = time.time()

# Afficher le temps approximatif nécessaire et le nombre d'URLs ouvertes
time_elapsed = end_time - start_time

if lang == "fr":
    message = f"{num_urls} URLs ont été ouvertes en environ {round(time_elapsed,2)} secondes."
elif lang == "es":
    message = f"Se abrieron {num_urls} URL en aproximadamente {round(time_elapsed,2)} segundos."
elif lang == "en":
    message = f"{num_urls} URLs were opened in approximately {round(time_elapsed,2)} seconds."
elif lang == "de":
    message = f"{num_urls} URLs wurden in etwa {round(time_elapsed,2)} Sekunden geöffnet."
else:
    message = "Language is not supported."

print(message)
