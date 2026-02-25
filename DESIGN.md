# Clipboard Hack - 設計書

## 概要

クリップボードの内容を履歴として保持し、選択したアイテムを様々な形式で解釈・表示する
デスクトップアプリケーション。

---

## フレームワーク選定

### 言語: Rust

- クロスプラットフォーム（Linux / macOS）で動作
- 低レベルなシステム統合（グローバルホットキー）が容易
- バイナリ配布が簡単（ランタイム不要）

### GUI: egui / eframe

- Immediate Mode GUI で実装がシンプル
- Linux (X11/Wayland) / macOS 両対応
- ウィンドウ管理、イベントループを eframe が提供

### クリップボード: arboard

- クロスプラットフォームなクリップボードアクセスの標準的な Rust クレート
- Linux (X11/Wayland) / macOS 対応

### グローバルホットキー: rdev

- OS の入力イベントをグローバルに傍受できる
- Linux / macOS 両対応
- **Linux**: libxtst（X11 Record Extension）が必要
- **macOS**: システム環境設定でアクセシビリティ権限の付与が必要

#### 依存ライブラリのインストール

Linux (X11):
```
sudo apt install libxtst-dev libx11-dev
```

macOS: システム環境設定 → プライバシーとセキュリティ → アクセシビリティ でアプリを許可

---

## システム設計

### アーキテクチャ

```
┌─────────────────────────────────────────────────────┐
│  Main Thread (egui event loop)                      │
│                                                     │
│  ┌──────────────┐    ┌────────────────────────────┐ │
│  │ History Panel│    │  Interpretation Panel       │ │
│  │              │    │                             │ │
│  │ [item 1]     │    │  ▸ Hex Dump                 │ │
│  │ [item 2] ◀── │────│    48 65 6c 6c 6f           │ │
│  │ [item 3]     │    │  ▸ UUID  (not valid)        │ │
│  │              │    │  ▸ Color #ff5500 ████       │ │
│  └──────────────┘    │  ▸ File Path (not found)   │ │
│                      └────────────────────────────┘ │
│          ↑                                           │
│    mpsc::Receiver<HotkeyEvent>                       │
└──────────────────────┬──────────────────────────────┘
                       │
┌──────────────────────┴──────────────────────────────┐
│  Hotkey Listener Thread (rdev::listen)               │
│  Ctrl+Shift+H が押されたら channel で通知             │
└─────────────────────────────────────────────────────┘
```

### スレッド構成

| スレッド | 役割 |
|---------|------|
| Main Thread | egui の描画ループ、クリップボード読み取り、状態管理 |
| Hotkey Thread | rdev でグローバルキーイベントを傍受し、ホットキー検出時に channel 送信 |

### スレッド間通信

- `std::sync::mpsc::channel::<()>()` でホットキーイベントを通知
- クリップボード読み取りはメインスレッドで行う（arboard の制約）

---

## ファイル構成

```
clipboard-hack/
├── DESIGN.md
├── Cargo.toml
└── src/
    ├── main.rs              エントリポイント、eframe 起動
    ├── app.rs               アプリ状態、egui UI 定義
    ├── history.rs           ClipboardEntry, ClipboardHistory
    ├── hotkey.rs            グローバルホットキーリスナー
    └── interpreter/
        ├── mod.rs           Interpreter トレイト、get_interpreters()
        ├── hex.rs           HexInterpreter
        ├── uuid.rs          UuidInterpreter
        ├── color.rs         ColorInterpreter
        └── filepath.rs      FilePathInterpreter
```

---

## データモデル

### ClipboardEntry

```rust
struct ClipboardEntry {
    content: String,
    captured_at: std::time::SystemTime,
}
```

### ClipboardHistory

```rust
struct ClipboardHistory {
    entries: VecDeque<ClipboardEntry>,  // 新しい順
    max_size: usize,                     // デフォルト 50
}
```

### Interpreter トレイト

```rust
pub trait Interpreter: Send + Sync {
    fn name(&self) -> &str;
    fn interpret(&self, content: &str) -> Option<InterpretResult>;
}

pub struct InterpretResult {
    pub items: Vec<InterpretItem>,
}

pub struct InterpretItem {
    pub label: String,
    pub value: String,
    pub color: Option<[u8; 4]>,  // RGBA - カラーコード解釈時にスウォッチ表示
}
```

---

## インタープリター仕様

### HexInterpreter (`hex.rs`)

UTF-8 バイト列として16進数ダンプ表示。

| 出力項目 | 内容 |
|---------|------|
| Bytes (hex) | スペース区切りの16進数バイト列 |
| Length | バイト数 |
| UTF-8 chars | 文字数 |

常に結果を返す（全テキストに適用可能）。

### UuidInterpreter (`uuid.rs`)

UUID 形式として解析を試みる。

| 出力項目 | 内容 |
|---------|------|
| Version | UUID バージョン（1, 4, 5, ...） |
| Variant | UUID バリアント |
| Hyphenated | 標準のハイフン区切り形式 |
| Simple | ハイフンなし形式 |
| URN | URN 形式 |

パース失敗時は `None`（インタープリター非表示）。

### ColorInterpreter (`color.rs`)

CSS カラー形式を解析。

対応フォーマット:
- `#RRGGBB`（例: `#ff5500`）
- `#RGB`（例: `#f50`）
- `#RRGGBBAA`（例: `#ff550080`）
- `rgb(R, G, B)`

| 出力項目 | 内容 |
|---------|------|
| Preview | colored ██████ swatch |
| R / G / B | 10進数チャンネル値 |
| Hex | 正規化された #RRGGBB |
| HSL | 色相・彩度・明度 |

パース失敗時は `None`。

### FilePathInterpreter (`filepath.rs`)

絶対パスまたは `~/...` 形式のパスを解析。

| 出力項目 | 内容 |
|---------|------|
| Exists | true / false |
| Type | File / Directory / Symlink |
| Size | ファイルサイズ（バイト） |
| Parent | 親ディレクトリ |
| Filename | ファイル名 |
| Extension | 拡張子 |

絶対パスでない場合は `None`。

---

## UI レイアウト

```
┌─ Clipboard Hack ──────────────────────────────────────────┐
│  [Capture Now]  [Clear History]  Hotkey: Ctrl+Shift+H      │
├───────────────────┬───────────────────────────────────────┤
│ History           │ Content                               │
│                   │ ┌─────────────────────────────────┐  │
│ 2026-02-25 10:30  │ │ Hello World                     │  │
│ Hello World       │ └─────────────────────────────────┘  │
│                   │                                       │
│ 2026-02-25 10:28  │ ▼ Hex Dump                           │
│ #ff5500           │   Bytes: 48 65 6c 6c 6f 20 57 6f ... │
│                   │   Length: 11 bytes                    │
│ 2026-02-25 10:25  │                                       │
│ /home/user/doc... │ ▼ Color Code  (not applicable)        │
│                   │                                       │
│                   │ ▼ UUID  (not applicable)              │
│                   │                                       │
│                   │ ▼ File Path  (not applicable)         │
│                   │                                       │
└───────────────────┴───────────────────────────────────────┘
```

---

## 拡張方法

新しい解釈を追加する手順：

1. `src/interpreter/` に新しいファイルを作成（例: `base64.rs`）
2. `Interpreter` トレイトを実装
3. `src/interpreter/mod.rs` の `get_interpreters()` に追加
4. `Cargo.toml` に必要な依存があれば追加してビルド

---

## ホットキー

デフォルト: **Ctrl + Shift + H**

`src/hotkey.rs` の `HOTKEY_*` 定数を変更することで調整可能（ビルドし直し）。
