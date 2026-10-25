#!/usr/bin/env python3
"""build-client.py — generates lib/client.js from the JSON dictionaries.

The file served to the browser must be self-contained and wrapped in the dsh
module loader (`window.__ModuleLoader__.load`). We generate it rather than hand
write it: the JSON files stay the single source of truth.

Usage:  python3 tools/build-client.py
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'dictionaries' / 'fr.json'
SOURCE_THIRD_PARTY = ROOT / 'dictionaries' / 'third-party-fr.json'
TARGET = ROOT / 'lib' / 'client.js'
SLOT = '__DICTIONARIES__'
SLOT_THIRD_PARTY = '__THIRD_PARTY__'

TEMPLATE = '''window.__ModuleLoader__.load({
\tid: "dsh-locale-fr",
\tfactory: (require) => {
\t\tvar module = { exports: {} };
\t\tvar exports = module.exports;
\t\tObject.defineProperty(exports, Symbol.toStringTag, { value: "Module" });

\t\t/* GENERATED — do not edit by hand.
\t\t   Sources: dictionaries/fr.json, dictionaries/third-party-fr.json
\t\t   Rebuild:  python3 tools/build-client.py */

\t\t/** French dictionaries, one per UI namespace. */
\t\tconst DICTIONARIES = __DICTIONARIES__;

\t\t/**
\t\t * Hard-coded Chinese fragments from community plugins, and their French.
\t\t *
\t\t * `@welsione/dsh-model-router` and `dsh-model-selector` write their labels
\t\t * as literals and never touch `ctx.locale`, so no language pack can reach
\t\t * them. Failing that, we translate what is DISPLAYED. These are fragments,
\t\t * not whole sentences: those plugins assemble their labels at runtime.
\t\t *
\t\t * Upstream requests filed:
\t\t *   github.com/welsione/dsh-model-router/issues/1
\t\t *   github.com/DeepVite/dsh-model-selector/issues/1
\t\t * Drop this layer as soon as either lands. Empty object = layer disabled.
\t\t */
\t\tconst THIRD_PARTY = __THIRD_PARTY__;

\t\t/** Language definition added to the shared catalog. */
\t\tconst LANGUAGE = {
\t\t\tid: "fr",
\t\t\tlabel: "Fran\\u00e7ais",
\t\t\tfallback: "en"
\t\t};

\t\t/** The locale registry is the only service we need. */
\t\tconst inject = ["locale"];

\t\t/**
\t\t * Register the French dictionaries, one namespace at a time.
\t\t *
\t\t * Each call is isolated: a namespace missing from this dsh version, or
\t\t * already holding a French dictionary, must not stop the others.
\t\t *
\t\t * @param locale - locale service.
\t\t * @param disposers - list to push the disposers onto.
\t\t */
\t\tfunction registerDictionaries(locale, disposers) {
\t\t\tfor (const ns of Object.keys(DICTIONARIES)) {
\t\t\t\ttry {
\t\t\t\t\tdisposers.push(locale.register(ns, "fr", DICTIONARIES[ns]));
\t\t\t\t} catch (error) {
\t\t\t\t\tconsole.warn(
\t\t\t\t\t\t"dsh-locale-fr: namespace \\"" + ns + "\\" not registered",
\t\t\t\t\t\terror
\t\t\t\t\t);
\t\t\t\t}
\t\t\t}
\t\t}

\t\t/**
\t\t * Fallback for dsh versions older than `addLanguage` (0.1.1-rc.2 and
\t\t * before): their locale catalog is frozen to zh/en and `setLocale("fr")`
\t\t * throws. We then patch the one function that reads a string,
\t\t * `lookup(namespace, key)`, to consult French first; the original engine
\t\t * stays the default answer, so an untranslated key still falls back to
\t\t * English exactly as before.
\t\t *
\t\t * French is then ALWAYS active: with no catalog entry, no selector can
\t\t * switch away from it. Removing the plugin restores the English UI. The
\t\t * patch disappears on its own as soon as dsh exposes `addLanguage`, and
\t\t * the language selector takes over.
\t\t *
\t\t * @param locale - locale service to patch.
\t\t * @returns restores the original function.
\t\t */
\t\tfunction patchLookup(locale) {
\t\t\tconst original = locale.lookup;
\t\t\tconst patched = function (ns, key) {
\t\t\t\tconst dict = DICTIONARIES[ns];
\t\t\t\tconst translated = dict && dict[key];
\t\t\t\tif (typeof translated === "string") return translated;
\t\t\t\treturn original.apply(this, arguments);
\t\t\t};
\t\t\tlocale.lookup = patched;
\t\t\treturn () => {
\t\t\t\tif (locale.lookup === patched) delete locale.lookup;
\t\t\t};
\t\t}

\t\t/** Ideographs plus full-width punctuation. */
\t\tconst IDEOGRAPHS = /[\\u3000-\\u303f\\u3400-\\u9fff\\uf900-\\ufaff\\uff00-\\uffef]/;

\t\t/** Areas where the text belongs to the user: never touched. */
\t\tconst OFF_LIMITS = "input, textarea, pre, code, [contenteditable], [data-message], [data-role], .cm-editor";

\t\t/** Attributes that carry visible text. */
\t\tconst ATTRIBUTES = ["title", "aria-label", "placeholder", "alt"];

\t\t/**
\t\t * Build the pattern matching every known fragment.
\t\t * Longest first, so a longer fragment wins over one of its substrings.
\t\t */
\t\tfunction fragmentPattern() {
\t\t\tconst escape = (s) => s.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\\\$&");
\t\t\tconst keys = Object.keys(THIRD_PARTY).sort((a, b) => b.length - a.length);
\t\t\tif (keys.length === 0) return null;
\t\t\treturn new RegExp(keys.map(escape).join("|"), "g");
\t\t}

\t\t/**
\t\t * Translate what plugins without a locale actually display.
\t\t *
\t\t * Fragment replacement in ONE pass: the French produced holds no ideograph
\t\t * any more, so the next pass leaves it alone and the observer converges.
\t\t * Nothing outside ideographs is touched, and never inside an input, a code
\t\t * block or the conversation transcript.
\t\t *
\t\t * @param pattern - regexp matching the known fragments.
\t\t * @returns stops observing.
\t\t */
\t\tfunction translateDisplay(pattern) {
\t\t\tconst translate = (value) =>
\t\t\t\tvalue.replace(pattern, (fragment) => THIRD_PARTY[fragment]);

\t\t\tconst offLimits = (element) =>
\t\t\t\t!element || (element.closest && element.closest(OFF_LIMITS) !== null);

\t\t\tconst sweep = (root) => {
\t\t\t\tif (!root) return;
\t\t\t\tif (root.nodeType === 3) {
\t\t\t\t\tif (IDEOGRAPHS.test(root.nodeValue) && !offLimits(root.parentElement)) {
\t\t\t\t\t\troot.nodeValue = translate(root.nodeValue);
\t\t\t\t\t}
\t\t\t\t\treturn;
\t\t\t\t}
\t\t\t\tif (root.nodeType !== 1 || !root.isConnected) return;
\t\t\t\tconst walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
\t\t\t\tconst pending = [];
\t\t\t\tfor (let n = walker.nextNode(); n; n = walker.nextNode()) {
\t\t\t\t\tif (IDEOGRAPHS.test(n.nodeValue) && !offLimits(n.parentElement)) {
\t\t\t\t\t\tpending.push(n);
\t\t\t\t\t}
\t\t\t\t}
\t\t\t\tfor (const n of pending) n.nodeValue = translate(n.nodeValue);

\t\t\t\tconst elements = [root, ...root.querySelectorAll("*")];
\t\t\t\tfor (const e of elements) {
\t\t\t\t\tif (offLimits(e)) continue;
\t\t\t\t\tfor (const attribute of ATTRIBUTES) {
\t\t\t\t\t\tconst value = e.getAttribute && e.getAttribute(attribute);
\t\t\t\t\t\tif (value && IDEOGRAPHS.test(value)) {
\t\t\t\t\t\t\te.setAttribute(attribute, translate(value));
\t\t\t\t\t\t}
\t\t\t\t\t}
\t\t\t\t}
\t\t\t};

\t\t\t// Only re-sweep what actually changed: while an answer streams in, the
\t\t\t// document mutates hundreds of times a second, and a full walk on every
\t\t\t// frame would slow the session down.
\t\t\tlet scheduled = false;
\t\t\tlet queued = new Set();
\t\t\tconst schedule = (node) => {
\t\t\t\tif (node) queued.add(node);
\t\t\t\tif (scheduled) return;
\t\t\t\tscheduled = true;
\t\t\t\trequestAnimationFrame(() => {
\t\t\t\t\tscheduled = false;
\t\t\t\t\tconst batch = queued;
\t\t\t\t\tqueued = new Set();
\t\t\t\t\tfor (const n of batch) sweep(n);
\t\t\t\t});
\t\t\t};

\t\t\tsweep(document.body);
\t\t\tconst observer = new MutationObserver((changes) => {
\t\t\t\tfor (const c of changes) {
\t\t\t\t\tif (c.type === "childList") {
\t\t\t\t\t\tfor (const n of c.addedNodes) schedule(n);
\t\t\t\t\t} else {
\t\t\t\t\t\tschedule(c.target);
\t\t\t\t\t}
\t\t\t\t}
\t\t\t});
\t\t\tobserver.observe(document.body, {
\t\t\t\tsubtree: true,
\t\t\t\tchildList: true,
\t\t\t\tcharacterData: true,
\t\t\t\tattributes: true,
\t\t\t\tattributeFilter: ATTRIBUTES
\t\t\t});
\t\t\treturn () => observer.disconnect();
\t\t}

\t\t/**
\t\t * Install French, through the official door when it exists.
\t\t *
\t\t * @param ctx - client cordis context.
\t\t * @returns releases everything this plugin installed.
\t\t */
\t\tfunction apply(ctx) {
\t\t\tconst disposers = [];
\t\t\tconst locale = ctx.locale;

\t\t\tregisterDictionaries(locale, disposers);

\t\t\tif (typeof locale.addLanguage === "function") {
\t\t\t\ttry {
\t\t\t\t\tdisposers.push(locale.addLanguage(LANGUAGE));
\t\t\t\t} catch (error) {
\t\t\t\t\tconsole.warn("dsh-locale-fr: language not added to the catalog", error);
\t\t\t\t}
\t\t\t} else if (typeof locale.lookup === "function") {
\t\t\t\tdisposers.push(patchLookup(locale));
\t\t\t} else {
\t\t\t\tconsole.warn(
\t\t\t\t\t"dsh-locale-fr: this dsh version exposes neither addLanguage nor lookup; UI left in English"
\t\t\t\t);
\t\t\t}

\t\t\tconst pattern = fragmentPattern();
\t\t\tif (pattern && typeof MutationObserver === "function" && document.body) {
\t\t\t\tdisposers.push(translateDisplay(pattern));
\t\t\t}

\t\t\treturn () => {
\t\t\t\tfor (const dispose of disposers) dispose();
\t\t\t};
\t\t}

\t\texports.apply = apply;
\t\texports.inject = inject;
\t\treturn module.exports;
\t}
});
'''


def indent(text, level=2):
    """Re-indent a JSON block on tabs, at the requested level."""
    return ('\n' + '\t' * level).join(text.split('\n'))


def main():
    dicts = json.loads(SOURCE.read_text(encoding='utf-8')) if SOURCE.exists() else {}
    third = (
        json.loads(SOURCE_THIRD_PARTY.read_text(encoding='utf-8'))
        if SOURCE_THIRD_PARTY.exists() else {}
    )
    rendered = indent(json.dumps(dicts, ensure_ascii=False, indent='\t', sort_keys=True))
    rendered_third = indent(
        json.dumps(third, ensure_ascii=False, indent='\t', sort_keys=True)
    )
    TARGET.write_text(
        TEMPLATE.replace(SLOT_THIRD_PARTY, rendered_third).replace(SLOT, rendered),
        encoding='utf-8',
    )
    total = sum(len(v) for v in dicts.values())
    print(
        f'{TARGET.name} generated: {len(dicts)} namespaces, {total} strings, '
        f'{len(third)} third-party fragments'
    )


if __name__ == '__main__':
    main()
