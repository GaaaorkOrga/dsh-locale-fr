#!/bin/sh
# Lance les cas de parité de dsh-locale-fr (anciens scripts Python contre `dfr`).
# Prérequis : `parite` dans le PATH, python3. Lecture seule sur ~/.dsh (copie des client.js du
# programme en service dans entrees/, ignorée par git). Aucun réseau, aucun secret.
# Usage : lancer.sh [chemin du binaire dfr]   (défaut : $CARGO_TARGET_DIR/debug/dfr)
set -u
RACINE=$(cd "$(dirname "$0")/../../.." && pwd)
T=$RACINE/tests/parite/dsh-locale-fr
DFR=${1:-${CARGO_TARGET_DIR:-$RACINE/target}/debug/dfr}
DSH=${DSH_HOME:-$HOME/.dsh}
code=0
# jeux « réels » : dictionnaires du dépôt, client.js des paquets @deepseek-ai installés, deux modules tiers
mkdir -p "$T/entrees/reel/dictionaries" "$T/entrees/reel/lib" "$T/entrees/reel-cjk" "$T/entrees/bundles-reels"
cp "$RACINE/dictionaries/"*.json "$T/entrees/reel/dictionaries/"
for p in "$DSH"/runtime/node_modules/@deepseek-ai/*/; do
  n=$(basename "$p"); [ -f "$p/lib/client.js" ] || continue
  mkdir -p "$T/entrees/bundles-reels/$n/lib" && cp "$p/lib/client.js" "$T/entrees/bundles-reels/$n/lib/client.js"
done
cp "$DSH/profiles/web/node_modules/@welsione/dsh-model-router/lib/client.js" "$T/entrees/reel-cjk/router-client.js" 2>/dev/null || : > "$T/entrees/reel-cjk/router-client.js"
cp "$DSH/profiles/web/node_modules/dsh-model-selector/lib/client/index.js" "$T/entrees/reel-cjk/selector-client.js" 2>/dev/null || : > "$T/entrees/reel-cjk/selector-client.js"
(cd "$T" && parite cli --ancien "sh $T/outils/ancien-build.sh" --nouveau "$DFR build-client" --cas "$T/cas-build.toml") || code=1
(cd "$T" && parite cli --ancien "python3 $RACINE/tools/harvest-dsh-dictionaries.py" --nouveau "$DFR harvest-dictionaries" --cas "$T/cas-dico.toml") || code=1
(cd "$T" && parite cli --ancien "sh $T/outils/ancien-cjk.sh" --nouveau "sh $T/outils/nouveau-cjk.sh $DFR" --cas "$T/cas-cjk.toml") || code=1
exit $code
