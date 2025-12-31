@echo off
:: Setup virtual domains for Auth Server development
:: Run as Administrator!

echo Adding virtual domains to hosts file...

:: Check if running as admin
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Please run this script as Administrator!
    echo Right-click and select "Run as administrator"
    pause
    exit /b 1
)

:: Add domains to hosts file
set HOSTS_FILE=%SystemRoot%\System32\drivers\etc\hosts

:: Check if already exists
findstr /C:"auth.local" %HOSTS_FILE% >nul 2>&1
if %errorLevel% equ 0 (
    echo Domains already configured in hosts file.
) else (
    echo.>> %HOSTS_FILE%
    echo # Auth Server Development>> %HOSTS_FILE%
    echo 127.0.0.1 auth.local>> %HOSTS_FILE%
    echo 127.0.0.1 api.auth.local>> %HOSTS_FILE%
    echo Domains added successfully!
)

echo.
echo Virtual domains configured:
echo   - http://auth.local        (Frontend)
echo   - http://api.auth.local    (Backend API)
echo.
echo Now run: docker-compose -f docker-compose.dev.yml up -d
echo.
pause
