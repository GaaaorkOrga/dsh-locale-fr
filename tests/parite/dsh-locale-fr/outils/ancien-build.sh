#!/bin/sh
# Ancien build-client.py : il déduit la racine de l'emplacement du script, donc on installe son
# dossier tools/ dans la racine de test (premier argument) avant de le lancer.
RACINE=$(cd "$(dirname "$0")/../../../.." && pwd)
P=$1
mkdir -p "$P/tools" && cp "$RACINE/tools/build-client.py" "$P/tools/" || exit 70
python3 "$P/tools/build-client.py"
rc=$?
rm -f "$P/tools/build-client.py"; rmdir "$P/tools" 2>/dev/null
exit $rc
