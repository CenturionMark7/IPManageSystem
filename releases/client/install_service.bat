@echo off
REM ========================================
REM PC Inventory Client - Service Installer
REM ========================================
echo.
echo This script will install the PC Inventory Client as a Windows Service.
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
    echo Please download NSSM from: https://nssm.cc/download
    echo Extract nssm.exe (64-bit version) to this directory.
    echo.
    pause
    exit /b 1
)

REM 実行ファイルの存在確認
if not exist "pc-inventory-client.exe" (
    echo ERROR: pc-inventory-client.exe not found.
    echo.
    pause
    exit /b 1
)

REM 設定ファイルの存在確認
if not exist "config.toml" (
    echo WARNING: config.toml not found.
    if exist "config.toml.template" (
        echo Please copy config.toml.template to config.toml and configure it.
    )
    echo.
    pause
    exit /b 1
)

echo ========================================
echo Installing service...
echo ========================================
echo.

REM サービスのインストール
nssm install "PCInventoryClient" "%~dp0pc-inventory-client.exe"

REM サービスの設定
nssm set "PCInventoryClient" AppDirectory "%~dp0"
nssm set "PCInventoryClient" DisplayName "PC Inventory Client"
nssm set "PCInventoryClient" Description "PC情報を収集してサーバーに送信するクライアントサービス"
nssm set "PCInventoryClient" Start SERVICE_AUTO_START
nssm set "PCInventoryClient" AppStdout "%~dp0client.log"
nssm set "PCInventoryClient" AppStderr "%~dp0client_error.log"
nssm set "PCInventoryClient" AppRotateFiles 1
nssm set "PCInventoryClient" AppRotateBytes 10485760

REM 失敗時の動作設定（10秒後に再起動）
nssm set "PCInventoryClient" AppExit Default Restart
nssm set "PCInventoryClient" AppRestartDelay 10000

echo.
echo ========================================
echo Service installed successfully!
echo ========================================
echo.
echo Service Name: PCInventoryClient
echo Display Name: PC Inventory Client
echo Startup Type: Automatic
echo.
echo The service has been installed but not started.
echo To start the service, use one of the following:
echo   - Run: start_service.bat
echo   - Run: net start PCInventoryClient
echo   - Use Windows Services Manager (services.msc)
echo.
pause
