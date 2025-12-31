@echo off
:: Generate self-signed SSL certificates for development
:: Requires OpenSSL (comes with Git for Windows)

echo Generating SSL certificates for auth.local...

:: Create ssl directory
if not exist "docker\nginx\ssl" mkdir docker\nginx\ssl

:: Generate private key and certificate
openssl req -x509 -nodes -days 365 -newkey rsa:2048 ^
    -keyout docker\nginx\ssl\auth.local.key ^
    -out docker\nginx\ssl\auth.local.crt ^
    -subj "/CN=auth.local" ^
    -addext "subjectAltName=DNS:auth.local,DNS:api.auth.local,DNS:*.auth.local"

if %errorlevel% neq 0 (
    echo ERROR: Failed to generate certificates. Make sure OpenSSL is installed.
    echo You can install it via: winget install OpenSSL.Light
    pause
    exit /b 1
)

echo.
echo Certificates generated successfully!
echo   - docker\nginx\ssl\auth.local.key
echo   - docker\nginx\ssl\auth.local.crt
echo.
echo IMPORTANT: You need to trust the certificate in Windows:
echo   1. Double-click docker\nginx\ssl\auth.local.crt
echo   2. Click "Install Certificate"
echo   3. Select "Local Machine" then "Next"
echo   4. Select "Place all certificates in the following store"
echo   5. Click "Browse" and select "Trusted Root Certification Authorities"
echo   6. Click "Next" then "Finish"
echo.
echo After trusting, restart your browser.
echo.
pause
