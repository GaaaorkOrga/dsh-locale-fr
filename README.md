# dsh-locale-fr — French for the DeepSeek Harness web UI

A language pack for [DeepSeek Harness](https://www.npmjs.com/package/@deepseek-ai/dsh) (`dsh`).
It puts the browser UI in **French**: sidebar, composer, command menu, execution trail,
settings and every section of them — **1163 strings across 33 namespaces**.

It ships a second, separable layer that translates two community plugins which hard-code
their Chinese labels (see [Third-party layer](#third-party-layer)).

> Screenshots and status: this pack is used daily against `@deepseek-ai/dsh` **0.1.1-rc.2**
> and is designed to keep working on 0.1.2+.

## Why this exists

dsh ships `zh` and `en` only. That is a shipping choice, **not an architectural limit**:
`@deepseek-ai/dsh-client-locale` exposes a real extension point.

- **dsh 0.1.2-rc.1 and later** — `ctx.locale.addLanguage({ id, label, fallback })` accepts any
  BCP 47-ish tag whose fallback chain terminates at `en`. The pack registers `fr`, and
  **Français shows up in Settings → General → Language** with no UI change at all. The choice
  persists in `~/.dsh/settings.yaml` as `locale.preference: fr`.
- **dsh 0.1.1-rc.2 and earlier** — the catalog is frozen to `zh`/`en` and `setLocale("fr")`
  throws. The pack then patches the single function that reads a string,
  `lookup(namespace, key)`, to consult French first. Same result, one caveat: French is
  **always** active, since there is no catalog entry to switch away from. Remove the plugin to
  get English back.

The pack detects which case it is in. Nothing to configure.

An untranslated key **always falls back to English**, never to Chinese.

## Install

```bash
# from a checkout
dsh plugin --profile web add "link:/path/to/dsh-locale-fr"

# or straight from git
dsh plugin --profile web add "github:GaaaorkOrga/dsh-locale-fr"
```

Web profile only — it is a browser-side plugin, and it mounts itself through its own
`cordis.patch.yml`, so `headless` and `tui` profiles never reference it and start without it.

On dsh 0.1.2+, pick **Français** in Settings → General → Language. Or set it directly:

```yaml
# ~/.dsh/settings.yaml
locale:
  preference: fr
```

**No file under `node_modules` is ever modified.** A dsh update cannot wipe the translation.

## How it is built

`dictionaries/fr.json` is the single source of truth. `lib/client.js` is **generated** from it —
never edit the generated file:

```bash
python3 tools/build-client.py
```

### Adding or fixing a string

Edit `dictionaries/fr.json`, rebuild, reload the page. Keys mirror the namespaces dsh
registers (`conversation`, `chat`, `trajectory`, `settings.models`, …).

Placeholders in braces — `{count}`, `{toolName}`, `{command}` — are part of the contract and
must be copied verbatim, spelling included.

### Refreshing after a dsh upgrade

A new dsh release adds strings. They show in English until you harvest and translate them:

```bash
# 1. Harvest the English dictionaries from the dsh you actually RUN
python3 tools/harvest-dsh-dictionaries.py \
  ~/.npm/_npx/<hash>/node_modules/@deepseek-ai  english.json

# 2. Diff against dictionaries/fr.json, translate the delta however you like,
#    merge it back into dictionaries/fr.json
# 3. Rebuild
python3 tools/build-client.py
```

Harvest from the version you **run**, not from whatever happens to sit in a cache — namespaces
and keys move between releases. That mistake cost a debugging round here.

## Porting to another language

Nothing in the machinery is French-specific:

1. Copy `dictionaries/fr.json` to `dictionaries/<lang>.json` and translate the values.
2. In `tools/build-client.py`, change `LANGUAGE` (`id`, `label` in its own language, `fallback`)
   and the `"fr"` passed to `locale.register`.
3. Rebuild, install, done.

Pull requests adding a language are welcome — say so in the issue first so we can agree on
whether it lives here or in a sibling package.

## Third-party layer

`@welsione/dsh-model-router` (0.0.8) and `dsh-model-selector` (0.5.0) never call `ctx.locale`:
every label is a string literal. **No language pack, in any language, can reach them** — it is
an integration gap, not a translation gap. The router's whole settings page stays Chinese, and
the model chip's label is visible on every screen.

`dictionaries/third-party-fr.json` holds **575 Chinese fragments** and their French, applied by a
`MutationObserver` on what is displayed. Fragments, not sentences: those plugins assemble their
labels at runtime.

Guard rails:

- nothing outside ideographs is touched;
- never inside an input, a code block or the conversation transcript;
- only fragments **known to those two plugins** are replaced, so arbitrary text — including
  Chinese a user pasted — is left alone by construction;
- the French produced holds no ideograph, so the observer converges;
- only mutated nodes are re-swept, batched per animation frame. A full-document walk on every
  frame was measured against 720 mutations in 2 s while an answer streams in.

**This layer is a stopgap.** Upstream requests are filed —
[welsione/dsh-model-router#1](https://github.com/welsione/dsh-model-router/issues/1),
[DeepVite/dsh-model-selector#1](https://github.com/DeepVite/dsh-model-selector/issues/1) — and
it should be deleted as soon as either lands. To disable it now, empty the file:

```bash
echo '{}' > dictionaries/third-party-fr.json && python3 tools/build-client.py
```

To regenerate it for a newer plugin version:

```bash
python3 tools/harvest-hardcoded-cjk.py \
  ~/.dsh/profiles/web/node_modules/@welsione/dsh-model-router/lib/client.js \
  ~/.dsh/profiles/web/node_modules/dsh-model-selector/lib/client/index.js > runs.json
```

## Known gaps

Out of reach of any language pack, because they do not go through the locale registry:

- **Slash-command descriptions** (`/compact`, `/export`, …) — served by the host.
- **Access-mode names** (Read Only / Workspace Write / Full access) — rendered from host
  identifiers.

## A word about dsh versions

`latest` moved from `0.1.1-rc.2` to `0.1.2-rc.1` on 2026-09-04. That release drops
`settingsNamespace` and `installSettingsSection` from `@deepseek-ai/dsh-settings`, which
`dsh-model-selector` and `@welsione/dsh-model-router` still import. The plugin loader
aggregates the failures and **dsh refuses to start** — it does not merely skip the two plugins.
Unrelated to this pack, but you will meet it if you run those plugins; pin your dsh version
until they catch up.

## Contributing

Issues and pull requests welcome — translation fixes, other languages, or taking over
maintenance. Code comments are in English; the dictionaries are plain JSON, no build chain
beyond Python 3.

## License

MIT — see [LICENSE](LICENSE).

---

## En français

Ce paquet met l'interface web de DeepSeek Harness en français : 1163 étiquettes réparties en
33 familles de textes. dsh ne livre que le chinois et l'anglais, mais son registre de langues
prévoit officiellement des langues supplémentaires — ce paquet s'y branche, sans modifier un
seul fichier livré par l'éditeur.

- **Installation** : `dsh plugin --profile web add "github:GaaaorkOrga/dsh-locale-fr"`, puis
  choisir **Français** dans Réglages → Général → Langue (dsh 0.1.2+). Sur une version
  antérieure, le français s'applique tout seul.
- **Corriger un mot** : éditer `dictionaries/fr.json`, relancer `python3 tools/build-client.py`,
  recharger la page. `lib/client.js` est engendré — ne jamais l'éditer à la main.
- **Une étiquette non traduite** retombe en anglais, jamais en chinois.
- Une seconde couche traduit l'affichage de deux modules communautaires qui écrivent leurs
  libellés chinois en dur, hors du système de langues. Elle est provisoire : deux demandes ont
  été déposées auprès de leurs auteurs.

Contributions bienvenues, y compris pour d'autres langues ou pour reprendre la maintenance.
