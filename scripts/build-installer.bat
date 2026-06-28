@echo off
rem COD Installer Builder — PowerShell wrapper
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dpn0.ps1" %*
if %ERRORLEVEL% NEQ 0 (
    echo Build failed.
    exit /b %ERRORLEVEL%
)
