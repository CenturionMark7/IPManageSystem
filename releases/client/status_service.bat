@echo off
REM ========================================
REM PC Inventory Client - Service Status
REM ========================================

echo Checking PC Inventory Client service status...
echo.

sc query "PCInventoryClient"

echo.
echo ========================================
echo For more details, you can use:
echo   - services.msc (Windows Services Manager)
echo   - Task Manager ^> Services tab
echo ========================================
echo.
pause
