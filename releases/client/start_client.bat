@echo off
echo ========================================
echo PC Inventory Client - Starting...
echo ========================================
echo.
echo NOTE: This is a manual start method.
echo For production use, consider using Windows Service instead.
echo See SERVICE_SETUP.md for details.
echo.
echo ========================================
echo.

cd /d "%~dp0"
pc-inventory-client.exe

echo.
echo ========================================
echo Client stopped. Press any key to exit.
echo ========================================
pause
