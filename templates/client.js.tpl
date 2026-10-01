window.__ModuleLoader__.load({
	id: "dsh-locale-fr",
	factory: (require) => {
		var module = { exports: {} };
		var exports = module.exports;
		Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });

		/* GENERATED — do not edit by hand.
		   Sources: dictionaries/fr.json, dictionaries/third-party-fr.json
		   Rebuild:  python3 tools/build-client.py */

		/** French dictionaries, one per UI namespace. */
		const DICTIONARIES = __DICTIONARIES__;

		/**
		 * Hard-coded Chinese fragments from community plugins, and their French.
		 *
		 * `@welsione/dsh-model-router` and `dsh-model-selector` write their labels
		 * as literals and never touch `ctx.locale`, so no language pack can reach
		 * them. Failing that, we translate what is DISPLAYED. These are fragments,
		 * not whole sentences: those plugins assemble their labels at runtime.
		 *
		 * Upstream requests filed:
		 *   github.com/welsione/dsh-model-router/issues/1
		 *   github.com/DeepVite/dsh-model-selector/issues/1
		 * Drop this layer as soon as either lands. Empty object = layer disabled.
		 */
		const THIRD_PARTY = __THIRD_PARTY__;

		/** Language definition added to the shared catalog. */
		const LANGUAGE = {
			id: "fr",
			label: "Fran\u00e7ais",
			fallback: "en"
		};

		/** The locale registry is the only service we need. */
		const inject = ["locale"];

		/**
		 * Register the French dictionaries, one namespace at a time.
		 *
		 * Each call is isolated: a namespace missing from this dsh version, or
		 * already holding a French dictionary, must not stop the others.
		 *
		 * @param locale - locale service.
		 * @param disposers - list to push the disposers onto.
		 */
		function registerDictionaries(locale, disposers) {
			for (const ns of Object.keys(DICTIONARIES)) {
				try {
					disposers.push(locale.register(ns, "fr", DICTIONARIES[ns]));
				} catch (error) {
					console.warn(
						"dsh-locale-fr: namespace \"" + ns + "\" not registered",
						error
					);
				}
			}
		}

		/**
		 * Fallback for dsh versions older than `addLanguage` (0.1.1-rc.2 and
		 * before): their locale catalog is frozen to zh/en and `setLocale("fr")`
		 * throws. We then patch the one function that reads a string,
		 * `lookup(namespace, key)`, to consult French first; the original engine
		 * stays the default answer, so an untranslated key still falls back to
		 * English exactly as before.
		 *
		 * French is then ALWAYS active: with no catalog entry, no selector can
		 * switch away from it. Removing the plugin restores the English UI. The
		 * patch disappears on its own as soon as dsh exposes `addLanguage`, and
		 * the language selector takes over.
		 *
		 * @param locale - locale service to patch.
		 * @returns restores the original function.
		 */
		function patchLookup(locale) {
			const original = locale.lookup;
			const patched = function (ns, key) {
				const dict = DICTIONARIES[ns];
				const translated = dict && dict[key];
				if (typeof translated === "string") return translated;
				return original.apply(this, arguments);
			};
			locale.lookup = patched;
			return () => {
				if (locale.lookup === patched) delete locale.lookup;
			};
		}

		/** Ideographs plus full-width punctuation. */
		const IDEOGRAPHS = /[\u3000-\u303f\u3400-\u9fff\uf900-\ufaff\uff00-\uffef]/;

		/** Areas where the text belongs to the user: never touched. */
		const OFF_LIMITS = "input, textarea, pre, code, [contenteditable], [data-message], [data-role], .cm-editor";

		/** Attributes that carry visible text. */
		const ATTRIBUTES = ["title", "aria-label", "placeholder", "alt"];

		/**
		 * Build the pattern matching every known fragment.
		 * Longest first, so a longer fragment wins over one of its substrings.
		 */
		function fragmentPattern() {
			const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
			const keys = Object.keys(THIRD_PARTY).sort((a, b) => b.length - a.length);
			if (keys.length === 0) return null;
			return new RegExp(keys.map(escape).join("|"), "g");
		}

		/**
		 * Translate what plugins without a locale actually display.
		 *
		 * Fragment replacement in ONE pass: the French produced holds no ideograph
		 * any more, so the next pass leaves it alone and the observer converges.
		 * Nothing outside ideographs is touched, and never inside an input, a code
		 * block or the conversation transcript.
		 *
		 * @param pattern - regexp matching the known fragments.
		 * @returns stops observing.
		 */
		function translateDisplay(pattern) {
			const translate = (value) =>
				value.replace(pattern, (fragment) => THIRD_PARTY[fragment]);

			const offLimits = (element) =>
				!element || (element.closest && element.closest(OFF_LIMITS) !== null);

			const sweep = (root) => {
				if (!root) return;
				if (root.nodeType === 3) {
					if (IDEOGRAPHS.test(root.nodeValue) && !offLimits(root.parentElement)) {
						root.nodeValue = translate(root.nodeValue);
					}
					return;
				}
				if (root.nodeType !== 1 || !root.isConnected) return;
				const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
				const pending = [];
				for (let n = walker.nextNode(); n; n = walker.nextNode()) {
					if (IDEOGRAPHS.test(n.nodeValue) && !offLimits(n.parentElement)) {
						pending.push(n);
					}
				}
				for (const n of pending) n.nodeValue = translate(n.nodeValue);

				const elements = [root, ...root.querySelectorAll("*")];
				for (const e of elements) {
					if (offLimits(e)) continue;
					for (const attribute of ATTRIBUTES) {
						const value = e.getAttribute && e.getAttribute(attribute);
						if (value && IDEOGRAPHS.test(value)) {
							e.setAttribute(attribute, translate(value));
						}
					}
				}
			};

			// Only re-sweep what actually changed: while an answer streams in, the
			// document mutates hundreds of times a second, and a full walk on every
			// frame would slow the session down.
			let scheduled = false;
			let queued = new Set();
			const schedule = (node) => {
				if (node) queued.add(node);
				if (scheduled) return;
				scheduled = true;
				requestAnimationFrame(() => {
					scheduled = false;
					const batch = queued;
					queued = new Set();
					for (const n of batch) sweep(n);
				});
			};

			sweep(document.body);
			const observer = new MutationObserver((changes) => {
				for (const c of changes) {
					if (c.type === "childList") {
						for (const n of c.addedNodes) schedule(n);
					} else {
						schedule(c.target);
					}
				}
			});
			observer.observe(document.body, {
				subtree: true,
				childList: true,
				characterData: true,
				attributes: true,
				attributeFilter: ATTRIBUTES
			});
			return () => observer.disconnect();
		}

		/**
		 * Install French, through the official door when it exists.
		 *
		 * @param ctx - client cordis context.
		 * @returns releases everything this plugin installed.
		 */
		function apply(ctx) {
			const disposers = [];
			const locale = ctx.locale;

			registerDictionaries(locale, disposers);

			if (typeof locale.addLanguage === "function") {
				try {
					disposers.push(locale.addLanguage(LANGUAGE));
				} catch (error) {
					console.warn("dsh-locale-fr: language not added to the catalog", error);
				}
			} else if (typeof locale.lookup === "function") {
				disposers.push(patchLookup(locale));
			} else {
				console.warn(
					"dsh-locale-fr: this dsh version exposes neither addLanguage nor lookup; UI left in English"
				);
			}

			const pattern = fragmentPattern();
			if (pattern && typeof MutationObserver === "function" && document.body) {
				disposers.push(translateDisplay(pattern));
			}

			return () => {
				for (const dispose of disposers) dispose();
			};
		}

		exports.apply = apply;
		exports.inject = inject;
		return module.exports;
	}
});
