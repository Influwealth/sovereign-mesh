@echo off
where py >nul 2>nul
if %errorlevel%==0 (
  py %*
) else (
  python.exe %*
)
