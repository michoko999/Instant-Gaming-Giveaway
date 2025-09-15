# Instant Gaming Giveaway Tool

<div align="center">

📖 This README is available in:  
[<img src="https://upload.wikimedia.org/wikipedia/en/a/a4/Flag_of_the_United_States.svg" height="15"> English](README.md) | 
[<img src="https://upload.wikimedia.org/wikipedia/en/c/c3/Flag_of_France.svg" height="15"> Français](README-FR.md)

</div>

## Overview
The Instant Gaming Giveaway Tool is an automated Python script that helps you participate in Instant Gaming contests by periodically checking the list of available giveaways. Perfect for gamers who want to increase their chances of winning games!

## ✨ Features

- 🌐 **Multi-language Support**:  
  🇬🇧 English | 🇫🇷 Français | 🇪🇸 Español | 🇩🇪 Deutsch | 🇵🇹 Português | 🇮🇹 Italiano | 🇵🇱 Polski
- ⚡ **Asynchronous Contest Verification** - Faster than ever!
- 🎨 **Colorful Interface** - Visually pleasing and informative
- 📊 **Progress Bar** - Track the process advancement
- ⏰ **Customizable Check Intervals**
- 🔄 **Preference Saving** - Persistent configuration
- 📝 **Participation History** - Keep track of your contests
- ✅ **Intelligent Contest Validation**:
  - Automatically detects active contests
  - Checks contest availability
  - Ignores expired or invalid contests

## 📋 Requirements
- Python 3.8 or higher
- Web browser
- Internet connection

## 📥 Installation
1. Make sure you have Python installed on your computer
2. Run `install library.bat` to install the required dependencies
3. Or manually install dependencies with: `pip install -r requirements.txt`

## 🛠️ Compiling to Executable
If you want to create an executable (.exe) version of the application:

1. Make sure you have PyInstaller installed:
   ```
   pip install pyinstaller
   ```

2. Compile the application using:
   ```
   pyinstaller --onefile --icon=Magic.ico --name="Giveaway_IG" --add-data "config.ini;." --add-data "traduction.json;." --add-data "List-Uncheck.csv;." --version-file=file_version_info.txt Giveaway_IG.py
   ```

3. Find the compiled executable in the `dist` folder

> [!NOTE]
> To achieve a portable-executable format, the application is packaged with PyInstaller into an `EXE`. Some antivirus engines (including Windows Defender) might report the packaged executable as a trojan, because PyInstaller has been used by others to package malicious Python code in the past. These reports can be safely ignored. If you absolutely do not trust the executable, you'll have to install Python yourself and run everything from source.

## 🚀 Usage
1. Run the program using `run.bat` or by executing `python Giveaway_IG.py`
2. Select your preferred language
3. Choose a CSV file containing the giveaway URLs (default: List-Uncheck.csv)
4. Set the time interval between each URL
5. The program will open each valid giveaway URL in your browser
6. Click on the orange "Participate" button in the center of each page

## 📁 File Structure
- `Giveaway_IG.py`: Main program file
- `traduction.json`: Contains translations for multiple languages
- `List-Uncheck.csv`: Default list of URLs to check. Some contests may be sourced from [this contest list](https://github.com/enzomtpYT/InstantGamingGiveawayList).
- `valid_urls.csv`: List of valid giveaway URLs
- `invalid_urls.csv`: List of expired or invalid URLs
- `unknown_urls.csv`: List of URLs with unknown status

## ❓ Troubleshooting
- If you encounter any issues with loading translations, check the JSON format in the translation files
- Make sure your CSV files are properly formatted with one URL per line
- Ensure you have a stable internet connection

## 📄 License
This project is licensed under the MIT License. See the LICENSE file for details.

---
For additional information or support in French, click on the French badge at the top of this document or access the [French documentation (README-FR.md)](README-FR.md).
