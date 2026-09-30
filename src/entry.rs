//! 作業種別とテンプレート展開。

use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryType {
    pub id: String,
    pub label: String,
    pub key: String,
    #[serde(default)]
    pub template: String,
}

impl EntryType {
    /// 比較用に小文字化したアクセラレータキー。
    pub fn key_char(&self) -> Option<char> {
        let mut chars = self.key.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => c.to_lowercase().next(),
            _ => None,
        }
    }
}

/// `{time}` `{date}` `{label}` だけを1パスで置換する。未知の `{...}` はそのまま残す。
pub fn expand(template: &str, now: NaiveDateTime, label: &str) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];

        let replaced = [
            ("{time}", now.format("%H:%M").to_string()),
            ("{date}", now.format("%Y-%m-%d").to_string()),
            ("{label}", label.to_owned()),
        ]
        .into_iter()
        .find(|(name, _)| tail.starts_with(name));

        match replaced {
            Some((name, value)) => {
                out.push_str(&value);
                rest = &tail[name.len()..];
            }
            None => {
                out.push('{');
                rest = &tail[1..];
            }
        }
    }

    out.push_str(rest);
    out
}

/// 見出し行と改行、テンプレートを連結したエントリ本文。
pub fn entry_text(header: &str, entry: &EntryType, now: NaiveDateTime) -> String {
    let mut text = expand(header, now, &entry.label);
    text.push('\n');
    text.push_str(&expand(&entry.template, now, &entry.label));
    text
}

/// カーソル行の前後にある文字と連結しないよう、挿入文字列を組み立てる。
///
/// 戻り値は `(挿入する文字列, 挿入後にカーソルを左へ戻す文字数)`。
pub fn plan_insertion(before_cursor: &str, after_cursor: &str, body: &str) -> (String, usize) {
    let mut text = String::with_capacity(body.len() + 2);
    if !before_cursor.is_empty() {
        text.push('\n');
    }
    text.push_str(body);

    if after_cursor.is_empty() {
        (text, 0)
    } else {
        text.push('\n');
        (text, 1)
    }
}
