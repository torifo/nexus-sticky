// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod event;
mod nexus;
mod tray;
mod validator;
mod workspace;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State, WebviewWindow};

use error::NexusError;
use nexus::{AppState, NexusManager};
use workspace::WorkspaceManager;

// ─────────────────────────────────────────────
// Tauri Commands
// ─────────────────────────────────────────────

/// 新規付箋ウィンドウを作成する (Requirement 1.1)
#[tauri::command]
async fn create_sticky_window(
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<String, NexusError> {
    nexus.create_sticky_window(None)
}

/// ウィンドウを閉じて追跡リストから除去する (Requirement 2.3)
#[tauri::command]
async fn close_window(
    window_id: String,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<(), NexusError> {
    nexus.remove_window(&window_id)
}

/// 自ウィンドウのIDを取得する
#[tauri::command]
async fn get_window_id(window: WebviewWindow) -> Result<String, NexusError> {
    Ok(window.label().to_string())
}

/// ウィンドウの現在状態を取得する（初期化時に使用）
#[tauri::command]
async fn get_window_state(
    window_id: String,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<serde_json::Value, NexusError> {
    nexus.get_window_state_json(&window_id)
}

/// テキスト内容を更新する (Requirement 11.2, 11.3)
#[tauri::command]
async fn update_content(
    window_id: String,
    content: String,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<(), NexusError> {
    nexus.update_content(&window_id, content)
}

/// 背景色を変更する (Requirement 12.2)
#[tauri::command]
async fn update_color(
    window_id: String,
    color: String,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<(), NexusError> {
    nexus.update_color(&window_id, color)
}

/// 透過率を変更する (Requirement 5.2)
#[tauri::command]
async fn update_opacity(
    window_id: String,
    opacity: f64,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<(), NexusError> {
    nexus.update_opacity(&window_id, opacity)
}

/// ピン留め（常に最前面）を切り替える (Requirement 4.2, 4.3)
#[tauri::command]
async fn toggle_pin(
    window_id: String,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<bool, NexusError> {
    nexus.toggle_pin(&window_id)
}

/// ウィンドウ位置をバックエンドに通知する（ドラッグ後に呼び出す）
#[tauri::command]
async fn update_position(
    window_id: String,
    x: i32,
    y: i32,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<(), NexusError> {
    nexus.update_position(&window_id, x, y)
}

/// ウィンドウサイズをバックエンドに通知する
#[tauri::command]
async fn update_size(
    window_id: String,
    width: u32,
    height: u32,
    nexus: State<'_, Arc<NexusManager>>,
) -> Result<(), NexusError> {
    nexus.update_size(&window_id, width, height)
}

// ─────────────────────────────────────────────
// Application Entry Point
// ─────────────────────────────────────────────

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .setup(|app| {
            let app_handle: AppHandle = app.handle().clone();

            // AppState を初期化して管理下に置く
            let app_state = Arc::new(Mutex::new(AppState::new()));
            app.manage(app_state.clone());

            // NexusManager を初期化して管理下に置く
            let nexus = Arc::new(NexusManager::new(app_handle.clone(), app_state));
            app.manage(nexus.clone());

            // システムトレイを構築する (Requirement 6.1)
            tray::build_tray(&app_handle)?;

            // 保存済みワークスペースを読み込んで復元する (Requirement 7.6, 7.7)
            let app_dir = app.path().app_data_dir()?;
            let workspace_manager = WorkspaceManager::new(app_dir);

            match workspace_manager.load_workspace() {
                Ok(workspace) if !workspace.windows.is_empty() => {
                    log::info!("Restoring {} windows from workspace", workspace.windows.len());
                    for window_data in workspace.windows {
                        if let Err(e) = nexus.create_sticky_window(Some(window_data)) {
                            log::error!("Failed to restore window: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    // 初回起動またはワークスペースが空の場合
                    log::info!("No saved windows, creating initial sticky");
                    if let Err(e) = nexus.create_sticky_window(None) {
                        log::error!("Failed to create initial sticky: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Failed to load workspace: {}, starting fresh", e);
                    if let Err(e) = nexus.create_sticky_window(None) {
                        log::error!("Failed to create initial sticky: {}", e);
                    }
                }
            }

            Ok(())
        })
        // ウィンドウが OS から閉じられた場合も追跡リストから除去する
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let window_id = window.label().to_string();
                // sticky- プレフィックスを持つウィンドウのみ処理
                if window_id.starts_with("sticky-") {
                    let nexus = window.app_handle().state::<Arc<NexusManager>>();
                    let _ = nexus.remove_window(&window_id);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            create_sticky_window,
            close_window,
            get_window_id,
            get_window_state,
            update_content,
            update_color,
            update_opacity,
            toggle_pin,
            update_position,
            update_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
