import test from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { defaultSettings, prefPages } = require("../settings-schema.js");

const fields = prefPages.flatMap(page => page.fields.map(field => ({
  group: page.group,
  page: page.name,
  section: page.section,
  type: field[0],
  label: field[1],
  key: field[2],
  descriptor: field,
})));

test("shared does not duplicate Finder-owned settings", () => {
  const finder = new Set(Object.keys(defaultSettings.finder));
  for (const key of Object.keys(defaultSettings.shared)) {
    assert.ok(!finder.has(key), `${key} is duplicated in shared and finder`);
  }
});

test("every preference descriptor has exactly one default", () => {
  const seen = new Set();
  for (const field of fields) {
    const key = `${field.section}.${field.key}`;
    assert.ok(Object.hasOwn(defaultSettings[field.section] ?? {}, field.key), `${key} has no default`);
    assert.ok(!seen.has(key), `${key} has multiple descriptors`);
    seen.add(key);
  }
  for (const [section, values] of Object.entries(defaultSettings)) {
    for (const key of Object.keys(values)) assert.ok(seen.has(`${section}.${key}`), `${section}.${key} has no descriptor`);
  }
});

test("defaults satisfy their range, select, and colour descriptors", () => {
  for (const field of fields) {
    const value = defaultSettings[field.section][field.key];
    const [, label, key, ...args] = field.descriptor;
    assert.ok(label.trim(), `${field.section}.${key} label is empty`);
    if (field.type === "range") {
      const [min, max, step] = args;
      assert.equal(typeof value, "number", `${field.section}.${key} range default is numeric`);
      assert.ok(min <= value && value <= max, `${field.section}.${key} default outside range`);
      assert.ok(step > 0, `${field.section}.${key} step must be positive`);
    } else if (field.type === "select") {
      const [options] = args;
      assert.ok(options.length > 0, `${field.section}.${key} has no options`);
      for (const option of options) assert.equal(option, option.trim(), `${field.section}.${key} option has whitespace`);
      assert.ok(options.includes(value), `${field.section}.${key} default is not an option`);
    } else if (field.type === "color") {
      assert.match(value, /^#[0-9a-f]{6}$/i, `${field.section}.${key} default is not #RRGGBB`);
    } else if (field.type === "toggle") {
      assert.equal(typeof value, "boolean", `${field.section}.${key} toggle default is not boolean`);
    } else if (field.type === "text") {
      assert.equal(typeof value, "string", `${field.section}.${key} text default is not a string`);
    } else {
      assert.fail(`${field.section}.${key} has unsupported descriptor type ${field.type}`);
    }
  }
});

test("preference page identities are unique", () => {
  const ids = prefPages.map(page => `${page.group}/${page.name}`);
  assert.equal(new Set(ids).size, ids.length);
});
