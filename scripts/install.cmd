@echo off
rem Double-click to install TermFetch for this user (no admin rights needed).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install-windows.ps1" %*
pause
