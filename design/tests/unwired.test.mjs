import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { isReferenced, unwiredKeys, newlyUnwired, nowWired, parseBaseline, loadUnwired, PATHS } from "../../scripts/unwired_keys.mjs";

test("a key is referenced by property access, bracket access or quoted name", () => {
  assert.equal(isReferenced("iconSize", "const n = s.iconSize;"), true);
  assert.equal(isReferenced("iconSize", 'const n = s["iconSize"];'), true);
  assert.equal(isReferenced("iconSize", `effects['dock.iconSize']`), true);
  assert.equal(isReferenced("iconSize", 'effects["dock.iconSize"]'), true);
});

test("a longer name or a bare word does not count as a reference", () => {
  assert.equal(isReferenced("size", "s.sizeLimit"), false);
  assert.equal(isReferenced("size", "const size = 1;"), false);
});

test("unwiredKeys reports section.key for settings the source never mentions, sorted", () => {
  const defaults = { dock: { b: 1, a: 2 }, shared: { c: true } };
  assert.deepEqual(unwiredKeys(defaults, "d.a"), ["dock.b", "shared.c"]);
});

test("the ratchet separates new unwired keys from baseline keys that are wired now", () => {
  assert.deepEqual(newlyUnwired(["a.x", "a.y"], ["a.x"]), ["a.y"]);
  assert.deepEqual(nowWired(["a.x"], ["a.x", "a.z"]), ["a.z"]);
  assert.deepEqual(parseBaseline("# note\n\na.x\n b.y \n"), ["a.x", "b.y"]);
});

test("the design has no unwired key outside the baseline (the list can only shrink)", () => {
  const baseline = parseBaseline(fs.readFileSync(PATHS.baseline, "utf8"));
  assert.deepEqual(newlyUnwired(loadUnwired(), baseline), []);
});
