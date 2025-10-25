@echo off
REM ========================================
REM PC Inventory Client - Stop Service
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

echo Stopping PC Inventory Client service...
net stop "PCInventoryClient"

if %errorLevel% equ 0 (
    echo.
    echo Service stopped successfully!
) else (
    echo.
    echo Failed to stop service.
    echo Please check the service status in services.msc
)

echo.
pause
