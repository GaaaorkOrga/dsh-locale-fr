#!/usr/bin/env python3
"""harvest-hardcoded-cjk.py — collect hard-coded CJK fragments from a bundle.

Some community plugins never call `ctx.locale`: their labels are string
literals, so no language pack can reach them. This tool lists every maximal run
of ideographs found inside a string literal (comments excluded), which is the
atom a display-level layer has to be able to replace.

Bundles escape CJK as \\uXXXX, so the file is decoded first.

Output: JSON  { "<chinese run>": "<chinese run>" } — fill in the right-hand
side with your translation, then feed it to tools/build-client.py through
dictionaries/third-party-fr.json.

Usage:
  python3 tools/harvest-hardcoded-cjk.py <client.js> [<client.js> ...] > out.json

Read only: never writes into node_modules.
"""
import json
import re
import sys
from pathlib import Path

ESCAPE = re.compile(r'\\u([0-9a-fA-F]{4})')
RUN = re.compile(r'[㐀-鿿豈-﫿]+')


def literals(text):
    """Yield the contents of string literals, comments excluded."""
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c == '/' and i + 1 < n and text[i + 1] == '/':
            k = text.find('\n', i)
            i = n if k < 0 else k + 1
            continue
        if c == '/' and i + 1 < n and text[i + 1] == '*':
            k = text.find('*/', i + 2)
            i = n if k < 0 else k + 2
            continue
        if c in '"\'`':
            j, escaped, body = i + 1, False, []
            while j < n:
                d = text[j]
                if escaped:
                    escaped = False
                elif d == '\\':
                    escaped = True
                elif d == c:
                    break
                else:
                    body.append(d)
                j += 1
            yield ''.join(body)
            i = j + 1
            continue
        i += 1


def main():
    runs = set()
    for path in sys.argv[1:]:
        raw = Path(path).read_text(encoding='utf-8', errors='replace')
        decoded = ESCAPE.sub(lambda m: chr(int(m.group(1), 16)), raw)
        for body in literals(decoded):
            runs |= {m.group(0) for m in RUN.finditer(body)}

    ordered = sorted(runs, key=len, reverse=True)
    print(json.dumps({r: r for r in ordered}, ensure_ascii=False, indent=2))
    print(f'{len(ordered)} runs', file=sys.stderr)


if __name__ == '__main__':
    main()
