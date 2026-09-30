# Daily Work Memo

朝から終業まで開きっぱなしにする「今日の1枚」のプレーンテキストメモ帳（macOS / Windows）。

仕様: [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) / 性能測定: [docs/PERFORMANCE.md](docs/PERFORMANCE.md)

## 使い方

起動すると今日の日次ファイルが開き、そのまま入力できます。保存は自動です。

| 操作 | キー（`Primary` = macOS: Cmd / Windows: Ctrl） |
|---|---|
| 作業種別を挿入 | `Primary+K` → キー（`W` `R` `S` `M` `D` `A` `G`）、または `↑↓` + `Enter` / クリック。`Esc` で閉じる |
| 今日の全文をコピー | `Primary+Shift+C` |
| Undo / Redo | `Primary+Z` / macOS: `Cmd+Shift+Z`、Windows: `Ctrl+Y` または `Ctrl+Shift+Z` |
| 即時保存（通常は不要） | `Primary+S` |

日付をまたいで開いたままにした場合は、次にウィンドウへフォーカスした時に今日のファイルへ切り替わります。

## ファイルの場所

| | macOS | Windows |
|---|---|---|
| config | `~/Library/Application Support/daily-work-memo/config.toml` | `%APPDATA%\daily-work-memo\config.toml` |
| 日次メモ | `~/Library/Application Support/daily-work-memo/daily/YYYY-MM-DD.txt` | `%APPDATA%\daily-work-memo\daily\YYYY-MM-DD.txt` |

日次メモは UTF-8（BOMなし）・LF の `.txt` です。config は初回起動時に [config.example.toml](config.example.toml) と同じ内容で生成され、変更は再起動で反映されます。本番用の `config.toml` はコミットしません（`.gitignore` 済み）。
`data_dir` には絶対パスのみ指定できます（`~` は展開しません）。

## 開発

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
./scripts/bundle-macos.sh        # target/release/DailyWorkMemo.app（universal・署名なし）
cargo run --release --example spike -- 10000   # Phase 0 spike
```

## Phase 0 Go / No-Go

**Go（条件付き）** — macOS で起動 0.3 s（warm）、アイドル CPU 0.1〜0.2 %、通常サイズで RSS 約 89 MB（Apple M3 ネイティブ。Rosetta 実行時は約 45 MB）、日本語 IME 入力も動作。
日次テキストが数百 KB を超えると RSS が 150 MB を超える制約あり。Windows 実機での確認は未実施。詳細は [docs/PERFORMANCE.md](docs/PERFORMANCE.md)。

## 仕様との差分

- **Undo/Redo を `editor.rs` で実装**: Iced 0.14 の TextEditor に Undo/Redo が無いため、編集のまとまりごとの全文スナップショット（最大100件・16MB）で実装。
- **`src/saver.rs`・`src/lib.rs` を追加**: 自動保存の debounce と書き込みを専用スレッドで行うため（UI をブロックしない）。`lib.rs` は `tests/` から各モジュールを使うため。
- **macOS の Cmd+Q 対策**: winit の標準メニューの Quit は close request を経ずに終了するため、Cmd キー押下時点で未保存分を書き込みに回し、終了処理（`atexit`）で書き込み完了を最大2秒待つ。保存失敗中・読み込み失敗中に終了した場合は、日次ファイルを上書きせず `<data_dir>/recovery/YYYY-MM-DD-HHMMSS.txt`（書けなければ一時ディレクトリ）へ本文を退避する。このため macOS のみ `libc` に直接依存。
- **全文コピー失敗は通知しない**: Iced のクリップボード書き込みは成否を返さないため検出できない。
- **Esc でエディタのフォーカスを外さない**: Iced 既定では Esc でフォーカスが外れ入力できなくなるため無効化。
- **`font_size` を 8〜72 に制限**、**未知の config キーはエラー**（typo の見落とし防止）。config エラー時も、有効な絶対パスの `data_dir` だけは引き継ぐ（メモの保存先が分かれるのを防ぐ）。
- **貼り付け・IME確定の CR は LF へ正規化**して挿入する。
- **日次ファイル読み込み失敗時は保存を停止**し、未編集ならフォーカス復帰時に読み直す。
- **ログは stderr のみ**（本文は出さない）。Windows の release ビルドはコンソールを持たないためログは見えない。
- **LICENSE は未作成**（ライセンスの選択は未決定）。

## 既知の制約

- 多重起動を防がない。2つ起動すると同じ日次ファイルを交互に上書きする。
- Windows のログオフ・シャットダウン時に close request が来るかは未検証。
- 日次テキストが数百 KB を超えると RSS が 150 MB を超える（[docs/PERFORMANCE.md](docs/PERFORMANCE.md)）。
