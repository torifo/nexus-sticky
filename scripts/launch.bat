@echo off
:: Nexus Sticky — Windows 起動スクリプト
:: PowerShell を非表示で呼び出してアプリを起動する
::
:: 使い方:
::   ダブルクリックで起動（CMD ウィンドウは表示されない）
::   スタートアップ登録: Win+R → shell:startup → このファイルのショートカットを配置

powershell -WindowStyle Hidden -ExecutionPolicy Bypass -File "%~dp0launch.ps1"
