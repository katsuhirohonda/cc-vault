# cc-vault 要件定義書

## 概要
Claude Codeの会話履歴を一元管理・検索できるRustパッケージ

## 目的
Claude Codeで発言した内容が様々な場所に散在しているため、これらを統合的に管理し、効率的に検索・参照できるシステムを構築する。

## コア機能

### 1. データ収集
- **データソース**: `~/.claude/projects/` 配下のJSON Lines (.jsonl)ファイル
- **収集方法**: PC内の該当ディレクトリを参照し、自動的に全プロジェクトの会話履歴を読み込み
- **データ形式**: 
  - JSON Lines形式（各行が独立したJSONオブジェクト）
  - フィールド: type, message, uuid, timestamp, parentUuid等

### 2. データストレージ
- **データベース**: Docker上のDuckDB
- **選定理由**:
  - カラムナストレージで分析クエリが高速
  - FTS（Full Text Search）拡張で全文検索が可能
  - 軽量でコンテナ化が容易
  - SQLサポートで複雑な検索条件も柔軟に実装可能

### 3. 検索機能
- **キーワード検索**（全文検索）
- **日付範囲での絞り込み**
- **プロジェクト/ディレクトリごとの検索**
- **正規表現検索**
- **お気に入りでの絞り込み**

### 4. ユーザーインターフェース
- **形式**: CLI/TUI（ターミナルユーザーインターフェース）
- **主要機能**:
  - インタラクティブな会話選択
  - お気に入り機能（選択してマーク、後で絞り込み表示）
  - 検索結果の見やすい表示

### 5. 開発環境
- **言語**: Rust
- **開発環境**: devContainer
- **含まれるツール**:
  - Rust + cargo
  - Docker（DuckDBコンテナ用）
  - rust-analyzer

## データスキーマ（案）

```sql
-- 会話履歴テーブル
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    project_path TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    message_type TEXT NOT NULL,
    content TEXT NOT NULL,
    user_role TEXT,
    is_favorite BOOLEAN DEFAULT FALSE,
    tags TEXT[] -- タグ配列
);

-- 全文検索用インデックス
CREATE INDEX idx_conversations_content_fts ON conversations USING FTS(content);

-- 日付検索用インデックス
CREATE INDEX idx_conversations_timestamp ON conversations(timestamp);

-- プロジェクト検索用インデックス
CREATE INDEX idx_conversations_project ON conversations(project_path);
```

## 使用例

```bash
# キーワード検索
cc-vault search "Rust error handling"

# プロジェクトと日付範囲を指定した検索
cc-vault search --project /path/to/project --from "2024-01-01" --to "2024-12-31"

# お気に入りのみを表示
cc-vault search --favorite

# 正規表現検索
cc-vault search --regex "fn\s+\w+\s*\("

# インタラクティブモード（TUI）
cc-vault interactive
```

## 技術スタック
- **言語**: Rust
- **データベース**: DuckDB (Dockerコンテナ)
- **CLI/TUIフレームワーク**: clap (CLI), ratatui (TUI)
- **JSONパーサー**: serde_json
- **非同期処理**: tokio

## 制約事項
- 初期バージョンではClaude Code専用とする
- 追加機能（エクスポート、バックアップ、統計等）は将来のバージョンで検討
- ローカル環境での使用を前提とし、クラウド同期機能は含まない
