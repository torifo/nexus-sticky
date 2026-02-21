@echo off
:: Nexus Sticky — Windows 起動スクリプト
:: WSL2 (Ubuntu) 経由でアプリを起動する
::
:: 使い方:
::   ダブルクリックで起動
::   スタートアップ登録: Win+R → shell:startup → このファイルのショートカットを配置

setlocal

set WSL_DIST=Ubuntu
set APP_BIN=nexus-sticky

:: WSL 内でのアプリパスを自動解決（ユーザー名に依存しない）
for /f "delims=" %%i in ('wsl -d %WSL_DIST% -- bash --norc --noprofile -c "which %APP_BIN% 2>/dev/null || find ~/devops -name %APP_BIN% -type f 2>/dev/null | head -1"') do set WSL_APP_PATH=%%i

if "%WSL_APP_PATH%"=="" (
    echo [ERROR] %APP_BIN% not found in WSL.
    echo Please build first: wsl -d %WSL_DIST% -- bash -c "cd ~/devops/nexus-sticky ^&^& make build"
    pause
    exit /b 1
)

:: 多重起動を防止
wsl -d %WSL_DIST% -- pgrep -x %APP_BIN% >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo Nexus Sticky is already running.
    exit /b 0
)

:: バックグラウンドで起動
wsl -d %WSL_DIST% -- bash --norc --noprofile -c "WEBKIT_DISABLE_COMPOSITING_MODE=1 RUST_LOG=warn nohup %WSL_APP_PATH% > /tmp/nexus-sticky.log 2>&1 &"
echo Nexus Sticky started.

endlocal
