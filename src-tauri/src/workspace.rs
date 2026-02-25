use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::NexusError;

/// 永続化されたワークスペース全体を表す構造体
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Workspace {
    pub version: String,
    pub windows: Vec<WindowData>,
}

/// 閉じた付箋の履歴レコード（タイムスタンプ・プレビュー付き）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClosedWindowRecord {
    pub data: WindowData,
    pub closed_at: u64,  // Unix タイムスタンプ（秒）
    pub preview: String, // 先頭40文字のプレビュー
}

/// 個別の付箋ウィンドウの永続化データ
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WindowData {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub content: String,
    pub color: String,
    pub opacity: f64,
    pub pinned: bool,
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            version: "1.0".to_string(),
            windows: Vec::new(),
        }
    }
}

pub struct WorkspaceManager {
    storage_path: PathBuf,
}

impl WorkspaceManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let storage_path = app_data_dir.join("workspace.json");
        WorkspaceManager { storage_path }
    }

    /// WorkspaceをJSON文字列にシリアライズする (Requirement 8.1, 8.4)
    pub fn serialize_workspace(workspace: &Workspace) -> Result<String, NexusError> {
        serde_json::to_string_pretty(workspace)
            .map_err(|e| NexusError::SerializationFailed(e.to_string()))
    }

    /// JSON文字列をWorkspaceにデシリアライズする (Requirement 8.2, 8.3)
    pub fn deserialize_workspace(json: &str) -> Result<Workspace, NexusError> {
        serde_json::from_str(json)
            .map_err(|e| NexusError::DeserializationFailed(e.to_string()))
    }

    /// ワークスペースをローカルストレージに保存する (Requirement 7.5, 7.8)
    pub fn save_workspace(&self, workspace: &Workspace) -> Result<(), NexusError> {
        let json = Self::serialize_workspace(workspace)?;
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.storage_path, json)?;
        log::info!("Workspace saved to {:?}", self.storage_path);
        Ok(())
    }

    /// ローカルストレージからワークスペースを読み込む (Requirement 7.6)
    pub fn load_workspace(&self) -> Result<Workspace, NexusError> {
        if !self.storage_path.exists() {
            log::info!("No workspace file found, using default workspace");
            return Ok(Workspace::default());
        }
        let json = std::fs::read_to_string(&self.storage_path)?;
        let workspace = Self::deserialize_workspace(&json)?;
        log::info!(
            "Workspace loaded: {} windows from {:?}",
            workspace.windows.len(),
            self.storage_path
        );
        Ok(workspace)
    }
}

/// 閉じた付箋の履歴を JSON ファイルに保存する
pub fn save_history(path: &Path, records: &[ClosedWindowRecord]) -> Result<(), NexusError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(records)
        .map_err(|e| NexusError::SerializationFailed(e.to_string()))?;
    std::fs::write(path, json)?;
    Ok(())
}

/// JSON ファイルから閉じた付箋の履歴を読み込む（ファイルがなければ空リスト）
pub fn load_history(path: &Path) -> Vec<ClosedWindowRecord> {
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// アプリデータディレクトリを解決する
///
/// 優先順位:
///   1. 環境変数 `NEXUS_DATA_DIR`（明示的なオーバーライド）
///   2. WSL 環境の自動検出 → Windows AppData を /mnt/c/... 経由で参照
///   3. Tauri デフォルト（Windows: %APPDATA%\com.nexus.sticky、Linux: ~/.local/share/com.nexus.sticky）
///
/// WSL / Windows .exe の両方が同一ディレクトリを参照することで履歴・ワークスペースを共有できる。
pub fn resolve_app_data_dir(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;

    // 1. 環境変数オーバーライド
    if let Ok(dir) = std::env::var("NEXUS_DATA_DIR") {
        let path = PathBuf::from(&dir);
        log::info!("Data dir from NEXUS_DATA_DIR: {:?}", path);
        return path;
    }

    // 2. WSL 環境の自動検出
    if Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists() {
        log::info!("WSL detected, resolving Windows AppData path via cmd.exe");
        if let Ok(out) = std::process::Command::new("cmd.exe")
            .args(["/c", "echo %APPDATA%"])
            .output()
        {
            let appdata = String::from_utf8_lossy(&out.stdout)
                .trim_end_matches(['\r', '\n', ' '])
                .to_string();
            if !appdata.is_empty() && !appdata.contains('%') {
                if let Ok(wsl_out) = std::process::Command::new("wslpath")
                    .arg(&appdata)
                    .output()
                {
                    let wsl_path = String::from_utf8_lossy(&wsl_out.stdout)
                        .trim()
                        .to_string();
                    if !wsl_path.is_empty() {
                        let path = PathBuf::from(wsl_path).join("com.nexus.sticky");
                        log::info!("Resolved WSL data dir: {:?}", path);
                        return path;
                    }
                }
            }
        }
        log::warn!("WSL detected but could not resolve Windows AppData, using Tauri default");
    }

    // 3. Tauri デフォルト
    app.path()
        .app_data_dir()
        .expect("Failed to resolve Tauri app data dir")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_workspace() -> Workspace {
        Workspace {
            version: "1.0".to_string(),
            windows: vec![
                WindowData {
                    id: "sticky-1".to_string(),
                    x: 100,
                    y: 200,
                    width: 300,
                    height: 250,
                    content: "Hello World".to_string(),
                    color: "#FFEB3B".to_string(),
                    opacity: 0.95,
                    pinned: false,
                },
                WindowData {
                    id: "sticky-2".to_string(),
                    x: 450,
                    y: 100,
                    width: 280,
                    height: 220,
                    content: "Test note with 日本語".to_string(),
                    color: "#F48FB1".to_string(),
                    opacity: 0.8,
                    pinned: true,
                },
            ],
        }
    }

    /// Property 2: ワークスペースラウンドトリップ (Requirement 8.5)
    #[test]
    fn test_roundtrip_property() {
        let workspace = sample_workspace();
        let json = WorkspaceManager::serialize_workspace(&workspace).unwrap();
        let restored = WorkspaceManager::deserialize_workspace(&json).unwrap();
        assert_eq!(workspace, restored, "Round-trip must preserve all data");
    }

    #[test]
    fn test_invalid_json_error() {
        let result = WorkspaceManager::deserialize_workspace("invalid json {{{");
        assert!(result.is_err());
        match result {
            Err(NexusError::DeserializationFailed(msg)) => assert!(!msg.is_empty()),
            _ => panic!("Expected DeserializationFailed"),
        }
    }

    #[test]
    fn test_empty_workspace_roundtrip() {
        let workspace = Workspace::default();
        let json = WorkspaceManager::serialize_workspace(&workspace).unwrap();
        let restored = WorkspaceManager::deserialize_workspace(&json).unwrap();
        assert_eq!(workspace, restored);
    }
}
