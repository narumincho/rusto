## その場限りの個人的なRust スクリプト

---

Notion のデータベースから webフックを受け取り,
そのページが属するデータベースに対して操作をする

データベースには, ユーザー名, ユーザーIDと更新ボタンの3つのプロパティがある

- ユーザーID
  - 空の場合は, ユーザー名を見て, 対応するユーザーIDを設定する
  - ユーザー名が変わった場合は, ページに過去の名前のブロックを追加して,
    ユーザー名を変更する
- ページアイコンが未設定の場合は, ユーザーの顔スキンのアイコン画像を設定する

---

## Cloudflare R2 大容量ファイルアップローダー (r2_uploader)

2GB 以上の大容量動画ファイルを Cloudflare R2 に安定して並行・分割アップロードするための CLI ツールです。

### 動作特徴
- **マルチパートアップロード**: 2GB のファイルをメモリに載せきれない問題を防ぐため、ファイルをチャンク（既定 15MB）に分割してアップロードします。
- **並行アップロード**: 複数のチャンクを並行してアップロードすることで、転送速度を向上させます。
- **進捗状況表示**: コンソールに進捗バーと推定残り時間が表示されます。
- **クリーンアップ**: アップロード中にエラーが発生した場合、R2 側の不完全なアップロードデータを自動で破棄（Abort）し、無駄な課金が発生しないようにします。

### 使用方法

#### 1. 環境変数の設定 (推奨)
アクセス資格情報を環境変数としてあらかじめ設定しておくと、コマンド入力がシンプルになります。

```bash
export R2_ACCOUNT_ID="your_account_id"
export R2_ACCESS_KEY_ID="your_access_key_id"
export R2_SECRET_ACCESS_KEY="your_secret_access_key"
```

#### 2. コマンドの実行
以下のコマンドでアップロードを開始します。パフォーマンスを出すために `--release` ビルドでの実行を推奨します。

```bash
cargo run --release --bin r2_uploader -- \
  --bucket <R2バケット名> \
  --file-path /path/to/large_video.mp4 \
  --key uploads/large_video.mp4
```

#### 引数オプションによるパラメータ直接指定
環境変数を使わずに、引数として接続情報を直接渡すこともできます。

```bash
cargo run --release --bin r2_uploader -- \
  --account-id <ACCOUNT_ID> \
  --access-key-id <ACCESS_KEY> \
  --secret-access-key <SECRET_KEY> \
  --bucket <R2バケット名> \
  --file-path /path/to/large_video.mp4 \
  --key uploads/large_video.mp4
```

#### チューニングオプション
- `-c, --chunk-size-mb <MB>`: 分割するチャンクのサイズ (最小 5MB、デフォルト 15MB)
- `-p, --concurrency <NUM>`: 同時アップロード接続数 (デフォルト 4)

例（チャンクサイズを20MB、同時アップロード数を8にして実行する場合）:
```bash
cargo run --release --bin r2_uploader -- \
  --bucket my-bucket \
  --file-path /path/to/video.mp4 \
  --key folder/video.mp4 \
  --chunk-size-mb 20 \
  --concurrency 8
```

