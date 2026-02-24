use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::error::NexusError;
use crate::event::{EventBus, StickyEvent};
use crate::validator::WindowValidator;
use crate::workspace::{WindowData, Workspace};

/// 閉じた付箋の履歴レコード（タイムスタンプ・プレビュー付き）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClosedWindowRecord {
    pub data: WindowData,
    pub closed_at: u64,  // Unix タイムスタンプ（秒）
    pub preview: String, // 先頭40文字のプレビュー
}

pub const DEFAULT_COLOR: &str = "#FFEB3B"; // Requirement 12.4
pub const DEFAULT_OPACITY: f64 = 0.95;     // Requirement 5.3
const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 250;
#[allow(dead_code)]
const MAX_FAILURE_COUNT: u32 = 3; // Requirement 10.4

/// 個別付箋ウィンドウのランタイム状態 (Requirement 1.2)
#[derive(Clone, Debug)]
pub struct StickyWindowState {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub content: String,
    pub color: String,
    pub opacity: f64,
    pub pinned: bool,
    #[allow(dead_code)]
    pub failure_count: u32, // Requirement 10.4
}

impl StickyWindowState {
    pub fn new(id: String, x: i32, y: i32, width: u32, height: u32) -> Self {
        StickyWindowState {
            id,
            x,
            y,
            width,
            height,
            content: String::new(),
            color: DEFAULT_COLOR.to_string(),
            opacity: DEFAULT_OPACITY,
            pinned: false,
            failure_count: 0,
        }
    }

    /// Workspaceへの永続化用データに変換する (Requirement 7.1-7.4)
    pub fn to_window_data(&self) -> WindowData {
        WindowData {
            id: self.id.clone(),
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            content: self.content.clone(),
            color: self.color.clone(),
            opacity: self.opacity,
            pinned: self.pinned,
        }
    }
}

/// アプリケーション全体の状態 (Requirement 1.2)
pub struct AppState {
    pub windows: HashMap<String, StickyWindowState>,
    pub next_window_id: u32,
    pub recently_closed: Vec<ClosedWindowRecord>, // 直近の閉じた付箋（最大20件）
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            windows: HashMap::new(),
            next_window_id: 1,
            recently_closed: Vec::new(),
        }
    }

    /// 一意のウィンドウIDを生成する (Requirement 1.3)
    pub fn generate_window_id(&mut self) -> String {
        let id = format!("sticky-{}", self.next_window_id);
        self.next_window_id += 1;
        id
    }
}

/// Nexusの中央制御マネージャー (Requirement 1)
pub struct NexusManager {
    app_handle: AppHandle,
    state: Arc<Mutex<AppState>>,
    event_bus: Arc<EventBus>,
}

impl NexusManager {
    pub fn new(app_handle: AppHandle, state: Arc<Mutex<AppState>>) -> Self {
        let event_bus = Arc::new(EventBus::new(app_handle.clone()));
        NexusManager {
            app_handle,
            state,
            event_bus,
        }
    }

    #[allow(dead_code)]
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// スクリーンサイズを取得する（フォールバックあり）
    fn get_screen_size(&self) -> (u32, u32) {
        self.app_handle
            .primary_monitor()
            .ok()
            .flatten()
            .map(|m| {
                let s = m.size();
                (s.width, s.height)
            })
            .unwrap_or((1920, 1080))
    }

    /// 新しい付箋ウィンドウを生成する (Requirement 1.1, 1.4, 3.1)
    pub fn create_sticky_window(&self, data: Option<WindowData>) -> Result<String, NexusError> {
        let (screen_w, screen_h) = self.get_screen_size();
        let validator = WindowValidator::new(screen_w, screen_h);

        // ウィンドウIDを生成 (Requirement 1.3)
        let window_id = {
            let mut state = self.state.lock().unwrap();
            state.generate_window_id()
        };

        // 初期位置・サイズの決定
        let (raw_x, raw_y, raw_w, raw_h) = if let Some(ref d) = data {
            (d.x, d.y, d.width, d.height)
        } else {
            // 複数ウィンドウが重ならないようにオフセット
            let offset = {
                let state = self.state.lock().unwrap();
                (state.windows.len() as i32) * 30
            };
            (100 + offset, 100 + offset, DEFAULT_WIDTH, DEFAULT_HEIGHT)
        };

        // 位置・サイズを画面内に収める (Requirement 9)
        let ((x, y), (w, h)) = validator.clamp_to_screen(raw_x, raw_y, raw_w, raw_h);

        // Tauriウィンドウを構築 (Requirement 3.1)
        let url = WebviewUrl::App("sticky.html".into());
        let pinned = data.as_ref().map_or(false, |d| d.pinned);

        WebviewWindowBuilder::new(&self.app_handle, &window_id, url)
            .title("Nexus Sticky")
            .inner_size(w as f64, h as f64)
            .position(x as f64, y as f64)
            .decorations(false)          // フレームレス (Requirement 3.1)
            .transparent(cfg!(not(target_os = "linux"))) // Linux/WSLg では透過無効
            .always_on_top(pinned)       // ピン留め状態を復元 (Requirement 4)
            .min_inner_size(200.0, 150.0) // 最小サイズ (Requirement 9.3)
            .resizable(true)
            .visible(true)
            .build()
            .map_err(|e| NexusError::WindowCreationFailed(e.to_string()))?;

        // ウィンドウ状態を構築してAppStateに追加 (Requirement 1.2)
        let mut window_state = StickyWindowState::new(window_id.clone(), x, y, w, h);
        if let Some(d) = data {
            window_state.content = d.content;
            window_state.color = d.color;
            window_state.opacity = d.opacity.max(0.0).min(1.0);
            window_state.pinned = d.pinned;
        }

        {
            let mut state = self.state.lock().unwrap();
            state.windows.insert(window_id.clone(), window_state);
        }

        log::info!("Created sticky window '{}' at ({}, {}) size {}x{}", window_id, x, y, w, h);
        Ok(window_id)
    }

    /// ウィンドウを追跡リストから削除して閉じる (Requirement 2.3)
    pub fn remove_window(&self, window_id: &str) -> Result<(), NexusError> {
        {
            let mut state = self.state.lock().unwrap();
            if let Some(window_state) = state.windows.remove(window_id) {
                let closed_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let preview = window_state.content.chars().take(40).collect();
                state.recently_closed.push(ClosedWindowRecord {
                    data: window_state.to_window_data(),
                    closed_at,
                    preview,
                });
                if state.recently_closed.len() > 20 {
                    state.recently_closed.remove(0);
                }
            }
            // 既に存在しない場合もエラーにしない（二重削除対策）
        }

        if let Some(window) = self.app_handle.get_webview_window(window_id) {
            let _ = window.close();
        }

        // 削除イベントを全ウィンドウに配信 (Requirement 2.3)
        let event = StickyEvent::WindowClosed {
            window_id: window_id.to_string(),
        };
        if let Err(e) = self.event_bus.emit_to_all(&event) {
            log::warn!("Failed to broadcast window-closed for '{}': {}", window_id, e);
        }

        log::info!("Removed sticky window '{}'", window_id);
        Ok(())
    }

    /// テキスト内容を更新する (Requirement 11)
    pub fn update_content(&self, window_id: &str, content: String) -> Result<(), NexusError> {
        // 10,000文字制限 (Requirement 11.4)
        if content.chars().count() > 10_000 {
            return Err(NexusError::WindowCreationFailed(
                "Content exceeds 10,000 characters".to_string(),
            ));
        }

        {
            let mut state = self.state.lock().unwrap();
            let window = state
                .windows
                .get_mut(window_id)
                .ok_or_else(|| NexusError::WindowNotFound(window_id.to_string()))?;
            window.content = content.clone();
        }

        // 全ウィンドウに内容変更を配信 (Requirement 2.4)
        let event = StickyEvent::ContentChanged {
            window_id: window_id.to_string(),
            content,
        };
        if let Err(e) = self.event_bus.emit_to_all(&event) {
            log::error!("Content change broadcast failed: {}", e);
        }
        Ok(())
    }

    /// 背景色を更新する (Requirement 12)
    pub fn update_color(&self, window_id: &str, color: String) -> Result<(), NexusError> {
        {
            let mut state = self.state.lock().unwrap();
            let window = state
                .windows
                .get_mut(window_id)
                .ok_or_else(|| NexusError::WindowNotFound(window_id.to_string()))?;
            window.color = color.clone();
        }

        // 該当ウィンドウのみに配信 (Requirement 12.3)
        let event = StickyEvent::ColorChanged {
            window_id: window_id.to_string(),
            color,
        };
        if let Err(e) = self.event_bus.emit_to_window(window_id, &event) {
            log::error!("Color change delivery failed: {}", e);
        }
        Ok(())
    }

    /// 透過率を更新する (Requirement 5)
    pub fn update_opacity(&self, window_id: &str, opacity: f64) -> Result<(), NexusError> {
        // 0.0〜1.0の範囲にクランプ (Requirement 5.1)
        let opacity = opacity.max(0.0).min(1.0);

        {
            let mut state = self.state.lock().unwrap();
            let window = state
                .windows
                .get_mut(window_id)
                .ok_or_else(|| NexusError::WindowNotFound(window_id.to_string()))?;
            window.opacity = opacity;
        }

        // 該当ウィンドウのみに配信 (Requirement 5.4)
        let event = StickyEvent::OpacityChanged {
            window_id: window_id.to_string(),
            opacity,
        };
        if let Err(e) = self.event_bus.emit_to_window(window_id, &event) {
            log::error!("Opacity change delivery failed: {}", e);
        }
        Ok(())
    }

    /// ピン留め（常に最前面表示）を切り替える (Requirement 4)
    pub fn toggle_pin(&self, window_id: &str) -> Result<bool, NexusError> {
        let pinned = {
            let mut state = self.state.lock().unwrap();
            let window = state
                .windows
                .get_mut(window_id)
                .ok_or_else(|| NexusError::WindowNotFound(window_id.to_string()))?;
            window.pinned = !window.pinned;
            window.pinned
        };

        // OS のウィンドウ属性を更新 (Requirement 4.2, 4.3)
        if let Some(win) = self.app_handle.get_webview_window(window_id) {
            win.set_always_on_top(pinned)
                .map_err(|e| NexusError::Tauri(e.to_string()))?;
        }

        // ピン留め状態を全ウィンドウに同期 (Requirement 4.4)
        let event = StickyEvent::PinnedChanged {
            window_id: window_id.to_string(),
            pinned,
        };
        if let Err(e) = self.event_bus.emit_to_all(&event) {
            log::error!("Pin state broadcast failed: {}", e);
        }

        log::info!("Window '{}' pinned={}", window_id, pinned);
        Ok(pinned)
    }

    /// 全ウィンドウを表示する (Requirement 6.5)
    pub fn show_all_windows(&self) -> Result<(), NexusError> {
        let ids = self.get_window_ids();
        for id in ids {
            if let Some(win) = self.app_handle.get_webview_window(&id) {
                let _ = win.show();
            }
        }
        Ok(())
    }

    /// 全ウィンドウを非表示にする (Requirement 6.6)
    pub fn hide_all_windows(&self) -> Result<(), NexusError> {
        let ids = self.get_window_ids();
        for id in ids {
            if let Some(win) = self.app_handle.get_webview_window(&id) {
                let _ = win.hide();
            }
        }
        Ok(())
    }

    /// 全ウィンドウ状態をWorkspaceとして収集する (Requirement 7)
    pub fn collect_workspace(&self) -> Workspace {
        let state = self.state.lock().unwrap();
        let windows = state.windows.values().map(|w| w.to_window_data()).collect();
        Workspace {
            version: "1.0".to_string(),
            windows,
        }
    }

    /// ウィンドウの失敗カウントを増やし、閾値超過で除去する (Requirement 10.4)
    #[allow(dead_code)]
    pub fn record_window_failure(&self, window_id: &str) {
        let should_remove = {
            let mut state = self.state.lock().unwrap();
            if let Some(window) = state.windows.get_mut(window_id) {
                window.failure_count += 1;
                window.failure_count >= MAX_FAILURE_COUNT
            } else {
                false
            }
        };

        if should_remove {
            log::warn!(
                "Window '{}' exceeded failure threshold ({}), removing",
                window_id,
                MAX_FAILURE_COUNT
            );
            let _ = self.remove_window(window_id);
        }
    }

    /// 最後に閉じた付箋を復元する（トレイの「最後に閉じた付箋を復元」用）
    pub fn restore_last_closed(&self) -> Result<String, NexusError> {
        let data = {
            let mut state = self.state.lock().unwrap();
            state.recently_closed.pop().map(|r| r.data)
        };
        match data {
            Some(window_data) => {
                log::info!("Restoring last closed sticky");
                self.create_sticky_window(Some(window_data))
            }
            None => Err(NexusError::WindowNotFound(
                "閉じた付箋の履歴がありません".to_string(),
            )),
        }
    }

    /// 最近閉じた付箋の一覧を新しい順で返す（履歴ウィンドウ用）
    pub fn get_recently_closed(&self) -> Vec<ClosedWindowRecord> {
        let state = self.state.lock().unwrap();
        state.recently_closed.iter().rev().cloned().collect()
    }

    /// インデックス指定で閉じた付箋を復元する（新しい順での 0-based index）
    pub fn restore_by_index(&self, index: usize) -> Result<String, NexusError> {
        let data = {
            let mut state = self.state.lock().unwrap();
            let len = state.recently_closed.len();
            if index >= len {
                return Err(NexusError::WindowNotFound(
                    format!("インデックス {} は範囲外です（{}件）", index, len),
                ));
            }
            let actual = len - 1 - index; // 新しい順→内部の古い順インデックスに変換
            state.recently_closed.remove(actual).data
        };
        log::info!("Restoring sticky from history (index {})", index);
        self.create_sticky_window(Some(data))
    }

    /// 履歴ウィンドウを開く（既に開いていればフォーカス）
    pub fn open_history_window(&self) -> Result<(), NexusError> {
        if let Some(win) = self.app_handle.get_webview_window("history") {
            win.set_focus().map_err(|e| NexusError::Tauri(e.to_string()))?;
            return Ok(());
        }
        let url = WebviewUrl::App("history.html".into());
        WebviewWindowBuilder::new(&self.app_handle, "history", url)
            .title("最近の付箋")
            .inner_size(300.0, 420.0)
            .decorations(false)
            .transparent(cfg!(not(target_os = "linux")))
            .resizable(false)
            .build()
            .map_err(|e| NexusError::WindowCreationFailed(e.to_string()))?;
        Ok(())
    }

    /// 全ウィンドウIDのリストを返す
    pub fn get_window_ids(&self) -> Vec<String> {
        let state = self.state.lock().unwrap();
        state.windows.keys().cloned().collect()
    }

    /// ウィンドウ位置を更新する（スクリーン境界内に収める）(Requirement 9.1, 9.2)
    pub fn update_position(&self, window_id: &str, x: i32, y: i32) -> Result<(), NexusError> {
        let (screen_w, screen_h) = self.get_screen_size();
        let (w, h) = {
            let state = self.state.lock().unwrap();
            state
                .windows
                .get(window_id)
                .map(|w| (w.width, w.height))
                .ok_or_else(|| NexusError::WindowNotFound(window_id.to_string()))?
        };

        let validator = WindowValidator::new(screen_w, screen_h);
        let (cx, cy) = validator.validate_position(x, y, w, h);

        {
            let mut state = self.state.lock().unwrap();
            if let Some(window) = state.windows.get_mut(window_id) {
                window.x = cx;
                window.y = cy;
            }
        }
        Ok(())
    }

    /// ウィンドウサイズを更新する
    pub fn update_size(&self, window_id: &str, width: u32, height: u32) -> Result<(), NexusError> {
        let (screen_w, screen_h) = self.get_screen_size();
        let validator = WindowValidator::new(screen_w, screen_h);
        let (w, h) = validator.validate_size(width, height);

        {
            let mut state = self.state.lock().unwrap();
            let window = state
                .windows
                .get_mut(window_id)
                .ok_or_else(|| NexusError::WindowNotFound(window_id.to_string()))?;
            window.width = w;
            window.height = h;
        }
        Ok(())
    }

    /// ウィンドウの現在状態をJSON値として返す
    pub fn get_window_state_json(&self, window_id: &str) -> Result<serde_json::Value, NexusError> {
        let state = self.state.lock().unwrap();
        state
            .windows
            .get(window_id)
            .map(|w| {
                serde_json::json!({
                    "id": w.id,
                    "x": w.x,
                    "y": w.y,
                    "width": w.width,
                    "height": w.height,
                    "content": w.content,
                    "color": w.color,
                    "opacity": w.opacity,
                    "pinned": w.pinned,
                })
            })
            .ok_or_else(|| NexusError::WindowNotFound(window_id.to_string()))
    }
}
