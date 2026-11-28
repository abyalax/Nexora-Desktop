#!/usr/bin/env node
// HTM-04: list design settings that the HTML never reads.
//
//   node scripts/unwired_keys.mjs [--update-baseline]
//
// A key counts as wired when the page source mentions its name as a property (`.key`, `["key"]`) or as a quoted
// string (an `effects` row, a `data-` lookup). This measures "is the key referenced at all", not "does it behave
// correctly"; it catches settings that are only declared in the schema.
//
// The ratchet: design/unwired-baseline.txt holds the keys that are known to be unwired. A key that is unwired and
// not in the baseline fails the run (exit 1), so the list can only shrink. Wire a key, then run with
// --update-baseline to drop it from the file.
//
// Exit codes: 0 = no new unwired keys, 1 = new unwired keys, 2 = an input could not be read.

import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const PATHS = {
  schema: path.join(ROOT, "design/settings-schema.js"),
  sources: [path.join(ROOT, "design/nexora-v1.html"), path.join(ROOT, "design/island-activities.js")],
  baseline: path.join(ROOT, "design/unwired-baseline.txt"),
};

const escapeRegExp = text => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

// True when `source` mentions `key` as a property access or a quoted name.
export function isReferenced(key, source) {
  return new RegExp(`(?:\\.|\\["|'|")${escapeRegExp(key)}\\b`).test(source);
}

// Sorted "section.key" names of every default setting that `source` never mentions.
export function unwiredKeys(defaultSettings, source) {
  const out = [];
  for (const [section, values] of Object.entries(defaultSettings)) {
    for (const key of Object.keys(values)) if (!isReferenced(key, source)) out.push(`${section}.${key}`);
  }
  return out.sort();
}

export function parseBaseline(text) {
  return text.split(/\r?\n/).map(l => l.trim()).filter(l => l && !l.startsWith("#"));
}

// Keys that are unwired now but not in the baseline: these fail the ratchet.
export function newlyUnwired(unwired, baseline) {
  const known = new Set(baseline);
  return unwired.filter(k => !known.has(k));
}

// Baseline entries that are wired now: the baseline can be tightened.
export function nowWired(unwired, baseline) {
  const current = new Set(unwired);
  return baseline.filter(k => !current.has(k));
}

export function loadUnwired(paths = PATHS) {
  const { defaultSettings } = createRequire(import.meta.url)(paths.schema);
  const source = paths.sources.map(p => fs.readFileSync(p, "utf8")).join("\n");
  return unwiredKeys(defaultSettings, source);
}

export function main(argv = process.argv.slice(2)) {
  let unwired;
  try {
    unwired = loadUnwired();
  } catch (err) {
    console.error(`unwired_keys: ${err.message}`);
    return 2;
  }
  if (argv.includes("--update-baseline")) {
    const header = "# Design keys the HTML never reads (scripts/unwired_keys.mjs). Remove a line when its key is wired.\n";
    fs.writeFileSync(PATHS.baseline, header + unwired.join("\n") + (unwired.length ? "\n" : ""));
    console.log(`baseline written: ${unwired.length} keys`);
    return 0;
  }
  const baseline = fs.existsSync(PATHS.baseline) ? parseBaseline(fs.readFileSync(PATHS.baseline, "utf8")) : [];
  const added = newlyUnwired(unwired, baseline);
  const wired = nowWired(unwired, baseline);
  console.log(`unwired keys: ${unwired.length} (baseline ${baseline.length})`);
  for (const k of unwired) console.log(`  ${k}`);
  if (wired.length) console.log(`\nWired since the baseline (run --update-baseline): ${wired.join(", ")}`);
  if (added.length) {
    console.error(`\nNew unwired keys: ${added.join(", ")}`);
    return 1;
  }
  return 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) process.exitCode = main();
