@echo off
echo ========================================
echo PC Inventory Server - Starting...
echo ========================================
echo.

cd /d "%~dp0"
pc-inventory-server.exe

echo.
echo ========================================
echo Server stopped. Press any key to exit.
echo ========================================
pause
