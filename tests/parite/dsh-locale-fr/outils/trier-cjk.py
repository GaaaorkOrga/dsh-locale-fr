#!/usr/bin/env python3
"""Tri canonique de la sortie de harvest-cjk (l'ordre des égalités de longueur est arbitraire en
Python) : lit le JSON sur stdin, le réécrit trié par (longueur décroissante, ordre des points de code)."""
import json, sys
d = json.load(sys.stdin)
ordre = sorted(d, key=lambda r: (-len(r), r))
print(json.dumps({r: d[r] for r in ordre}, ensure_ascii=False, indent=2))
