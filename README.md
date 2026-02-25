# Vital Monitor

**Vital Monitor** は macOS 用のシステムメトリクス監視アプリです。CPU 使用率、メモリ圧力、ネットワーク遅延をトレイバーに常時表示し、アラートを設定できます。

## 機能

- **リアルタイム監視**: CPU、メモリ、ネットワークメトリクスを 3 秒間隔で更新
- **トレイ表示**: メニューバーにシステム情報を常時表示
- **アラート機能**: 各メトリクスの閾値を超えた場合に通知
- **表示モード切り替え**: リスト表示またはローテーション表示を選択可能
- **カスタマイズ可能**: 表示項目のON/OFF、アラートの有効化/無効化

## スクリーンショット

トレイバーに以下のような形式で表示されます：
```
CPU 45% Mem 60% NW 25ms
```

## インストール

### ビルド済みバイナリ

[Releases](https://github.com/z-fujimori/VitalMonitor/releases) ページから最新版をダウンロードしてください。


## 使用方法

### トレイメニュー

アプリを起動するとメニューバーにアイコンが表示されます。クリックすると以下のオプションが表示されます：

- **CPU/MEM/NW**: 表示項目の ON/OFF 切り替え
- **Display Mode**
  - List: すべてのメトリクスを常時表示
  - Rotation: 複数メトリクスをローテーション表示
- **Show Alert**: アラート表示の ON/OFF
- **Quit**: アプリを終了

### 設定

メトリクスのアラート閾値は `src-tauri/src/metrics/types.rs` で定義されています：

```rust
// CPU使用率
CpuPolicy::default()        // 50%: Normal, 75%: Warning, 90%: Critical

// メモリ圧力
MemoryPolicy::default()     // 60%: Normal, 75%: Warning, 90%: Critical

// ネットワーク遅延
NetworkPolicy::default()    // 50ms: Normal, 200ms: Warning, 450ms: Critical
```

## 技術スタック

- **フロントエンド**: React 19 + TypeScript + Vite
- **デスクトップフレームワーク**: Tauri 2
- **バックエンド**: Rust (tokio 非同期ランタイム)
- **ビルドツール**: Cargo

## プロジェクト構造

```
vaital-monitor/
├── src/                          # フロントエンド (React)
│   ├── App.tsx
│   ├── main.tsx
│   └── assets/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs               # メインアプリケーション
│   │   ├── main.rs              # エントリーポイント
│   │   ├── mac_metrics.rs        # macOS メトリクス取得
│   │   └── metrics/
│   │       └── types.rs         # 型定義・ポリシー
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── tsconfig.json
└── vite.config.ts
```

## 開発ガイド

### メトリクス追加

新しいメトリクスを追加する場合：

1. `src-tauri/src/metrics/types.rs` で `Policy<V>` を実装
2. `src-tauri/src/mac_metrics.rs` で取得関数を実装
3. `src-tauri/src/lib.rs` の `spawn_tray_updater` に追加

### UI メニューの追加

トレイメニューを拡張する場合：

1. `src-tauri/src/ui/mod.rs` で `TrayUiState` に新しいメニューアイテムを追加
2. `src-tauri/src/lib.rs` の `run()` 関数でメニュー構成を更新
3. イベントハンドラーで処理を実装

## ライセンス

MIT

## 作者

[@z-fujimori](https://github.com/z-fujimori)
