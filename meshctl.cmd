@echo off
where py >nul 2>nul
if %errorlevel%==0 (
  py meshctl_mcp.py %*
) else (
  python.exe meshctl_mcp.py %*
)
