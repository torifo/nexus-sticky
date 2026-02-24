use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::NexusError;

/// ウィンドウ間およびバックエンドで伝播するイベントの種類 (Requirement 2)
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum StickyEvent {
    ContentChanged { window_id: String, content: String },
    ColorChanged { window_id: String, color: String },
    OpacityChanged { window_id: String, opacity: f64 },
    PinnedChanged { window_id: String, pinned: bool },
    WindowClosed { window_id: String },
    PositionChanged { window_id: String, x: i32, y: i32 },
    SizeChanged { window_id: String, width: u32, height: u32 },
}

impl StickyEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            StickyEvent::ContentChanged { .. } => "content-changed",
            StickyEvent::ColorChanged { .. } => "color-changed",
            StickyEvent::OpacityChanged { .. } => "opacity-changed",
            StickyEvent::PinnedChanged { .. } => "pinned-changed",
            StickyEvent::WindowClosed { .. } => "window-closed",
            StickyEvent::PositionChanged { .. } => "position-changed",
            StickyEvent::SizeChanged { .. } => "size-changed",
        }
    }

    #[allow(dead_code)]
    pub fn window_id(&self) -> &str {
        match self {
            StickyEvent::ContentChanged { window_id, .. } => window_id,
            StickyEvent::ColorChanged { window_id, .. } => window_id,
            StickyEvent::OpacityChanged { window_id, .. } => window_id,
            StickyEvent::PinnedChanged { window_id, .. } => window_id,
            StickyEvent::WindowClosed { window_id } => window_id,
            StickyEvent::PositionChanged { window_id, .. } => window_id,
            StickyEvent::SizeChanged { window_id, .. } => window_id,
        }
    }
}

/// イベント配信システム。エラー時も他のウィンドウへの配信を継続する (Requirement 10)
pub struct EventBus {
    app_handle: AppHandle,
}

impl EventBus {
    pub fn new(app_handle: AppHandle) -> Self {
        EventBus { app_handle }
    }

    /// 全ウィンドウにイベントを配信する (Requirement 2.1, 2.2)
    /// 失敗してもエラーをログ記録して処理を継続する (Requirement 10.1, 10.2)
    pub fn emit_to_all(&self, event: &StickyEvent) -> Result<(), NexusError> {
        let event_name = event.event_name();
        match self.app_handle.emit(event_name, event) {
            Ok(_) => {
                log::debug!("Emitted '{}' to all windows", event_name);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to emit '{}' to all: {}", event_name, e);
                Err(NexusError::EventDeliveryFailed(e.to_string()))
            }
        }
    }

    /// 特定ウィンドウにイベントを配信する (Requirement 5.4, 12.3)
    pub fn emit_to_window(&self, window_id: &str, event: &StickyEvent) -> Result<(), NexusError> {
        let event_name = event.event_name();
        if let Some(window) = self.app_handle.get_webview_window(window_id) {
            match window.emit(event_name, event) {
                Ok(_) => {
                    log::debug!("Emitted '{}' to window '{}'", event_name, window_id);
                    Ok(())
                }
                Err(e) => {
                    log::error!(
                        "Failed to emit '{}' to window '{}': {}",
                        event_name,
                        window_id,
                        e
                    );
                    Err(NexusError::EventDeliveryFailed(e.to_string()))
                }
            }
        } else {
            log::warn!("emit_to_window: window '{}' not found", window_id);
            Err(NexusError::WindowNotFound(window_id.to_string()))
        }
    }

    /// ソースウィンドウ以外の全ウィンドウにイベントを配信する
    /// 失敗したウィンドウをリストで返す (Requirement 10.2)
    #[allow(dead_code)]
    pub fn emit_to_others(
        &self,
        source_window_id: &str,
        event: &StickyEvent,
        all_window_ids: &[String],
    ) -> Vec<(String, NexusError)> {
        let mut failures = Vec::new();
        for window_id in all_window_ids {
            if window_id != source_window_id {
                if let Err(e) = self.emit_to_window(window_id, event) {
                    log::error!("Event delivery to '{}' failed: {}", window_id, e);
                    failures.push((window_id.clone(), e));
                    // 配信失敗してもループを継続 (Requirement 10.2)
                }
            }
        }
        failures
    }
}
