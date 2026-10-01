#!/bin/sh
# Nouveau `dfr harvest-cjk` (1er argument : chemin du binaire), sortie triée par le même outil.
RACINE=$(cd "$(dirname "$0")/../../../.." && pwd)
D=$1; shift
"$D" harvest-cjk "$@" | python3 "$RACINE/tests/parite/dsh-locale-fr/outils/trier-cjk.py"
