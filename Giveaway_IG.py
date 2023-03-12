# Version: 1.02
# Auteur: Michoko, Ourson
# Date: 18/02/2023

import csv
import time
import webbrowser
import keyboard

lang = input("Language : (fr/en/es/de) :")

if lang == "fr":
    print(" ")
    print("#-------------------------#")
    print("# Auteur: Michoko, Ourson #")
    print("# Date: 18/02/2023        #")
    print("# ------------------------#")
    print(" ")
    
    nom_du_fichier = (input("Entrez le nom du fichier: "))
    print(" ")
    while True:
        try:
            seconds_per_url = int(input("Entrez le temps en secondes entre chaque ouverture d'URL : "))
            break
        except ValueError:
            print("Veuillez entrer un nombre entier pour le temps en secondes.")
            print(" ")

elif lang == "es":
    print(" ")
    print("# ------------------------#")
    print("# Autor: Michoko, Ourson  #")
    print("# Fecha: Fecha: 18/02/2023#")
    print("# ------------------------#")
    print(" ")
    nom_du_fichier = (input("Ingrese el nombre del archivo (con la extensión del archivo):"))
    print(" ")
    while True:
        try:
            seconds_per_url = int(input("Enter the time in seconds between each URL opening: "))
            break
        except ValueError:
            print("Introduzca un número entero para el tiempo en segundos.")
            print(" ")

elif lang == "en":
    print(" ")
    print("# ------------------------#")
    print("# Autor: Michoko, Ourson  #")
    print("# Date: 02/18/2023        #")
    print("# ------------------------#")
    print(" ")
    nom_du_fichier = (input("Enter the file name (with file extension):"))
    print(" ")
    while True:
        try:
            seconds_per_url = int(input("Enter the time in seconds between each URL opening: "))
            break
        except ValueError:
            print("Please enter an integer for the time in seconds.")
            print(" ")

elif lang == "de":
    print(" ")
    print("# ------------------------#")
    print("# Autor: Michoko, Ourson  #")
    print("# Datum: 18.02.2023       #")
    print("# ------------------------#")
    print(" ")
    nom_du_fichier = (input("Geben Sie den Dateinamen (mit Dateiendung) ein:"))
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
