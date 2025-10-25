@echo off
REM ========================================
REM PC Inventory Client - Start Service
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

echo Starting PC Inventory Client service...
net start "PCInventoryClient"

if %errorLevel% equ 0 (
    echo.
    echo Service started successfully!
) else (
    echo.
    echo Failed to start service.
    echo Please check the service status in services.msc
)

echo.
pause
