import test from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { defaultSettings, prefPages } = require("../settings-schema.js");

// docs/spec_reconciliation.md: for an existing setting the Rust range and default win and the design changes.
// [row, section, key, min, max, default]
const RANGES = [
  [1, "dock", "iconSize", 32, 72, 48],
  [2, "dock", "iconSpacing", 0, 24, 14],
  [3, "dock", "edgeDistance", 0, 60, 8],
  [4, "island", "rounding", 0, 100, 50],
  [5, "shared", "globalBlur", 0, 100, 60],
  [23, "shared", "stageIconSpacing", 0, 60, 20],
  [24, "shared", "stageIconCount", 2, 10, 6],
  [25, "dock", "iconMagnification", 1, 2.5, 2],
];
const DEFAULTS = [
  [18, "shared", "enableFinder", false],
  [19, "shared", "displayLanguage", "Indonesian"],
  [20, "island", "onlineLyrics", true],
  [21, "island", "sendToPhone", true],
  [27, "shared", "stageClickDesktopHide", true],
  [27, "shared", "stageAutoPauseMedia", false],
  [15, "island", "displayMode", "Fixed"],
  [16, "island", "material", "Black"],
];

const field = (section, key) => prefPages.filter(p => p.section === section).flatMap(p => p.fields).find(f => f[2] === key);

for (const [row, section, key, min, max, def] of RANGES) {
  test(`row ${row}: ${section}.${key} is ${min}–${max}, default ${def}`, () => {
    const f = field(section, key);
    assert.deepEqual([f[3], f[4]], [min, max]);
    assert.equal(defaultSettings[section][key], def);
  });
}

for (const [row, section, key, def] of DEFAULTS) {
  test(`row ${row}: ${section}.${key} defaults to ${def}`, () => assert.equal(defaultSettings[section][key], def));
}

test("row 6: the interface scale reaches 200 %", () => assert.equal(field("shared", "globalScale")[4], 2));

test("row 15: the Island display modes are the three core modes plus On demand", () => {
  assert.deepEqual(field("island", "displayMode")[3], ["Fixed", "Overlay", "Smart", "On demand"]);
});

test("row 16: the Island materials use the core names", () => {
  assert.deepEqual(field("island", "material")[3], ["Blur", "Glass", "Black"]);
});

test("ADR-0007: the Dock has no 'Only display with…' mode", () => {
  assert.ok(!field("dock", "displayMode")[3].includes("onlyWith"));
});
