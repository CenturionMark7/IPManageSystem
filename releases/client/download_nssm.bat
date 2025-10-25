@echo off
REM ========================================
REM NSSM Download Helper Script
REM ========================================
echo.
echo This script will help you download NSSM (Non-Sucking Service Manager).
echo NSSM is required for Windows Service installation.
echo.
echo ========================================
echo NSSM Download Instructions
echo ========================================
echo.
echo Please follow these steps:
echo.
echo 1. Open your web browser
echo 2. Go to: https://nssm.cc/download
echo 3. Download the latest version (e.g., nssm-2.24.zip)
echo 4. Extract the ZIP file
echo 5. Navigate to the 'win64' folder inside the extracted folder
echo 6. Copy 'nssm.exe' from the 'win64' folder
echo 7. Paste 'nssm.exe' into this directory:
echo    %~dp0
echo.
echo ========================================
echo.

REM ブラウザでダウンロードページを開く
echo Opening NSSM download page in your default browser...
start https://nssm.cc/download

echo.
echo After downloading and extracting nssm.exe to this folder,
echo run 'install_service.bat' to install the service.
echo.
pause
