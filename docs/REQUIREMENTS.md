# Daily Work Memo v0.1
## 要件定義・設計方針・開発指示書

**ステータス:** v0.1 実装用仕様  
**対象:** macOS / Windows  
**実装候補:** Rust + Iced 0.14  
**目的:** 1日の業務メモを、低スペックWindowsでもストレスなく書き続けられる「今日の1枚」のメモ帳を作る。

---

## 1. このプロダクトが解決すること

### 1.1 解決したい問題

業務中には、以下のような短い情報が断続的に発生する。

- 作業中に気づいたこと
- Redmineで確認した内容
- Slackで確認した内容
- メールで確認した内容
- 後で思い出したいこと
- アクションアイテム
- 打ち合わせ中のメモ
- 他の画面からコピーしたテキスト

これらは「正式なタスク管理」や「ナレッジ管理」を始めるほどではない一方、その場で残さないと失われる。

既存のノートアプリやIDEでも記録はできるが、Vault、ファイルツリー、タブ、Markdown機能、プラグイン、同期、Git、AIなど、今回の用途には不要な概念や処理を抱えやすい。

本アプリは、**朝から終業まで開きっぱなしにする1枚のメモ帳**に用途を限定する。

### 1.2 提供価値

価値の中心は機能量ではなく、次の3点である。

1. **即座に書ける** — アプリを前面に出した瞬間、そのまま入力できる。
2. **書いたものを失わない** — 保存操作を意識せず、ローカルファイルへ継続保存される。
3. **最低限の意味だけ付けられる** — 作業種別を1〜2キーで挿入し、後から人間やLLMが読み返しやすい。

この3点に寄与しない機能は、原則としてv0.1に追加しない。

### 1.3 このアプリではないもの

本アプリは以下を目指さない。

- Markdownエディタ
- IDE
- タスク管理システム
- Redmine / Slack / メールクライアント
- PKM / ナレッジベース
- クリップボード履歴管理アプリ
- AIチャットクライアント
- 従業員監視・作業トラッキングアプリ

---

## 2. プロダクト原則

### P1. 「今日の1枚」が主役

通常画面には、今日のテキストだけを表示する。常設のファイルツリー、サイドバー、タブ、プレビューは置かない。

### P2. 保存という操作をユーザーに要求しない

入力内容は自動保存する。`Ctrl/Cmd + S` を覚える必要はない。

### P3. 構造化しすぎない

入力時にプロジェクト、タグ、担当者、期限などを要求しない。必要であれば人間が自由文で書く。

### P4. 種類付けは「軽い意味タグ」

作業種別は厳密なデータモデルではない。人間が「これはSlack確認」「これはアクション」と軽く意味を残すための補助である。

### P5. ネットワークを通常動作に持ち込まない

v0.1は完全ローカルで動作する。ネットワーク接続の有無で入力体験が変わってはならない。

### P6. 軽量性は品質要件

「軽いと嬉しい」ではなく、**重い場合はこのプロダクトの価値を満たしていない**と判断する。

### P7. 原文が正

将来LLM整理を追加しても、日次rawメモは残す。整理結果で原文を上書きしない。

---

## 3. v0.1のスコープ

### 3.1 必須機能

v0.1で実装するのは以下のみとする。

1. 今日の日次テキストを開く
2. 普通の複数行テキスト編集
3. 自動保存
4. 作業種別セレクタのフロート表示
5. 作業種別の挿入
6. 作業種別・キー・テンプレートのconfigカスタマイズ
7. 今日の全文をクリップボードへコピー
8. 日付が変わった場合の日次ファイル切り替え
9. 保存失敗をユーザーへ明示
10. Windows / macOS向けビルド

### 3.2 v0.1では実装しないもの

以下は明示的に非スコープとする。

- LLM API連携
- ChatGPT / Codex / Claudeとの直接連携
- Markdownシンタックスハイライト
- Markdownプレビュー
- Markdown lint / formatter
- ファイルツリー
- タブ
- 任意ファイルを開くUI
- Git統合
- クリップボード監視
- アクティブアプリ監視
- 全文検索
- タグ管理
- プロジェクト管理
- リマインダー通知
- カレンダー連携
- クラウド同期
- プラグインシステム
- 自動アップデート
- 常駐トレイ
- グローバルホットキー

「便利そうだから」で上記をv0.1へ戻さないこと。

---

## 4. 想定利用フロー

### 4.1 朝

1. アプリを起動する。
2. OSのローカル日付から今日の日次ファイルを決定する。
3. 既存ファイルがあれば読み込み、なければ空のファイルを作る。
4. テキストエリアへ自動フォーカスする。
5. そのまま入力できる。

起動時にワークスペース選択、ファイル選択、テンプレート選択などは表示しない。

### 4.2 日中

通常は自由に書く。

例:

```text
API側は問題なさそう
画面側のバリデーション確認する
```

作業種別を付けたい場合のみ、`Ctrl/Cmd + K` を押す。

```text
┌─────────────────────────┐
│ W  作業メモ             │
│ R  Redmine確認          │
│ S  Slack確認            │
│ M  メール                │
│ D  リマインド            │
│ A  アクションアイテム    │
│ G  打ち合わせ            │
└─────────────────────────┘
```

`A` を押すと現在のカーソル位置へ、例えば以下を挿入する。

```text
[16:42] アクションアイテム

```

テンプレートを設定している場合は、その直後に追加する。

### 4.3 終業時

v0.1ではLLM処理を実装しない。

`Ctrl/Cmd + Shift + C` で今日の全文をクリップボードへコピーし、ユーザーが手動でChatGPT、Codex等へ貼る。

将来はこの地点を自動整理・Inbox出力へ拡張できるが、v0.1の内部設計でLLM provider抽象化などを先回りして作らない。

---

## 5. UI要件

### 5.1 メインウィンドウ

通常時のコンテンツは原則としてテキストエリアのみとする。

```text
┌──────────────────────────────────────────────┐
│ Daily Work Memo — 2026-09-30                 │  ← OSウィンドウタイトル
├──────────────────────────────────────────────┤
│                                              │
│  今日のメモ                                  │
│                                              │
│  ...                                         │
│                                              │
│                                              │
└──────────────────────────────────────────────┘
```

アプリ内に常設ツールバーを置かない。

### 5.2 テキスト表示

- プレーンテキスト
- 折り返しON
- 横スクロールを基本的に発生させない
- OS標準フォントまたは自然なシステムフォールバックを使用
- 独自フォントをバンドルしない
- 日本語IMEを第一級の要件として扱う
- 文字サイズはconfigで変更可能にしてよい

### 5.3 フロートセレクタ

`Ctrl/Cmd + K` でメインテキスト上にオーバーレイ表示する。

要件:

- 背景処理で別ウィンドウを作らない
- 表示中もアプリ内で完結する
- `↑` / `↓` で選択
- `Enter` で決定
- `Esc` で閉じる
- configで定義された`key`を押すと即決定
- マウスクリックでも決定可能
- 選択後はエディタへフォーカスを戻す
- 入力カーソルは挿入したテンプレート末尾へ置く

### 5.4 状態表示

正常時は保存状態を常時表示しなくてよい。

以下の場合のみ、控えめだが見落とせない表示を行う。

- config読み込み失敗
- 日次ファイル読み込み失敗
- 保存失敗
- 全文コピー失敗

保存失敗時は、ユーザーが「保存された」と誤認できない状態にする。

---

## 6. キーボード仕様

`Primary` はmacOSでは`Cmd`、Windowsでは`Ctrl`を意味する。

| 操作 | キー | 必須 |
|---|---|---|
| 作業種別セレクタ | `Primary + K` | Yes |
| 今日の全文をコピー | `Primary + Shift + C` | Yes |
| Undo | `Primary + Z` | Yes |
| Redo | OS慣例に従う | Yes |
| Copy / Cut / Paste | OS慣例に従う | Yes |
| Select All | `Primary + A` | Yes |
| 保存 | 自動 | Yes |

`Primary + S` を押した場合は即時flushしてもよいが、主要な利用方法にはしない。

エディタ標準動作を壊す独自ショートカットを増やさない。

---

## 7. 作業種別とテンプレート

### 7.1 初期作業種別

初期configには以下を入れる。

| id | label | key | template |
|---|---|---|---|
| work | 作業メモ | w | 空 |
| redmine | Redmine確認 | r | 空 |
| slack | Slack確認 | s | 空 |
| mail | メール | m | 空 |
| reminder | リマインド | d | 空 |
| action | アクションアイテム | a | 空 |
| meeting | 打ち合わせ | g | 空 |

**テンプレートの初期値はすべて空**とする。

### 7.2 挿入形式

デフォルトのエントリ見出し:

```text
[{time}] {label}
```

例:

```text
[10:24] Slack確認

```

見出し形式はconfigの`entry_header`で変更可能とする。

### 7.3 テンプレート

`template`は見出し直後へそのまま挿入する。

例:

```toml
template = "- [ ] "
```

結果:

```text
[16:42] アクションアイテム
- [ ] 
```

v0.1でサポートする置換変数は次の3つだけとする。

- `{time}` — ローカル時刻 `HH:mm`
- `{date}` — ローカル日付 `YYYY-MM-DD`
- `{label}` — entry typeのlabel

独自スクリプト、条件分岐、マクロ言語は実装しない。

### 7.4 挿入位置

- 現在のカーソル位置へ挿入する。
- 選択範囲がある場合、選択を破壊してテンプレートへ置き換えない。選択末尾またはカーソル位置へ挿入する実装を優先する。
- 既存行と見出しが連結しないよう、必要な改行を最小限補う。
- 不要な空行を大量に自動生成しない。

---

## 8. config仕様

### 8.1 形式

TOMLを使用する。

### 8.2 configの配置

OS標準のユーザーconfigディレクトリを使う。

- macOS: `~/Library/Application Support/<app-id>/config.toml`
- Windows: `%APPDATA%\\<app-id>\\config.toml`

実装ではパスをハードコードせず、OS標準ディレクトリ解決ライブラリを利用する。

### 8.3 初期config例

```toml
# Daily Work Memo

data_dir = ""
font_size = 15
autosave_debounce_ms = 250
entry_header = "[{time}] {label}"

[[entry_types]]
id = "work"
label = "作業メモ"
key = "w"
template = ""

[[entry_types]]
id = "redmine"
label = "Redmine確認"
key = "r"
template = ""

[[entry_types]]
id = "slack"
label = "Slack確認"
key = "s"
template = ""

[[entry_types]]
id = "mail"
label = "メール"
key = "m"
template = ""

[[entry_types]]
id = "reminder"
label = "リマインド"
key = "d"
template = ""

[[entry_types]]
id = "action"
label = "アクションアイテム"
key = "a"
template = ""

[[entry_types]]
id = "meeting"
label = "打ち合わせ"
key = "g"
template = ""
```

`data_dir = ""` の場合はOS標準のアプリデータディレクトリを使用する。

`~`展開をサポートする場合はWindows/macOSで同じ規則にする。難しければv0.1では絶対パスのみ許可し、曖昧な展開は実装しない。

### 8.4 config検証

起動時に以下を検証する。

- `id` が空でない
- `id` が重複していない
- `label` が空でない
- `key` が1文字である
- `key` が大文字小文字を無視して重複しない
- `autosave_debounce_ms` が極端な値でない

configにエラーがある場合:

1. ユーザーconfigを勝手に書き換えない。
2. 最低限の内蔵デフォルトで起動可能なら起動する。
3. configエラーを画面上で通知する。
4. ログに具体的な原因を出す。

config変更のホットリロードはv0.1では不要。再起動で反映する。

---

## 9. データ保存仕様

### 9.1 保存形式

日次rawはUTF-8プレーンテキストとする。

- BOMなし
- 改行は内部・保存ともLFを基本とする
- 拡張子は`.txt`

Markdown構文を要求しない。

### 9.2 ディレクトリ構造

```text
<data_dir>/
  daily/
    2026-09-29.txt
    2026-09-30.txt
    2026-10-01.txt
```

DBは使用しない。

ユーザーがファイルシステムから直接rawを読める状態を維持する。

### 9.3 自動保存

テキストが変更されたら保存をスケジュールする。

推奨挙動:

1. 編集イベントを受ける。
2. 250ms（config値）入力が途切れたら保存する。
3. 新しい編集が入ればタイマーをリセットする。
4. ウィンドウがフォーカスを失う場合は即時flushする。
5. アプリ終了時は未保存内容を必ずflushする。

ディスクI/OでUIスレッドをブロックしない。

### 9.4 原子的保存

日次ファイルは、可能な限り「旧内容または新内容のどちらかが残る」保存を行う。

Rust実装では、Windows/macOS両対応のatomic replaceを提供する小さなライブラリを使用してよい。2026-09時点の候補として`atomic-write-file`がある。

保存ロジックを独自に複雑化するより、テスト済みライブラリを優先する。

### 9.5 保存失敗

保存失敗は黙って無視しない。

- 画面内にエラーを残す
- メモリ上の最新内容は保持する
- 次のautosaveでも再試行可能にする
- 未保存のまま終了しようとした場合は警告する

「保存できなかったが終了して内容を失う」を最優先で防ぐ。

---

## 10. 日付切り替え

### 10.1 起動時

OSローカル日付を取得し、`YYYY-MM-DD.txt`を開く。

### 10.2 アプリを日跨ぎで開きっぱなしにした場合

入力中に突然別ファイルへ切り替えることは避ける。

v0.1では次の挙動を推奨する。

- ウィンドウが再びフォーカスされた時に日付変更を確認する。
- 日付が変わっていたら旧日付をflushする。
- 今日の日次ファイルへ切り替える。
- すでに今日のファイルがあれば読み込む。

この処理は単体テスト可能な純粋ロジックとして切り出す。

---

## 11. 手動LLM連携のための最小機能

LLM統合はv0.1では行わない。

代わりに、今日の内容を簡単に他のアプリへ渡せるようにする。

### 11.1 全文コピー

`Primary + Shift + C`で今日のメモ全文をOSクリップボードへコピーする。

成功時のトーストは不要。失敗時のみ通知してよい。

### 11.2 将来拡張との境界

将来想定:

```text
raw daily memo
      ↓
LLMによる整理
      ↓
inboxへの出力
```

ただしv0.1では以下を作らない。

- `LlmProvider` trait
- API key設定
- model設定
- prompt設定画面
- Inbox schema
- 自動送信

必要になった時点で、実際の利用方法を見てから設計する。

---

## 12. 技術選定

### 12.1 言語

**Rust stable / Rust 2024 edition**を第一候補とする。

理由:

- 単一ネイティブ実行ファイルにしやすい
- Node/Electronランタイムを不要にできる
- Windows/macOSを同一コードベースで扱える
- ローカルファイル中心の小さなアプリに適する

Rust自体を目的化しない。軽量性・IME・入力品質を満たさなければUI技術を再検討する。

### 12.2 GUI

**Iced 0.14を第一候補**とする。

2026-09時点でIced 0.14には複数行`TextEditor`があり、キーバインドをカスタマイズできるため、本アプリのv0.1を「エディタエンジン自作」なしで実現できる可能性が高い。

ただし、Iced採用は最初の性能・IMEスパイク合格を条件とする。

### 12.3 最小依存候補

```text
iced                GUI / TextEditor
serde               config deserialize
toml                config format
directories         OS標準ディレクトリ
chrono              local date/time
atomic-write-file   atomic file replace
```

必要性が発生するまで以下は入れない。

```text
tokio
reqwest
ropey
tree-sitter
comrak
pulldown-cmark
notify
libgit2
SQLite
```

依存追加時は「どのv0.1要件を満たすためか」をPR/コミット説明に書く。

---

## 13. 最初に行う技術スパイク

本実装に入る前に、Icedで軽量性と入力品質を確認する。

### 13.1 スパイク内容

1つのウィンドウにIced `TextEditor`だけを置いた最小アプリを作る。

確認項目:

- Windowsで日本語IME入力
- macOSで日本語IME入力
- コピー / ペースト
- Undo / Redo
- マウス選択
- キーボード選択
- 1万行程度のテキスト
- 起動速度
- アイドルCPU
- メモリ使用量
- 連続入力中の引っかかり

### 13.2 Go / No-Go

以下に重大な問題があれば、v0.1実装へ進む前にGUI toolkitを再検討する。

- 日本語IMEの変換・確定が壊れる
- キー入力に体感できる遅延がある
- 低スペックWindowsで起動が明確に重い
- アイドル時に継続的な高CPU使用がある
- 通常サイズの日次メモでスクロールが引っかかる

このスパイクを飛ばして機能実装を先に進めない。

---

## 14. 性能要件

性能は意図した低スペックWindows実機で最終確認する。

### 14.1 目標値

以下はv0.1の性能予算とする。

| 指標 | 目標 | 再検討ライン |
|---|---:|---:|
| 起動〜入力可能（cold） | 1秒以内 | 明確に1秒超が常態化 |
| 起動〜入力可能（warm） | 500ms以内 | 1秒超 |
| アイドルCPU | 1%未満を目標 | 継続的に数% |
| 常用時RSS | 100MB以下を目標 | 150MB超が常態化 |
| 日次テキスト | 1MBまで快適 | 数百KBで引っかかる |
| autosave | 入力をブロックしない | 保存時に文字入力が止まる |

数値は目的ではない。「メモを残そう」と思ってアプリ操作をためらう体感がないことを優先する。

### 14.2 性能を守るための禁止事項

- 入力ごとの同期ディスク書き込み
- 入力ごとの全文parse
- 常時ファイルシステム監視
- 不要な1秒未満周期のpolling
- WebView
- 内蔵ブラウザ
- バックグラウンドAI処理
- 起動時ネットワークアクセス

---

## 15. アーキテクチャ

過剰な抽象化を避け、以下程度に分離する。

```text
src/
  main.rs          起動、Iced bootstrap
  app.rs           App state / message / update
  editor.rs        TextEditor操作、挿入処理
  entry.rs         EntryType、template展開
  config.rs        config load / validation
  storage.rs       daily path、load、atomic save
  date.rs          日付切り替え判定
  ui/
    mod.rs
    entry_picker.rs
```

### 15.1 状態の概念例

```rust
struct AppState {
    active_date: NaiveDate,
    editor: text_editor::Content,
    entry_picker: EntryPickerState,
    config: Config,
    save_state: SaveState,
}
```

これは実装を拘束するAPIではない。必要以上にRepository層、UseCase層、Domain層を増やさないこと。

### 15.2 重要な分離

最低限、以下だけはUIロジックから分離する。

- config parse / validate
- template expansion
- 日次ファイルパス決定
- atomic save
- 日付切り替え判定

これらは単体テスト可能にする。

---

## 16. エラー処理

### 16.1 起動時ファイル読み込み失敗

- エラーを表示
- 空内容で同じファイルへ上書きしない
- 明示的に復旧するまで破壊的操作を避ける

### 16.2 config失敗

- デフォルトconfigで起動可能なら起動
- エラー理由を表示
- 元configは変更しない

### 16.3 保存失敗

- 非モーダルだが持続する警告
- メモリ内編集を継続可能
- 再保存を試せる
- 終了時に未保存なら確認

### 16.4 panic

releaseビルドでpanicが発生した場合にrawファイルを消したり空ファイルで上書きしたりしないことを最優先する。

---

## 17. プライバシー・セキュリティ

v0.1は以下を守る。

- 入力内容をネットワーク送信しない
- テレメトリを送信しない
- クリップボードを監視しない
- OS上の他アプリ内容を監視しない
- ユーザーが貼り付けた内容だけを保持する
- rawはユーザーのローカルファイルとして保存する

ログへ本文を出力しない。デバッグログにも日次メモ本文を含めない。

---

## 18. テスト要件

### 18.1 Unit Test

最低限以下を自動テストする。

#### config

- 正常config読み込み
- 重複idの拒否
- 重複keyの拒否
- 空labelの拒否
- 不正TOML

#### template

- `{time}`展開
- `{date}`展開
- `{label}`展開
- 空template
- 日本語template

#### storage

- 日付→ファイルパス
- 新規ファイル
- 既存ファイル読み込み
- UTF-8日本語
- atomic overwrite
- 保存失敗のエラー伝播

#### date rollover

- 同日なら切り替えない
- 翌日なら切り替える
- 年越し
- 月跨ぎ

### 18.2 UI手動テスト

Windows/macOSの両方で確認する。

- 日本語IMEで連続入力
- IME変換中にカーソル移動
- 英数/日本語混在
- 絵文字
- 長文paste
- Undo / Redo
- Copy / Cut / Paste
- `Primary + K` → key選択
- picker表示中にEsc
- picker選択後のフォーカス復帰
- autosave中の連続入力
- ウィンドウ最小化/復帰
- アプリ強制終了後のraw確認

### 18.3 回帰テスト用サンプル

`fixtures/`に本文内容を含むテスト専用ファイルを用意する。

実ユーザーの日次rawをfixtureへコピーしない。

---

## 19. 開発手順

実装エージェントは以下の順序を守る。

### Phase 0: Performance / IME Spike

**成果物:** 最小TextEditorアプリと測定結果

- Rust project作成
- Iced TextEditor表示
- Windows/macOSでビルド
- 日本語IME確認
- 起動時間 / CPU / RSS確認
- Go/No-GoをREADMEへ記録

ここでNGなら次へ進まない。

### Phase 1: Daily Buffer

**成果物:** 今日のテキストを読み書きできる

- OS標準config/data path
- 今日の日付決定
- daily file load/create
- TextEditorへ読み込み
- autosave debounce
- atomic write
- save error state

### Phase 2: Entry Picker

**成果物:** `Primary + K`から作業種別を挿入できる

- EntryType model
- 初期7種類
- picker overlay
- arrows / Enter / Esc
- accelerator key
- header生成
- template展開
- editor focus復帰

### Phase 3: Config

**成果物:** configから種類・キー・テンプレートを変更できる

- config TOML load
- initial config生成
- validation
- invalid config fallback
- font size / autosave interval
- custom data_dir

### Phase 4: Daily Rollover / Manual Handoff

**成果物:** 一日運用できる

- focus復帰時の日付確認
- old day flush
- new day load
- `Primary + Shift + C`全文コピー

### Phase 5: Hardening

**成果物:** v0.1 release candidate

- Windows poor machine実測
- macOS実測
- IME再確認
- abnormal terminationテスト
- file permission errorテスト
- config errorテスト
- dependency整理
- release build

---

## 20. Codex / 開発エージェントへの実装ルール

この文書を開発指示として渡す場合、以下を優先すること。

1. **仕様にない機能を善意で追加しない。**
2. 実装前にPhase 0を行う。
3. UIの見栄えより入力レスポンスを優先する。
4. エディタエンジンを自作しない。
5. Markdown parserを入れない。
6. DBを入れない。
7. ネットワーク機能を入れない。
8. LLM abstractionを先に作らない。
9. config DSLを作らない。
10. 保存失敗を握りつぶさない。
11. 日次raw本文をログへ出さない。
12. 依存crateは必要最小限にする。
13. 1つの変更ごとに`cargo fmt`、`cargo clippy`、`cargo test`を通す。
14. Windows/macOS固有コードを増やす前に共通実装で済まないか確認する。
15. 仕様に判断がない場合は「最も小さく、最も戻しやすい実装」を選ぶ。

### 20.1 完了報告フォーマット

各Phase完了時に以下を報告する。

```text
## Implemented
- ...

## Verification
- cargo fmt: pass
- cargo clippy: pass
- cargo test: pass
- Windows manual test: ...
- macOS manual test: ...

## Performance
- cold start: ...
- warm start: ...
- idle CPU: ...
- RSS: ...

## Deviations from spec
- none / ...

## Next
- ...
```

仕様から逸脱した場合は必ず明記する。

---

## 21. リポジトリ推奨構成

```text
daily-work-memo/
  Cargo.toml
  Cargo.lock
  README.md
  LICENSE
  src/
    main.rs
    app.rs
    editor.rs
    entry.rs
    config.rs
    storage.rs
    date.rs
    ui/
      mod.rs
      entry_picker.rs
  tests/
    config_test.rs
    storage_test.rs
    template_test.rs
    date_test.rs
  fixtures/
  docs/
    REQUIREMENTS.md
    PERFORMANCE.md
```

`docs/PERFORMANCE.md`にはPhase 0およびrelease candidateでの実測値を残す。

---

## 22. CI / Release

### 22.1 CI

GitHub Actions等を使う場合、Windows/macOSネイティブrunnerで以下を実行する。

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

### 22.2 v0.1配布

v0.1では配布機構を作り込まない。

- Windows: release `.exe`または最小限のパッケージ
- macOS: `.app` bundle
- コード署名、自動更新、インストーラ最適化は必要になってから検討

配布機能のために本体を重くしない。

---

## 23. v0.1受け入れ基準

以下をすべて満たせばv0.1完成とする。

### Core

- [ ] Windowsで起動すると今日のメモが即座に編集可能
- [ ] macOSで起動すると今日のメモが即座に編集可能
- [ ] 日本語IMEが実用上問題なく動く
- [ ] 入力内容が自動保存される
- [ ] 保存のためのユーザー操作が不要
- [ ] 再起動後も今日の内容が復元される
- [ ] 日次rawが普通の`.txt`として読める

### Entry Types

- [ ] `Primary + K`でpickerが出る
- [ ] 7種類の初期entry typeが出る
- [ ] accelerator keyで即選択できる
- [ ] 選択したlabelと時刻が挿入される
- [ ] template空でも正常動作する
- [ ] configでlabel/key/templateを変更できる

### Safety

- [ ] atomic saveが動く
- [ ] 保存失敗が通知される
- [ ] 保存失敗時にメモリ上の内容を失わない
- [ ] configエラーでユーザーconfigを破壊しない
- [ ] 本文をログへ出さない

### Daily Workflow

- [ ] 日付変更後、次回focus時に今日のファイルへ移る
- [ ] 昨日のrawはそのまま残る
- [ ] `Primary + Shift + C`で今日の全文をコピーできる

### Lightweight

- [ ] 意図した低スペックWindows実機で入力に体感的な引っかかりがない
- [ ] autosaveが入力を止めない
- [ ] アイドル時に不要な継続処理をしていない
- [ ] 性能測定値が`docs/PERFORMANCE.md`に残っている

---

## 24. v0.1後に検討してよいもの

実際に数週間使い、明確な摩擦が確認されたものだけ追加候補にする。

候補:

- 終業時LLM整理
- 指定Inboxへの出力
- 最近数日のraw参照
- アプリを前面に出すグローバルショートカット
- 特定entry typeの直接ショートカット
- 検索
- 最小限の「今日をコピー」以外のhandoff

以下は特に慎重に扱う。

- クリップボード監視
- アクティブアプリ監視
- Git
- Markdown機能
- AI常駐
- プラグイン

追加判断は「できるか」ではなく、**毎日の入力摩擦を下げるか**で行う。

---

## 25. 開発開始時にそのまま渡す指示

```text
このリポジトリで Daily Work Memo v0.1 を実装してください。

docs/REQUIREMENTS.md を唯一の機能仕様として扱ってください。
仕様にない機能は追加しないでください。

最初に Phase 0 の Iced TextEditor performance / IME spike を実施してください。
WindowsとmacOSで、日本語IME、起動時間、アイドルCPU、RSS、連続入力の体感を確認し、docs/PERFORMANCE.mdへ結果を記録してください。
重大な問題がなければPhase 1以降を順に実装してください。

重要事項:
- 本体は「今日の1枚」のプレーンテキストメモ帳です。
- Markdown、Git、LLM、DB、WebView、プラグインは不要です。
- 自動保存は入力をブロックしないこと。
- rawファイルは普通のUTF-8 .txtで保持してください。
- 保存失敗を握りつぶさないでください。
- 本文をログに出さないでください。
- configでentry typeのlabel/key/templateを変更可能にしてください。
- template初期値はすべて空です。
- Primary+Kのフロートpickerを中心操作にしてください。
- Primary+Shift+Cで今日の全文をクリップボードへコピーしてください。
- 依存crateは必要最小限にしてください。

各Phase終了時に fmt / clippy / test、実装内容、仕様との差分、性能値を報告してください。
```

---

## 26. 技術リファレンス（2026-09-30時点）

- Iced 0.14 `TextEditor`: https://docs.rs/iced/latest/iced/widget/text_editor/
- Iced `TextEditor` bindings: https://docs.rs/iced/latest/iced/widget/text_editor/enum.Binding.html
- atomic-write-file: https://docs.rs/atomic-write-file/latest/atomic_write_file/

これらは実装候補の確認用であり、特定crateへの恒久的な固定を意味しない。性能・IME・保守性を満たさない場合は再評価する。
