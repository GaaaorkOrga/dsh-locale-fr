#!/bin/sh
# Ancien harvest-hardcoded-cjk.py, sortie triée comme celle du nouveau (voir trier-cjk.py).
RACINE=$(cd "$(dirname "$0")/../../../.." && pwd)
python3 "$RACINE/tools/harvest-hardcoded-cjk.py" "$@" | python3 "$RACINE/tests/parite/dsh-locale-fr/outils/trier-cjk.py"
