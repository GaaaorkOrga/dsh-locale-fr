# Quarantaine du 2026-10-25 (chantier RUST17)

Décision de Benoit : « mise à l'écart maintenant ».

## Mis en quarantaine
Rien : les trois anciens outils Python sont les oracles des bancs de parité (règle des oracles).
Dossier de quarantaine : `/Users/benoit/benoit-claude/QUARANTAINE/2026-10-25/dsh-locale-fr/_POURQUOI.md`.

## Déplacés en référence (`tests/reference-ancien/`)
- `tools/build-client.py` (remplacé par `dfr build-client`)
- `tools/harvest-dsh-dictionaries.py` (remplacé par `dfr harvest-dictionaries`)
- `tools/harvest-hardcoded-cjk.py` (remplacé par `dfr harvest-cjk`)
Chemins mis à jour dans `tests/parite/dsh-locale-fr/lancer.sh`, `outils/ancien-build.sh`, `outils/ancien-cjk.sh`, `deploiement.toml` et les commentaires des `cas-*.toml`.

## Laissé en place
- `lib/client.js` (engendré par `dfr`) et `lib/index.js` (coquille) : chargés par le harnais en service. Le commentaire « Rebuild: python3 tools/build-client.py » de `lib/client.js` et de `templates/client.js.tpl` est conservé : le fichier engendré doit rester identique à l'octet. La commande actuelle est `dfr build-client`.
