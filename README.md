# mcp-arxiv-server

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust製のarXiv論文検索MCP (Model Context Protocol) サーバーです。公式Rust MCP SDK ([rmcp](https://github.com/modelcontextprotocol/rust-sdk) 0.16) を使用しています。

## 必要環境

- Rust 1.85以上
- MCP対応クライアント（Claude Code、MCP Inspector など）

## ビルド

```bash
cargo build --release
```

ビルド後のバイナリは `target/release/mcp-arxiv-server` に生成されます。

## 使い方

Claude Codeに登録します。バイナリの絶対パスが必要なので、プロジェクトのルートで `pwd` を実行して確認してください。

```bash
# プロジェクトルートで絶対パスを確認
pwd
# 例: /Users/you/Desktop/mcp-arxiv-server

# 上で確認したパスを使って登録
claude mcp add arxiv /Users/you/Desktop/mcp-arxiv-server/target/release/mcp-arxiv-server
```

### プロンプト例

登録後、Claude Codeで以下のように依頼できます。

- 「arXivで拡散モデルの最新論文を検索して」
- 「RAG関連のarXiv論文を10件調べて」
- 「Yann LeCunの自己教師あり学習に関するarXiv論文を探して」

`search_arxiv` ツールが自動的に呼び出されます。

### MCP Inspectorによる動作確認

[MCP Inspector](https://github.com/modelcontextprotocol/inspector) を使って動作確認できます。`<絶対パス>` の部分は上記 `pwd` で確認したパスに置き換えてください。

```bash
npx @modelcontextprotocol/inspector <絶対パス>/target/release/mcp-arxiv-server
```

Inspector UIを開き、ツール一覧から `search_arxiv` を実行してレスポンスを確認してください。

## ツールリファレンス

### `search_arxiv`

指定したクエリに一致するarXiv論文を検索します。

**パラメータ:**

| 名前          | 型     | 必須   | デフォルト | 説明                                       |
| ------------- | ------ | ------ | ---------- | ------------------------------------------ |
| `query`       | string | はい   | —          | 検索クエリ（キーワード、著者、タイトル等） |
| `max_results` | number | いいえ | 5          | 取得する最大件数（最大20）                 |

**レスポンス:** 論文の配列。各要素のフィールドは `arxiv_id`, `title`, `authors`, `abstract`, `url`, `pdf_url`, `published_date`, `categories`。

## arXiv APIの利用について

本サーバーは公開されている [arXiv API](https://info.arxiv.org/help/api/index.html) を利用しています。arXivのガイドラインに従い、以下にご留意ください。

- リクエスト間隔は **3秒に1リクエスト程度** を上限の目安に
- 大規模な自動クロールは避ける
- 本プロジェクトはarXiv側のレート制限に依存しており、クライアント側のスロットリングは実装していません

## ライセンス

MITライセンスです。詳細は [LICENSE](LICENSE) ファイルを参照してください。
