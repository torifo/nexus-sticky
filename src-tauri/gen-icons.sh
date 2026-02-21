#!/usr/bin/env bash
# アイコンを生成するスクリプト（要: cargo tauri icon コマンド）
# 使い方: cargo tauri icon ./icons/app-icon.png

set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f "icons/app-icon.png" ]; then
    # Python で最小限の黄色 PNG を生成する
    python3 - <<'PYEOF'
import struct, zlib, os

os.makedirs("icons", exist_ok=True)

def make_png(width, height, color_rgba=(255, 235, 59, 255)):
    """単色の PNG バイナリを生成する"""
    def png_chunk(chunk_type, data):
        crc = zlib.crc32(chunk_type + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + chunk_type + data + struct.pack(">I", crc)

    sig = b'\x89PNG\r\n\x1a\n'
    ihdr = png_chunk(b'IHDR', struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))

    # RGB only (no alpha in IHDR color type 2)
    r, g, b, _ = color_rgba
    row = b'\x00' + bytes([r, g, b] * width)
    raw = row * height
    idat = png_chunk(b'IDAT', zlib.compress(raw))
    iend = png_chunk(b'IEND', b'')
    return sig + ihdr + idat + iend

for size in [32, 128]:
    data = make_png(size, size)
    with open(f"icons/{size}x{size}.png", "wb") as f:
        f.write(data)
    print(f"Created icons/{size}x{size}.png")

# 128x128@2x (256px)
data = make_png(256, 256)
with open("icons/128x128@2x.png", "wb") as f:
    f.write(data)
print("Created icons/128x128@2x.png")

# icon.png (256px - for tray)
with open("icons/icon.png", "wb") as f:
    f.write(data)
print("Created icons/icon.png")

print("Note: For production, use proper icon files or run: cargo tauri icon <your-icon.png>")
PYEOF
fi

# tauri icon コマンドを使って ICO と ICNS を生成
if command -v cargo &>/dev/null; then
    source "$HOME/.cargo/env" 2>/dev/null || true
    cargo tauri icon icons/icon.png 2>/dev/null || \
        echo "Skipping tauri icon generation (cargo tauri not available)"
fi

echo "Icon generation complete."
