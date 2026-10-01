//! `dfr harvest-cjk` — lists every maximal run of ideographs found inside a string literal of
//! the given bundles (former `tools/harvest-hardcoded-cjk.py`). Read only.

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::BTreeSet;
use std::path::Path;

/// Contents of the string literals of `text`, comments excluded.
pub fn litteraux(text: &str) -> Vec<String> {
    let c: Vec<char> = text.chars().collect();
    let n = c.len();
    let mut sortie = Vec::new();
    let mut i = 0;
    while i < n {
        let ch = c[i];
        if ch == '/' && i + 1 < n && c[i + 1] == '/' {
            i = c[i..]
                .iter()
                .position(|&x| x == '\n')
                .map_or(n, |k| i + k + 1);
            continue;
        }
        if ch == '/' && i + 1 < n && c[i + 1] == '*' {
            i = (i + 2..n.saturating_sub(1))
                .find(|&k| c[k] == '*' && c[k + 1] == '/')
                .map_or(n, |k| k + 2);
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            let (mut j, mut echappe, mut corps) = (i + 1, false, String::new());
            while j < n {
                let d = c[j];
                if echappe {
                    echappe = false;
                } else if d == '\\' {
                    echappe = true;
                } else if d == ch {
                    break;
                } else {
                    corps.push(d);
                }
                j += 1;
            }
            sortie.push(corps);
            i = j + 1;
            continue;
        }
        i += 1;
    }
    sortie
}

/// Replaces `\uXXXX` escapes by the character (unpaired surrogates are left untouched).
pub fn decoder(brut: &str) -> String {
    #[allow(clippy::unwrap_used)]
    let re = Regex::new(r"\\u([0-9a-fA-F]{4})").unwrap();
    re.replace_all(brut, |m: &regex::Captures| {
        let code = u32::from_str_radix(&m[1], 16).unwrap_or(0xFFFD);
        char::from_u32(code).map_or_else(|| m[0].to_string(), |c| c.to_string())
    })
    .into_owned()
}

/// JSON `{ run: run }`, longest runs first, as the Python tool printed it. Second value: count.
pub fn executer(fichiers: &[&Path]) -> Result<(String, usize)> {
    #[allow(clippy::unwrap_used)]
    let run = Regex::new("[\u{3400}-\u{9fff}\u{f900}-\u{faff}]+").unwrap();
    let mut trouves = BTreeSet::new();
    for f in fichiers {
        let brut =
            String::from_utf8_lossy(&std::fs::read(f).with_context(|| f.display().to_string())?)
                .into_owned();
        for corps in litteraux(&decoder(&brut)) {
            trouves.extend(run.find_iter(&corps).map(|m| m.as_str().to_string()));
        }
    }
    let mut ordonne: Vec<String> = trouves.into_iter().collect();
    // Python: sorted(set, key=len, reverse=True) is stable over an arbitrary set order; we fix it
    // as length descending then code point ascending.
    ordonne.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()).then(a.cmp(b)));
    let mut texte = String::from("{");
    for (k, r) in ordonne.iter().enumerate() {
        let j = serde_json::to_string(r)?;
        texte.push_str(&format!("{}\n  {}: {}", if k > 0 { "," } else { "" }, j, j));
    }
    texte.push_str(if ordonne.is_empty() { "}" } else { "\n}" });
    Ok((texte, ordonne.len()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn litteraux_sans_commentaires() {
        let l = litteraux("a = '你好'; // \"注释\"\n/* \"块\" */ b = \"设置\";");
        assert_eq!(l, vec!["你好".to_string(), "设置".to_string()]);
    }

    #[test]
    fn echappes_unicode_decodes() {
        assert_eq!(decoder("\\u8bbe\\u7f6e x"), "设置 x");
    }
}
