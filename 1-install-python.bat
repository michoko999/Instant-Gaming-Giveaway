@echo off
setlocal

REM Téléchargement du programme d'installation de Python
curl https://www.python.org/ftp/python/3.9.6/python-3.9.6-amd64.exe --output python-3.9.6-amd64.exe

REM Installation de Python avec les options par défaut
python-3.9.6-amd64.exe /quiet InstallAllUsers=1 PrependPath=1 Include_test=0

REM Nettoyage du fichier d'installation
del python-3.9.6-amd64.exe

echo Installation terminée !
