@echo off
REM ========================================
REM PC Inventory Client - Service Uninstaller
REM ========================================
echo.
echo This script will uninstall the PC Inventory Client Windows Service.
echo.

REM 管理者権限チェック
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: This script requires Administrator privileges.
    echo Please right-click and select "Run as administrator"
    echo.
    pause
    exit /b 1
)

REM カレントディレクトリを設定
cd /d "%~dp0"

REM NSSMの存在確認
if not exist "nssm.exe" (
    echo ERROR: nssm.exe not found in the current directory.
    echo.
    pause
    exit /b 1
)

echo ========================================
echo Stopping and removing service...
echo ========================================
echo.

REM サービスを停止
echo Stopping service...
nssm stop "PCInventoryClient"
timeout /t 3 /nobreak >nul

REM サービスを削除
echo Removing service...
nssm remove "PCInventoryClient" confirm

if %errorLevel% equ 0 (
    echo.
    echo ========================================
    echo Service uninstalled successfully!
    echo ========================================
    echo.
) else (
    echo.
    echo ========================================
    echo ERROR: Failed to uninstall service.
    echo ========================================
    echo.
    echo Please check if the service exists using:
    echo   services.msc
    echo.
)

pause
