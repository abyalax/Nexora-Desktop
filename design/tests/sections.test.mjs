import test from "node:test";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
import assert from "node:assert/strict";
const { prefPages, pageCards } = require("../settings-schema.js");

test("every section start names a field that exists on its page", () => {
  for (const page of prefPages) {
    const keys = new Set(page.fields.map(f => f[2]));
    for (const key of Object.keys(page.starts)) assert.ok(keys.has(key), `${page.group}/${page.name}: unknown start ${key}`);
  }
});

test("cards keep every field once, in order", () => {
  for (const page of prefPages) {
    assert.deepEqual(pageCards(page).flatMap(c => c.fields), page.fields, page.name);
  }
});

test("a page without starts is one card titled by the page name", () => {
  const page = prefPages.find(p => p.name === "Audio");
  assert.deepEqual(pageCards(page).map(c => c.title), ["Audio"]);
});

test("Appearance has one card per spec section, each opening on its toggle", () => {
  const page = prefPages.find(p => p.name === "Appearance");
  const cards = pageCards(page);
  assert.equal(cards.length, 8);
  assert.equal(cards[1].title, "Reflections");
  assert.equal(cards[1].fields[0][2], "pinnedReflections");
});
