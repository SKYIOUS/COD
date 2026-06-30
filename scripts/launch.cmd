@echo off
setlocal
set VSCODE_DEV=1
set VSCODE_SKIP_PRELAUNCH=1
start "" "%~dp0..\.build\electron\COD.exe" "%~dp0.."
endlocal