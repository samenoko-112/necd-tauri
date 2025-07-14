# NeCd Tauri

![header](https://img.shields.io/badge/NeCd-Tauri-blue?style=for-the-badge)

> YouTubeから動画をダウンロードするTauri+Vue.jsアプリケーション

## 概要

NeCd Tauriは、YouTubeから動画をダウンロードするためのデスクトップアプリケーションです。  
元のPython版NeCdをTauri+Vue.jsに移植したものです。

## 機能

### ダウンロード形式と品質設定
- **対応フォーマット**
  - 動画: mp4, mkv
  - 音声: mp3, opus, flac
- **品質設定**
  - 動画: HD(720p)～4K(2160p)を選択可能
  - 音声: 128kbps～320kbpsを選択可能
  - その他の形式: 自動で最高品質を選択

### Cookie認証
- **対応方法**
  - テキストファイルからの読み込み
  - Firefoxブラウザからの読み込み

### ダウンロード設定
- **同時接続数**
  - デフォルトで有効（最大16接続）
  - 0に設定することで無効化可能
- **プレイリスト対応**
  - プレイリスト用の最適化された保存形式
  - タイトルとインデックスを含むファイル名で保存

### メタデータ設定
- **サムネイル**
  - 埋め込み機能
  - 1:1比率へのクロップ機能（音楽ファイル向け）
- **チャプター**
  - 動画へのチャプター情報の埋め込み
- **メタデータ**
  - タイトル、アーティスト、アルバム情報などの自動埋め込み

## 必要条件

### 必須ソフトウェア
- **Node.js 18以上**
  - フロントエンドのビルドに必要

- **Rust 1.70以上**
  - Tauriアプリケーションのビルドに必要

- **yt-dlp**
  - コア機能の実装に必須
  - インストールコマンド:
  ```bash
  pip install yt-dlp
  ```

- **ffmpeg**
  - ファイル変換、マージ、メタデータ埋め込みに必要
  - 各OSのパッケージマネージャーからインストール可能

## セットアップ

### 1. 依存関係のインストール

```bash
# フロントエンド依存関係
npm install

# Rust依存関係（初回のみ）
cd src-tauri
cargo build
cd ..
```

### 2. 開発サーバーの起動

```bash
npm run tauri dev
```

### 3. ビルド

```bash
npm run tauri build
```

## 使用方法

1. アプリケーションを起動
2. ダウンロードしたい動画のURLを入力
3. 保存先フォルダを選択
4. 形式と品質を設定
5. 必要に応じてオプションを設定
6. 「実行」ボタンをクリック

## トラブルシューティング

### yt-dlpがインストールされていない場合
```bash
pip install yt-dlp
```

### ffmpegがインストールされていない場合

**Windows:**
```bash
# Chocolateyを使用
choco install ffmpeg

# または公式サイトからダウンロード
```

**macOS:**
```bash
# Homebrewを使用
brew install ffmpeg
```

**Linux:**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install ffmpeg

# Arch Linux
sudo pacman -S ffmpeg
```

### メタデータ埋め込みエラー
opusやflac形式にメタデータを埋め込む際にエラーが発生する場合は、`mutagen`ライブラリのインストールが必要です。

```bash
pip install mutagen
```

## 開発

### プロジェクト構造
```
necd-tauri/
├── src/                 # Vue.jsフロントエンド
│   ├── App.vue         # メインアプリケーション
│   └── main.ts         # エントリーポイント
├── src-tauri/          # Rustバックエンド
│   ├── src/
│   │   ├── lib.rs      # メインロジック
│   │   └── main.rs     # エントリーポイント
│   ├── Cargo.toml      # Rust依存関係
│   └── tauri.conf.json # Tauri設定
└── package.json        # Node.js依存関係
```

### 技術スタック
- **フロントエンド**: Vue.js 3 + TypeScript
- **バックエンド**: Rust + Tauri
- **UI**: カスタムCSS
- **ビルドツール**: Vite

## ライセンス

MIT License

## 元プロジェクト

このプロジェクトは [samenoko-112/necd](https://github.com/samenoko-112/necd) をTauri+Vue.jsに移植したものです。
