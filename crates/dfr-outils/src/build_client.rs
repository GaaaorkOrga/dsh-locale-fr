//! `dfr build-client` — generates `lib/client.js` from `dictionaries/*.json` (former
//! `tools/build-client.py`). The output is byte-identical to the Python one.

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use serde_json::ser::{PrettyFormatter, Serializer};
use std::collections::BTreeMap;
use std::path::Path;

/// Template of the browser module; slots `__DICTIONARIES__` and `__THIRD_PARTY__`.
const TEMPLATE: &str = include_str!("../../../templates/client.js.tpl");
const SLOT: &str = "__DICTIONARIES__";
const SLOT_THIRD_PARTY: &str = "__THIRD_PARTY__";

/// JSON on tabs with sorted keys and non-ASCII kept, re-indented at `level` tabs (Python:
/// `json.dumps(ensure_ascii=False, indent='\t', sort_keys=True)` then `indent()`).
pub fn rendre(valeur: &Value) -> Result<String> {
    let trie = trier(valeur);
    let mut tampon = Vec::new();
    let mut ser = Serializer::with_formatter(&mut tampon, PrettyFormatter::with_indent(b"\t"));
    trie.serialize(&mut ser)?;
    let texte = String::from_utf8(tampon)?;
    Ok(texte.split('\n').collect::<Vec<_>>().join("\n\t\t"))
}

/// Recursively sorts object keys by code point (Python `sort_keys`).
fn trier(valeur: &Value) -> Value {
    match valeur {
        Value::Object(m) => {
            let tri: BTreeMap<&String, Value> = m.iter().map(|(k, v)| (k, trier(v))).collect();
            Value::Object(tri.into_iter().map(|(k, v)| (k.clone(), v)).collect())
        }
        Value::Array(a) => Value::Array(a.iter().map(trier).collect()),
        autre => autre.clone(),
    }
}

fn lire_ou_vide(chemin: &Path) -> Result<Value> {
    if !chemin.exists() {
        return Ok(Value::Object(Default::default()));
    }
    let texte = std::fs::read_to_string(chemin).with_context(|| chemin.display().to_string())?;
    serde_json::from_str(&texte).with_context(|| format!("{} is not valid JSON", chemin.display()))
}

/// Builds the content of `client.js` from the two dictionaries.
pub fn construire(dicos: &Value, tiers: &Value) -> Result<String> {
    Ok(TEMPLATE
        .replace(SLOT_THIRD_PARTY, &rendre(tiers)?)
        .replace(SLOT, &rendre(dicos)?))
}

/// Reads the pack at `racine`, writes `lib/client.js`, returns the summary line.
pub fn executer(racine: &Path) -> Result<String> {
    let dicos = lire_ou_vide(&racine.join("dictionaries/fr.json"))?;
    let tiers = lire_ou_vide(&racine.join("dictionaries/third-party-fr.json"))?;
    std::fs::write(racine.join("lib/client.js"), construire(&dicos, &tiers)?)?;
    let compte = |v: &Value| v.as_object().map_or(0, |m| m.len());
    let total: usize = dicos
        .as_object()
        .map_or(0, |m| m.values().map(compte).sum());
    Ok(format!(
        "client.js generated: {} namespaces, {} strings, {} third-party fragments",
        compte(&dicos),
        total,
        compte(&tiers)
    ))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn rendu_trie_et_indente_par_tabulations() {
        let v: Value = serde_json::from_str(r#"{"b":{"y":"é\"x","x":"1"},"a":"z"}"#).unwrap();
        assert_eq!(
            rendre(&v).unwrap(),
            "{\n\t\t\t\"a\": \"z\",\n\t\t\t\"b\": {\n\t\t\t\t\"x\": \"1\",\n\t\t\t\t\"y\": \"é\\\"x\"\n\t\t\t}\n\t\t}"
        );
    }

    #[test]
    fn objet_vide() {
        assert_eq!(rendre(&Value::Object(Default::default())).unwrap(), "{}");
    }
}
