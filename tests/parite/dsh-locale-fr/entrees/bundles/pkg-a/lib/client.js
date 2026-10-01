const NS = "chat";
const base = { "send": "Send" };
const en = { "title": "Chat \"title\"", "hello": "Héllo\nworld", plain: "Plain" };
// locale.register("ignored", "en", { "x": "y" });
locale.register(NS, { en });
locale.register("settings.models", "en", { "ref": base["send"], "own": "Own" });
const TABLE = [["en", { "t1": "One", "t2": "Two" }], ["zh", { "t1": "一" }]];
for (const [locale, dict] of TABLE) { locale.register("iter.ns", locale, dict); }
