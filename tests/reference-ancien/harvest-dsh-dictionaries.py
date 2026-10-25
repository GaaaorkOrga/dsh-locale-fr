#!/usr/bin/env python3
"""harvest-dsh-dictionaries.py — collect the English dictionaries from dsh bundles.

Output: JSON  { "<namespace>": { "<key>": "<english text>" } }
Read only: never writes into node_modules.

Usage:
  python3 tools/harvest-dsh-dictionaries.py <dir-of-@deepseek-ai-packages> <out.json>

The <dir> argument is scanned for */lib/client.js. Two useful sources:
  - the npx cache:  ~/.npm/_npx/<hash>/node_modules/@deepseek-ai
  - a local install: <prefix>/node_modules/@deepseek-ai
Harvest from the version you actually RUN: namespaces and keys move
between dsh releases.
"""
import json
import re
import sys
from pathlib import Path

RACINE = Path(sys.argv[1])
SORTIE = Path(sys.argv[2])

RE_CONST_STR = re.compile(r'(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*"([^"]*)"')
# quoted key or bare identifier; value is always a double-quoted string
RE_PAIRE = re.compile(
    r'(?:"((?:[^"\\]|\\.)*)"|([A-Za-z_$][\w$]*))\s*:\s*"((?:[^"\\]|\\.)*)"'
)


def _saute(texte, i):
    """Skip one opaque token (string, template, comment). Returns the new
    index, or None when text[i] opens none of those."""
    c = texte[i]
    if c == '/' and i + 1 < len(texte):
        if texte[i + 1] == '/':
            j = texte.find('\n', i)
            return len(texte) if j < 0 else j + 1
        if texte[i + 1] == '*':
            j = texte.find('*/', i + 2)
            return len(texte) if j < 0 else j + 2
        return None
    if c in '"\'`':
        j, echap = i + 1, False
        while j < len(texte):
            d = texte[j]
            if echap:
                echap = False
            elif d == '\\':
                echap = True
            elif d == c:
                return j + 1
            j += 1
        return len(texte)
    return None


def fin_litteral(texte, debut):
    """Index of the closing brace matching text[start] == '{'."""
    profondeur, i = 0, debut
    while i < len(texte):
        saut = _saute(texte, i)
        if saut is not None:
            i = saut
            continue
        c = texte[i]
        if c == '{':
            profondeur += 1
        elif c == '}':
            profondeur -= 1
            if profondeur == 0:
                return i
        i += 1
    return -1


def litteral_nomme(texte, nom, position=None):
    """Literal assigned to `name`. With `position`, take the declaration
    closest BEFORE that point (bundles declare the dictionary right above its
    registration), otherwise the first one after."""
    trouves = list(
        re.finditer(r'(?:const|let|var)\s+' + re.escape(nom) + r'\s*=\s*\{', texte)
    )
    if not trouves:
        return None
    if position is None:
        m = trouves[0]
    else:
        avant = [x for x in trouves if x.start() < position]
        m = avant[-1] if avant else trouves[0]
    fin = fin_litteral(texte, m.end() - 1)
    return texte[m.end() - 1:fin + 1] if fin > 0 else None


def args_de(texte, i_ouvrante):
    """Split the arguments of a call whose text[i_open] == '('."""
    profondeur, i = 0, i_ouvrante
    debut_arg, args = i_ouvrante + 1, []
    while i < len(texte):
        saut = _saute(texte, i)
        if saut is not None:
            i = saut
            continue
        c = texte[i]
        if c in '([{':
            profondeur += 1
        elif c in ')]}':
            profondeur -= 1
            if profondeur == 0:
                args.append(texte[debut_arg:i])
                return [a.strip() for a in args], i
        elif c == ',' and profondeur == 1:
            args.append(texte[debut_arg:i])
            debut_arg = i + 1
        i += 1
    return None, -1


def resoudre_ns(brut, constantes):
    brut = brut.strip()
    if brut[:1] in '"\'':
        return brut[1:-1]
    return constantes.get(brut.split('.')[-1])


# value borrowed from another dictionary:  "key": otherDict["key"]
RE_EMPRUNT = re.compile(
    r'(?:"((?:[^"\\]|\\.)*)"|([A-Za-z_$][\w$]*))\s*:\s*'
    r'([A-Za-z_$][\w$]*)\[\s*"((?:[^"\\]|\\.)*)"\s*\]'
)


def resoudre_dico(arg, texte, position=None):
    arg = arg.strip()
    if arg.startswith('{'):
        paires = RE_PAIRE.findall(arg)
        for cite, nu, source, cle_source in RE_EMPRUNT.findall(arg):
            bloc = litteral_nomme(texte, source, position)
            if not bloc:
                continue
            valeurs = decoder(RE_PAIRE.findall(bloc))
            texte_cible = valeurs.get(json.loads('"' + cle_source + '"'))
            if texte_cible is not None:
                paires.append(
                    (cite or nu, '', texte_cible.replace('\\', '\\\\').replace('"', '\\"'))
                )
        return paires
    bloc = litteral_nomme(texte, arg, position)
    return RE_PAIRE.findall(bloc) if bloc else []


def decoder(paires):
    return {
        json.loads('"' + (cite or nu) + '"'): json.loads('"' + val + '"')
        for cite, nu, val in paires
    }


dicos = {}
for fichier in sorted(RACINE.glob('*/lib/client.js')):
    texte = fichier.read_text(encoding='utf-8', errors='replace')
    constantes = dict(RE_CONST_STR.findall(texte))

    for m in re.finditer(r'locale\.register\s*\(', texte):
        args, _ = args_de(texte, m.end() - 1)
        if not args or len(args) < 2:
            continue
        ns = resoudre_ns(args[0], constantes)
        if not ns:
            continue
        if len(args) >= 3 and args[1].strip().strip('"\'') == 'en':
            paires = resoudre_dico(args[2], texte, m.start())
        elif len(args) == 2 and args[1].startswith('{'):
            corps = args[1]
            m_en = re.search(r'\ben\b\s*(?::\s*([A-Za-z_$][\w$]*))?', corps)
            if not m_en:
                continue
            nom = m_en.group(1) or 'en'
            paires = resoudre_dico(nom, texte, m.start())
        else:
            continue
        if paires:
            dicos.setdefault(ns, {}).update(decoder(paires))

    # iterated form: for (const [locale, dict] of <array>) ... register(NS, locale, dict)
    for m in re.finditer(
        r'of\s+([A-Za-z_$][\w$]*)\s*\)[^;]{0,200}?locale\.register\s*\(\s*'
        r'([A-Za-z_$][\w$.]*|"[^"]+")\s*,\s*locale\s*,\s*dict',
        texte,
        re.S,
    ):
        ns = resoudre_ns(m.group(2), constantes)
        if not ns:
            continue
        m_tab = re.search(
            r'(?:const|let|var)\s+' + re.escape(m.group(1)) + r'\s*=\s*\[', texte
        )
        if not m_tab:
            continue
        m_en = re.search(r'\[\s*"en"\s*,\s*\{', texte[m_tab.end():])
        if not m_en:
            continue
        depart = m_tab.end() + m_en.end() - 1
        fin = fin_litteral(texte, depart)
        if fin > 0:
            paires = RE_PAIRE.findall(texte[depart:fin + 1])
            if paires:
                dicos.setdefault(ns, {}).update(decoder(paires))

SORTIE.write_text(json.dumps(dicos, ensure_ascii=False, indent=2), encoding='utf-8')
total = sum(len(v) for v in dicos.values())
print(f'{len(dicos)} namespaces, {total} keys')
for ns in sorted(dicos, key=lambda n: -len(dicos[n])):
    print(f'  {len(dicos[ns]):4d}  {ns}')
