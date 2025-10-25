@echo off
REM ========================================
REM PC Inventory Client - Restart Service
REM ========================================

REM 管理者権限チェック
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: This script requires Administrator privileges.
    echo Please right-click and select "Run as administrator"
    echo.
    pause
    exit /b 1
)

echo Restarting PC Inventory Client service...
echo.

echo Stopping service...
net stop "PCInventoryClient"
timeout /t 3 /nobreak >nul

echo Starting service...
net start "PCInventoryClient"

if %errorLevel% equ 0 (
    echo.
    echo Service restarted successfully!
) else (
    echo.
    echo Failed to restart service.
    echo Please check the service status in services.msc
)

echo.
pause
