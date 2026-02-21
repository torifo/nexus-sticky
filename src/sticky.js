/**
 * Nexus Sticky - フロントエンドロジック
 * Tauri 2.0 の withGlobalTauri: true 設定のもとで動作
 */

'use strict';

// Tauri 2.0 グローバル API
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ─────────────────────────────────────────────
// 状態管理
// ─────────────────────────────────────────────
let windowId = null;
let currentColor = '#FFEB3B';
let currentOpacity = 0.95;
let isPinned = false;
let debounceTimer = null;
const DEBOUNCE_MS = 200; // Requirement 2.4
const MAX_CHARS = 10000;  // Requirement 11.4

// DOM要素
const stickyEl    = document.getElementById('sticky-window');
const contentEl   = document.getElementById('content');
const charCountEl = document.getElementById('char-count');
const pinBtn      = document.getElementById('pin-btn');
const colorBtn    = document.getElementById('color-btn');
const opacityBtn  = document.getElementById('opacity-btn');
const minimizeBtn = document.getElementById('minimize-btn');
const closeBtn    = document.getElementById('close-btn');
const colorPanel  = document.getElementById('color-panel');
const opacityPanel = document.getElementById('opacity-panel');
const opacitySlider = document.getElementById('opacity-slider');
const opacityValue  = document.getElementById('opacity-value');

// ─────────────────────────────────────────────
// 初期化 (Requirement 全体)
// ─────────────────────────────────────────────
async function init() {
  try {
    // 自ウィンドウIDを取得
    windowId = await invoke('get_window_id');

    // バックエンドから現在状態を取得して UI に適用
    const state = await invoke('get_window_state', { windowId });
    applyState(state);

    // イベントリスナーを設定
    setupUiListeners();
    await setupEventListeners();

    console.log('[NexusSticky] Initialized:', windowId);
  } catch (err) {
    console.error('[NexusSticky] Init failed:', err);
  }
}

/** バックエンドの状態を UI に適用する */
function applyState(state) {
  if (state.content)  contentEl.value = state.content;
  if (state.color)    applyColor(state.color);
  if (state.opacity != null) applyOpacity(state.opacity);
  if (state.pinned != null)  applyPin(state.pinned);
  updateCharCount();
}

// ─────────────────────────────────────────────
// UI ヘルパー
// ─────────────────────────────────────────────
function applyColor(color) {
  currentColor = color;
  stickyEl.style.backgroundColor = color;
  // 選択中のswatchをハイライト
  document.querySelectorAll('.color-swatch').forEach(sw => {
    sw.classList.toggle('selected', sw.dataset.color === color);
  });
}

function applyOpacity(opacity) {
  currentOpacity = opacity;
  document.documentElement.style.opacity = opacity;
  opacitySlider.value = Math.round(opacity * 100);
  opacityValue.textContent = `${Math.round(opacity * 100)}%`;
}

function applyPin(pinned) {
  isPinned = pinned;
  pinBtn.classList.toggle('active', pinned);
  pinBtn.title = pinned ? '常に最前面（ON）' : '常に最前面（OFF）';
}

function updateCharCount() {
  const len = contentEl.value.length;
  charCountEl.textContent = `${len} / ${MAX_CHARS}`;
  charCountEl.classList.toggle('warning', len > MAX_CHARS * 0.9);
}

function closeAllPanels() {
  colorPanel.hidden = true;
  opacityPanel.hidden = true;
}

// ─────────────────────────────────────────────
// UI イベントリスナー (Requirement 3.3, 4.2, 5.2, 11.2, 12.2)
// ─────────────────────────────────────────────
function setupUiListeners() {
  // ピン留めボタン (Requirement 4.2, 4.3)
  pinBtn.addEventListener('click', async () => {
    closeAllPanels();
    try {
      const pinned = await invoke('toggle_pin', { windowId });
      applyPin(pinned);
    } catch (err) {
      console.error('[NexusSticky] toggle_pin failed:', err);
    }
  });

  // 色選択ボタン (Requirement 12.2)
  colorBtn.addEventListener('click', () => {
    opacityPanel.hidden = true;
    colorPanel.hidden = !colorPanel.hidden;
  });

  // 透過率ボタン (Requirement 5.2)
  opacityBtn.addEventListener('click', () => {
    colorPanel.hidden = true;
    opacityPanel.hidden = !opacityPanel.hidden;
  });

  // 色スウォッチクリック (Requirement 12.2)
  document.querySelectorAll('.color-swatch').forEach(swatch => {
    swatch.addEventListener('click', async () => {
      const color = swatch.dataset.color;
      applyColor(color);
      closeAllPanels();
      try {
        await invoke('update_color', { windowId, color });
      } catch (err) {
        console.error('[NexusSticky] update_color failed:', err);
      }
    });
  });

  // 透過率スライダー (Requirement 5.2)
  opacitySlider.addEventListener('input', () => {
    const opacity = parseInt(opacitySlider.value) / 100;
    applyOpacity(opacity);
  });

  opacitySlider.addEventListener('change', async () => {
    const opacity = parseInt(opacitySlider.value) / 100;
    try {
      await invoke('update_opacity', { windowId, opacity });
    } catch (err) {
      console.error('[NexusSticky] update_opacity failed:', err);
    }
  });

  // 最小化ボタン (Requirement 3.3)
  minimizeBtn.addEventListener('click', async () => {
    closeAllPanels();
    try {
      // Tauri 2.0: getCurrentWindow()
      const { getCurrentWindow } = window.__TAURI__.window;
      await getCurrentWindow().minimize();
    } catch (err) {
      console.error('[NexusSticky] minimize failed:', err);
    }
  });

  // 閉じるボタン (Requirement 3.3)
  closeBtn.addEventListener('click', async () => {
    try {
      await invoke('close_window', { windowId });
    } catch (err) {
      console.error('[NexusSticky] close_window failed:', err);
    }
  });

  // テキスト入力（デバウンス 200ms）(Requirement 11.2, 11.3, 2.4)
  contentEl.addEventListener('input', () => {
    const len = contentEl.value.length;
    updateCharCount();

    // 10,000文字制限に達した場合の通知 (Requirement 11.5)
    if (len >= MAX_CHARS) {
      charCountEl.classList.remove('limit-reached');
      void charCountEl.offsetWidth; // reflow for restart animation
      charCountEl.classList.add('limit-reached');
    }

    // バックエンドへのデバウンス送信 (Requirement 2.4)
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      try {
        await invoke('update_content', { windowId, content: contentEl.value });
      } catch (err) {
        console.error('[NexusSticky] update_content failed:', err);
      }
    }, DEBOUNCE_MS);
  });

  // パネル外クリックで閉じる
  document.addEventListener('click', (e) => {
    if (!colorBtn.contains(e.target) && !colorPanel.contains(e.target)) {
      colorPanel.hidden = true;
    }
    if (!opacityBtn.contains(e.target) && !opacityPanel.contains(e.target)) {
      opacityPanel.hidden = true;
    }
  });

  // ウィンドウリサイズ時にサイズをバックエンドに通知
  let resizeTimer = null;
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimer);
    resizeTimer = setTimeout(async () => {
      try {
        await invoke('update_size', {
          windowId,
          width: window.innerWidth,
          height: window.innerHeight,
        });
      } catch (_) {}
    }, 500);
  });
}

// ─────────────────────────────────────────────
// Tauri イベントリスナー（クロスウィンドウ同期）
// Requirement 2.1, 2.2, 2.4
// ─────────────────────────────────────────────
async function setupEventListeners() {
  // コンテンツ変更（他ウィンドウからの同期）(Requirement 2.4)
  await listen('content-changed', (event) => {
    const { window_id, content } = event.payload.data;
    if (window_id !== windowId) {
      // 他ウィンドウからの変更を反映（カーソル位置を保持）
      const sel = contentEl.selectionStart;
      contentEl.value = content;
      try { contentEl.setSelectionRange(sel, sel); } catch (_) {}
      updateCharCount();
    }
  });

  // 色変更（Requirement 2.1）
  await listen('color-changed', (event) => {
    const { window_id, color } = event.payload.data;
    if (window_id === windowId) {
      applyColor(color);
    }
  });

  // 透過率変更（Requirement 5.4）
  await listen('opacity-changed', (event) => {
    const { window_id, opacity } = event.payload.data;
    if (window_id === windowId) {
      applyOpacity(opacity);
    }
  });

  // ピン留め変更（Requirement 4.4, 2.2）
  await listen('pinned-changed', (event) => {
    const { window_id, pinned } = event.payload.data;
    if (window_id === windowId) {
      applyPin(pinned);
    }
  });

  // ウィンドウ削除通知
  await listen('window-closed', (event) => {
    const { window_id } = event.payload.data;
    if (window_id === windowId) {
      // 自分が削除された場合（通常は close_window コマンドで処理済み）
      window.close();
    }
  });
}

// ─────────────────────────────────────────────
// エントリーポイント
// ─────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', init);
