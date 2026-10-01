//! `dfr harvest-dictionaries` — collects the English dictionaries registered by dsh bundles
//! (`*/lib/client.js`), former `tools/harvest-dsh-dictionaries.py`. Read only.
//! First draft by the Pool (route code-complexe), applied and corrected here.

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

fn regex_statique(cellule: &'static OnceLock<Regex>, motif: &str) -> &'static Regex {
    cellule.get_or_init(|| match Regex::new(motif) {
        Ok(r) => r,
        Err(e) => panic!("motif statique invalide {motif}: {e}"),
    })
}

const CITE: &str = r#"(?:"((?:[^"\\]|\\.)*)"|([A-Za-z_$][\w$]*))"#;

fn re_const_str() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    regex_statique(
        &R,
        r#"(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*"([^"]*)""#,
    )
}
fn re_paire() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    regex_statique(&R, &format!(r#"{CITE}\s*:\s*"((?:[^"\\]|\\.)*)""#))
}
fn re_emprunt() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    regex_statique(
        &R,
        &format!(r#"{CITE}\s*:\s*([A-Za-z_$][\w$]*)\[\s*"((?:[^"\\]|\\.)*)"\s*\]"#),
    )
}
fn re_register() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    regex_statique(&R, r"locale\.register\s*\(")
}
fn re_iteree() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    regex_statique(
        &R,
        r#"(?s)of\s+([A-Za-z_$][\w$]*)\s*\)[^;]{0,200}?locale\.register\s*\(\s*([A-Za-z_$][\w$.]*|"[^"]+")\s*,\s*locale\s*,\s*dict"#,
    )
}
fn re_en_corps() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    regex_statique(&R, r"\ben\b\s*(?::\s*([A-Za-z_$][\w$]*))?")
}
fn re_en_tableau() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    regex_statique(&R, r#"\[\s*"en"\s*,\s*\{"#)
}

/// Pair as captured: (quoted key, bare key, raw value). Exactly one key side is non-empty.
type Paire = (String, String, String);

fn trouver_depuis(texte: &str, aiguille: &str, depuis: usize) -> Option<usize> {
    texte.get(depuis..)?.find(aiguille).map(|i| i + depuis)
}

/// Skips one opaque token (comment, string, template); `None` when `texte[i]` opens none.
fn saute(texte: &str, i: usize) -> Option<usize> {
    let o = texte.as_bytes();
    let c = *o.get(i)?;
    if c == b'/' && i + 1 < o.len() {
        return match o[i + 1] {
            b'/' => Some(trouver_depuis(texte, "\n", i).map_or(o.len(), |j| j + 1)),
            b'*' => Some(trouver_depuis(texte, "*/", i + 2).map_or(o.len(), |j| j + 2)),
            _ => None,
        };
    }
    if c == b'"' || c == b'\'' || c == b'`' {
        let mut echappe = false;
        for (decalage, d) in texte[i + 1..].char_indices() {
            if echappe {
                echappe = false;
            } else if d == '\\' {
                echappe = true;
            } else if d == c as char {
                return Some(i + 1 + decalage + 1);
            }
        }
        return Some(o.len());
    }
    None
}

/// Next index after the character starting at `i`.
fn suivant(texte: &str, i: usize) -> usize {
    i + texte[i..].chars().next().map_or(1, char::len_utf8)
}

/// Index of the `}` matching the `{` at `debut`, or `None`.
fn fin_litteral(texte: &str, debut: usize) -> Option<usize> {
    let (mut profondeur, mut i) = (0i64, debut);
    while i < texte.len() {
        if let Some(s) = saute(texte, i) {
            i = s;
            continue;
        }
        match texte.as_bytes()[i] {
            b'{' => profondeur += 1,
            b'}' => {
                profondeur -= 1;
                if profondeur == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i = suivant(texte, i);
    }
    None
}

fn litteral_nomme(texte: &str, nom: &str, position: Option<usize>) -> Option<String> {
    let re = Regex::new(&format!(
        r"(?:const|let|var)\s+{}\s*=\s*\{{",
        regex::escape(nom)
    ))
    .ok()?;
    let trouves: Vec<_> = re.find_iter(texte).collect();
    let m = match position {
        None => trouves.first()?,
        Some(p) => trouves
            .iter()
            .rfind(|x| x.start() < p)
            .or(trouves.first())?,
    };
    let ouvrante = m.end() - 1;
    let fin = fin_litteral(texte, ouvrante)?;
    (fin > 0).then(|| texte[ouvrante..=fin].to_string())
}

/// Arguments of the call whose `(` is at `ouvrante`.
fn args_de(texte: &str, ouvrante: usize) -> Option<Vec<String>> {
    let (mut profondeur, mut i, mut debut) = (0i64, ouvrante, ouvrante + 1);
    let mut args = Vec::new();
    while i < texte.len() {
        if let Some(s) = saute(texte, i) {
            i = s;
            continue;
        }
        match texte.as_bytes()[i] {
            b'(' | b'[' | b'{' => profondeur += 1,
            b')' | b']' | b'}' => {
                profondeur -= 1;
                if profondeur == 0 {
                    args.push(texte[debut..i].to_string());
                    return Some(args.iter().map(|a| a.trim().to_string()).collect());
                }
            }
            b',' if profondeur == 1 => {
                args.push(texte[debut..i].to_string());
                debut = i + 1;
            }
            _ => {}
        }
        i = suivant(texte, i);
    }
    None
}

fn resoudre_ns(brut: &str, constantes: &BTreeMap<String, String>) -> Option<String> {
    let brut = brut.trim();
    match brut.chars().next() {
        None => None,
        Some('"' | '\'') => {
            let mut c = brut.chars();
            c.next();
            c.next_back();
            Some(c.as_str().to_string())
        }
        Some(_) => constantes
            .get(brut.rsplit('.').next().unwrap_or(brut))
            .cloned(),
    }
}

fn paires_de(texte: &str) -> Vec<Paire> {
    re_paire()
        .captures_iter(texte)
        .map(|c| {
            let g = |i: usize| c.get(i).map_or(String::new(), |m| m.as_str().to_string());
            (g(1), g(2), g(3))
        })
        .collect()
}

/// `json.loads('"' + s + '"')`.
fn decoder_chaine(s: &str) -> Result<String> {
    serde_json::from_str::<String>(&format!("\"{s}\""))
        .with_context(|| format!("invalid JSON escape: {s}"))
}

fn decoder(paires: &[Paire]) -> Result<Map<String, Value>> {
    let mut m = Map::new();
    for (cite, nu, val) in paires {
        let cle = decoder_chaine(if cite.is_empty() { nu } else { cite })?;
        m.insert(cle, Value::String(decoder_chaine(val)?));
    }
    Ok(m)
}

fn resoudre_dico(arg: &str, texte: &str, position: Option<usize>) -> Result<Vec<Paire>> {
    let arg = arg.trim();
    if arg.starts_with('{') {
        let mut paires = paires_de(arg);
        for c in re_emprunt().captures_iter(arg) {
            let g = |i: usize| c.get(i).map_or(String::new(), |m| m.as_str().to_string());
            let Some(bloc) = litteral_nomme(texte, &g(3), position) else {
                continue;
            };
            let valeurs = decoder(&paires_de(&bloc))?;
            if let Some(Value::String(cible)) = valeurs.get(&decoder_chaine(&g(4))?) {
                let echappe = cible.replace('\\', "\\\\").replace('"', "\\\"");
                paires.push((g(1), g(2), echappe));
            }
        }
        return Ok(paires);
    }
    Ok(litteral_nomme(texte, arg, position).map_or_else(Vec::new, |b| paires_de(&b)))
}

fn fusionner(dicos: &mut Map<String, Value>, ns: String, paires: &[Paire]) -> Result<()> {
    let nouveau = decoder(paires)?;
    if let Value::Object(m) = dicos.entry(ns).or_insert_with(|| Value::Object(Map::new())) {
        m.extend(nouveau);
    }
    Ok(())
}

/// Harvests the dictionaries of every bundle text, in insertion order.
pub fn recolter(textes: &[String]) -> Result<Map<String, Value>> {
    let mut dicos = Map::new();
    for texte in textes {
        let constantes: BTreeMap<String, String> = re_const_str()
            .captures_iter(texte)
            .map(|c| (c[1].to_string(), c[2].to_string()))
            .collect();

        for m in re_register().find_iter(texte) {
            let Some(args) = args_de(texte, m.end() - 1) else {
                continue;
            };
            if args.len() < 2 {
                continue;
            }
            let Some(ns) = resoudre_ns(&args[0], &constantes).filter(|n| !n.is_empty()) else {
                continue;
            };
            let paires = if args.len() >= 3 && args[1].trim().trim_matches(['"', '\'']) == "en" {
                resoudre_dico(&args[2], texte, Some(m.start()))?
            } else if args.len() == 2 && args[1].starts_with('{') {
                let Some(c) = re_en_corps().captures(&args[1]) else {
                    continue;
                };
                let nom = c.get(1).map_or("en", |x| x.as_str());
                resoudre_dico(nom, texte, Some(m.start()))?
            } else {
                continue;
            };
            if !paires.is_empty() {
                fusionner(&mut dicos, ns, &paires)?;
            }
        }

        for c in re_iteree().captures_iter(texte) {
            let Some(ns) = resoudre_ns(&c[2], &constantes).filter(|n| !n.is_empty()) else {
                continue;
            };
            let Ok(re_tab) = Regex::new(&format!(
                r"(?:const|let|var)\s+{}\s*=\s*\[",
                regex::escape(&c[1])
            )) else {
                continue;
            };
            let Some(m_tab) = re_tab.find(texte) else {
                continue;
            };
            let Some(m_en) = re_en_tableau().find(&texte[m_tab.end()..]) else {
                continue;
            };
            let depart = m_tab.end() + m_en.end() - 1;
            if let Some(fin) = fin_litteral(texte, depart).filter(|f| *f > 0) {
                let paires = paires_de(&texte[depart..=fin]);
                if !paires.is_empty() {
                    fusionner(&mut dicos, ns, &paires)?;
                }
            }
        }
    }
    Ok(dicos)
}

/// Reads `racine/*/lib/client.js` (sorted), returns the JSON (indent 2) and the summary.
pub fn executer(racine: &Path) -> Result<(String, String)> {
    let mut dossiers: Vec<_> = std::fs::read_dir(racine)
        .with_context(|| racine.display().to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.join("lib/client.js").is_file())
        .collect();
    dossiers.sort();
    let mut textes = Vec::new();
    for d in &dossiers {
        textes.push(String::from_utf8_lossy(&std::fs::read(d.join("lib/client.js"))?).into_owned());
    }
    let dicos = recolter(&textes)?;
    let compte = |v: &Value| v.as_object().map_or(0, Map::len);
    let total: usize = dicos.values().map(compte).sum();
    let mut liste: Vec<(usize, &String)> = dicos.iter().map(|(k, v)| (compte(v), k)).collect();
    liste.sort_by_key(|(n, _)| std::cmp::Reverse(*n));
    let mut resume = format!("{} namespaces, {} keys\n", dicos.len(), total);
    for (n, ns) in liste {
        resume.push_str(&format!("  {n:4}  {ns}\n"));
    }
    Ok((serde_json::to_string_pretty(&Value::Object(dicos))?, resume))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn lire(src: &str, ns: &str, cle: &str) -> Option<String> {
        let d = recolter(&[src.to_string()]).unwrap();
        d.get(ns)?.get(cle)?.as_str().map(String::from)
    }

    #[test]
    fn trois_arguments() {
        let src =
            r#"const NS = "app"; locale.register(NS, "en", { "hello": "Hello", bye: "Bye" });"#;
        assert_eq!(lire(src, "app", "hello").as_deref(), Some("Hello"));
        assert_eq!(lire(src, "app", "bye").as_deref(), Some("Bye"));
    }

    #[test]
    fn deux_arguments() {
        let src = r#"const en = { "a": "A" }; locale.register("app", { en });"#;
        assert_eq!(lire(src, "app", "a").as_deref(), Some("A"));
    }

    #[test]
    fn emprunt() {
        let src =
            r#"const base = { "g": "Hello" }; locale.register("app", "en", { "x": base["g"] });"#;
        assert_eq!(lire(src, "app", "x").as_deref(), Some("Hello"));
    }

    #[test]
    fn forme_iteree() {
        let src = r#"const NS = "app"; const tab = [["en", { "x": "X" }]];
            for (const [locale, dict] of tab) { locale.register(NS, locale, dict); }"#;
        assert_eq!(lire(src, "app", "x").as_deref(), Some("X"));
    }

    #[test]
    fn echappements_et_ordre() {
        let src = r#"locale.register("b", "en", { "z": "a\"b\nc", "a": "2" }); locale.register("a", "en", { "y": "3" });"#;
        assert_eq!(lire(src, "b", "z").as_deref(), Some("a\"b\nc"));
        let d = recolter(&[src.to_string()]).unwrap();
        assert_eq!(d.keys().collect::<Vec<_>>(), ["b", "a"]);
        assert_eq!(
            d["b"].as_object().unwrap().keys().collect::<Vec<_>>(),
            ["z", "a"]
        );
    }

    #[test]
    fn texte_non_ascii_ne_coupe_pas_un_caractere() {
        let src = "/* é */ locale.register(\"app\", \"en\", { \"k\": \"é—ü\" }); // 你好";
        assert_eq!(lire(src, "app", "k").as_deref(), Some("é—ü"));
    }
}
