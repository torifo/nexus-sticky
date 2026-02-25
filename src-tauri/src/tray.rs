use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::nexus::NexusManager;
use crate::workspace::WorkspaceManager;

/// システムトレイを構築してアプリに登録する (Requirement 6)
pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let new_sticky =
        MenuItem::with_id(app, "new_sticky", "新規付箋作成", true, None::<&str>)?;
    let history =
        MenuItem::with_id(app, "history", "最近の付箋", true, None::<&str>)?;
    let show_all =
        MenuItem::with_id(app, "show_all", "全て表示", true, None::<&str>)?;
    let hide_all =
        MenuItem::with_id(app, "hide_all", "全て非表示", true, None::<&str>)?;
    let restore_last =
        MenuItem::with_id(app, "restore_last", "最後に閉じた付箋を復元", true, None::<&str>)?;
    let sep2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit =
        MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;

    let sep1 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&new_sticky, &history, &sep1, &show_all, &hide_all, &restore_last, &sep2, &quit])?;

    // トレイアイコンを構築 (Requirement 6.1)
    // デフォルトウィンドウアイコン、またはフォールバック黄色アイコンを使用
    // デフォルトウィンドウアイコンを使用（tauri.conf.json の bundle.icon から読み込まれる）
    let icon = app
        .default_window_icon()
        .cloned()
        .expect("No window icon found. Please ensure icons/ directory has valid PNG files.");

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Nexus Sticky")
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .build(app)?;

    log::info!("System tray initialized");
    Ok(())
}

/// システムトレイのメニューイベントを処理する (Requirement 6.4-6.7)
fn handle_menu_event(app: &AppHandle, event_id: &str) {
    let nexus = app.state::<Arc<NexusManager>>();

    match event_id {
        // 新規付箋を作成する (Requirement 6.4)
        "new_sticky" => {
            log::info!("Tray: creating new sticky window");
            if let Err(e) = nexus.create_sticky_window(None) {
                log::error!("Tray: failed to create sticky window: {}", e);
            }
        }

        // 全ウィンドウを表示する (Requirement 6.5)
        "show_all" => {
            log::info!("Tray: showing all windows");
            if let Err(e) = nexus.show_all_windows() {
                log::error!("Tray: failed to show all windows: {}", e);
            }
        }

        // 全ウィンドウを非表示にする (Requirement 6.6)
        "hide_all" => {
            log::info!("Tray: hiding all windows");
            if let Err(e) = nexus.hide_all_windows() {
                log::error!("Tray: failed to hide all windows: {}", e);
            }
        }

        // 履歴ウィンドウを開く
        "history" => {
            log::info!("Tray: opening history window");
            if let Err(e) = nexus.open_history_window() {
                log::error!("Tray: failed to open history window: {}", e);
            }
        }

        // 最後に閉じた付箋を復元する
        "restore_last" => {
            log::info!("Tray: restoring last closed sticky");
            if let Err(e) = nexus.restore_last_closed() {
                log::warn!("Tray: no recently closed sticky to restore: {}", e);
            }
        }

        // ワークスペースを保存してアプリを終了する (Requirement 6.7, 7.5)
        "quit" => {
            log::info!("Tray: saving workspace and quitting");
            let workspace = nexus.collect_workspace();
            let manager = WorkspaceManager::new(nexus.data_dir().to_path_buf());
            if let Err(e) = manager.save_workspace(&workspace) {
                log::error!("Tray: failed to save workspace on quit: {}", e);
            }
            app.exit(0);
        }

        _ => {
            log::warn!("Tray: unknown menu event '{}'", event_id);
        }
    }
}
