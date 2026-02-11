# 3DGS Studio

**3DGS Studio** は、3D Gaussian Splatting の編集・作成を支援するデスクトップアプリケーションです。
現在は、**SAM 2 (Segment Anything Model 2)** を用いたインタラクティブな画像セグメンテーション機能（AIバックエンド）の実装を進めています。

## 特徴

* **Rust製 高速AI推論エンジン:** Pythonを使わず、Rust + ONNX Runtime で SAM2 モデルを直接駆動。
* **クロスプラットフォーム:** Tauri を採用し、Windows, macOS, Linux で動作（予定）。
* **インタラクティブな操作:** 画像をクリックするだけで、対象物を瞬時にマスク生成（実装中）。

## 技術スタック

* **Frontend:** React / TypeScript
* **Backend:** Rust
* **Framework:** [Tauri v2](https://tauri.app/)
* **AI Inference:** [ort](https://github.com/pykeio/ort) (ONNX Runtime bindings for Rust)
* **Model:** Meta SAM 2 (Tiny)

## プロジェクト構成

* `src-tauri/`: Tauri アプリケーション本体
* `crates/ai/`: SAM2 推論ロジックを担当する独立クレート
    * 画像の前処理 (Resize, Normalize)
    * エンコーダー・デコーダー パイプライン
* `assets/models/`: ONNXモデルファイルの格納場所

## セットアップと実行

### 前提条件

* Rust (最新の安定版)
* Node.js (v18以上推奨)
* Build Tools (Windowsの場合は C++ Build Tools)

### インストール

1.  リポジトリをクローンします:
    ```bash
    git clone [https://github.com/your-username/3dgs-studio.git](https://github.com/your-username/3dgs-studio.git)
    cd 3dgs-studio
    ```

2.  **ONNXモデルの準備 (重要):**
    SAM2のONNXモデル（Tiny版）をダウンロードし、以下のディレクトリに配置してください。
    ※ モデルファイルはサイズが大きいため、このリポジトリには含まれていません。

    **配置場所:**
    * `assets/models/sam2/sam2_hiera_tiny_encoder.onnx`
    * `assets/models/sam2/sam2_hiera_tiny_decoder.onnx`

3.  依存関係のインストール:
    ```bash
    npm install
    # または
    pnpm install
    ```

### テストの実行 (AIバックエンド)

現在、AI推論エンジンのユニットテストが実装されています。以下のコマンドで動作確認が可能です。

```bash
cargo test -p ai