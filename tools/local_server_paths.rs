//! ローカル検証サーバーの静的ファイル解決。バイナリ本体とテストの両方から使う。

use std::path::{Path, PathBuf};

/// パーセントエンコーディングを戻す。壊れたエスケープはそのまま残す。
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// URL パスを `dist` 配下の実ファイルに解決する。
/// `dist` の外へ出るパスは `None` (デコードしてから検査する)。
pub fn resolve(dist: &Path, url_path: &str) -> Option<PathBuf> {
    let rel = url_path.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    let decoded = percent_decode(rel);
    if decoded.split('/').any(|seg| seg == ".." || seg.is_empty()) {
        return None;
    }
    let p = dist.join(&decoded);
    Some(if p.is_dir() { p.join("index.html") } else { p })
}
