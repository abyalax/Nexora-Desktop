#!/usr/bin/env node
// HTM-07: diff the Rust settings catalogue against the HTML design schema.
//
//   node scripts/schema_parity.mjs [--strict] [--catalog <md>] [--schema <js>] [--html <html>]
//
// Compares the "Design key" column of docs/settings_catalog.md with the `section.key` pairs of
// `defaultSettings` in design/settings-schema.js, and prints the keys missing on either side.
//
// Exit codes: 0 = ran (differences are only reported, so CI shows a warning, not a failure)
//             1 = differences found and --strict was given
//             2 = an input could not be read
//
// `defaultSettings` and `prefPages` are imported from design/settings-schema.js; the HTML fallback supports older fixtures.
// Pure functions are exported for design/tests/schema-parity.test.mjs; the CLI runs only when executed directly.

import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DEFAULTS = {
  catalog: path.join(ROOT, "docs/settings_catalog.md"),
  schema: path.join(ROOT, "design/settings-schema.js"),
  html: path.join(ROOT, "design/nexora-v1.html"),
};

const cells = line => line.trim().replace(/^\||\|$/g, "").split("|").map(c => c.trim());

// Rows of every catalogue table that has a "Design key" column: [{ id, key, section }].
export function parseCatalog(markdown) {
  const rows = [];
  let section = "";
  let idCol = -1;
  let keyCol = -1;
  for (const line of markdown.split(/\r?\n/)) {
    if (line.startsWith("## ")) {
      section = line.slice(3).trim();
      idCol = keyCol = -1;
    } else if (line.startsWith("|")) {
      const c = cells(line);
      if (c.includes("Design key")) {
        idCol = c.indexOf("ID");
        keyCol = c.indexOf("Design key");
      } else if (keyCol >= 0 && idCol >= 0 && !/^-+$/.test(c[0])) rows.push({ id: c[idCol], key: c[keyCol], section });
    }
  }
  return rows;
}

// Every `section.name` of the schema's defaultSettings object.
export function schemaKeys(defaultSettings) {
  return Object.entries(defaultSettings).flatMap(([section, fields]) =>
    Object.keys(fields).map(name => `${section}.${name}`),
  );
}

// Keys missing on either side. `inSchemaOnly` and `inCatalogOnly` are sorted; catalogue rows keep their id.
export function diffKeys(catalogRows, keys) {
  const have = new Set(keys);
  const listed = new Set(catalogRows.map(r => r.key));
  return {
    inCatalogOnly: catalogRows.filter(r => !have.has(r.key)).sort((a, b) => a.id.localeCompare(b.id)),
    inSchemaOnly: [...new Set(keys)].filter(k => !listed.has(k)).sort(),
  };
}

// Text of the object literal that starts at the first `{` after `const <name> =`.
export function extractObjectLiteral(source, name) {
  const start = source.search(new RegExp(`const\\s+${name}\\s*=\\s*\\{`));
  if (start < 0) throw new Error(`const ${name} not found`);
  const open = source.indexOf("{", start);
  let depth = 0;
  let quote = null;
  for (let i = open; i < source.length; i++) {
    const ch = source[i];
    if (quote) {
      if (ch === "\\") i++;
      else if (ch === quote) quote = null;
    } else if (ch === '"' || ch === "'" || ch === "`") quote = ch;
    else if (ch === "{") depth++;
    else if (ch === "}" && --depth === 0) return source.slice(open, i + 1);
  }
  throw new Error(`unterminated object literal for ${name}`);
}

export function defaultSettingsFromHtml(html) {
  // JSON round trip: the sandbox's objects belong to another realm.
  return JSON.parse(JSON.stringify(vm.runInNewContext(`(${extractObjectLiteral(html, "defaultSettings")})`)));
}

// Read design/settings-schema.js, falling back to inline HTML only for callers that pass an older fixture.
export function loadDefaultSettings({ schema = DEFAULTS.schema, html = DEFAULTS.html } = {}) {
  if (fs.existsSync(schema)) {
    const mod = createRequire(import.meta.url)(schema);
    if (!mod.defaultSettings) throw new Error(`${schema} does not export defaultSettings`);
    return { settings: mod.defaultSettings, source: path.relative(ROOT, schema) };
  }
  return {
    settings: defaultSettingsFromHtml(fs.readFileSync(html, "utf8")),
    source: `${path.relative(ROOT, html)} (settings-schema.js not found, HTM-01 pending)`,
  };
}

const bySection = keys => {
  const groups = new Map();
  for (const k of keys) {
    const section = k.split(".")[0];
    groups.set(section, [...(groups.get(section) ?? []), k]);
  }
  return groups;
};

export function formatReport({ catalogRows, keys, diff, source }) {
  const out = [
    `Rust catalogue: ${catalogRows.length} rows · HTML schema: ${keys.length} keys (${source})`,
    "",
    `In the catalogue, missing from the HTML schema (${diff.inCatalogOnly.length}):`,
    ...(diff.inCatalogOnly.length ? diff.inCatalogOnly.map(r => `  ${r.id.padEnd(8)} ${r.key}`) : ["  none"]),
    "",
    `In the HTML schema, missing from the catalogue (${diff.inSchemaOnly.length}):`,
  ];
  if (!diff.inSchemaOnly.length) out.push("  none");
  for (const [section, list] of bySection(diff.inSchemaOnly)) out.push(`  ${section} (${list.length}): ${list.map(k => k.split(".")[1]).join(", ")}`);
  return out.join("\n");
}

function parseArgs(argv) {
  const opts = { strict: false, ...DEFAULTS };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--strict") opts.strict = true;
    else if (["--catalog", "--schema", "--html"].includes(argv[i]) && argv[i + 1]) opts[argv[i].slice(2)] = path.resolve(argv[++i]);
    else throw new Error(`unknown argument: ${argv[i]}`);
  }
  return opts;
}

export function main(argv = process.argv.slice(2)) {
  let opts, catalogRows, loaded;
  try {
    opts = parseArgs(argv);
    catalogRows = parseCatalog(fs.readFileSync(opts.catalog, "utf8"));
    loaded = loadDefaultSettings(opts);
  } catch (err) {
    console.error(`schema parity: ${err.message}`);
    return 2;
  }
  const keys = schemaKeys(loaded.settings);
  const diff = diffKeys(catalogRows, keys);
  console.log(formatReport({ catalogRows, keys, diff, source: loaded.source }));
  const total = diff.inCatalogOnly.length + diff.inSchemaOnly.length;
  if (process.env.GITHUB_ACTIONS && total) {
    console.log(`::warning title=HTML schema parity::${diff.inCatalogOnly.length} catalogue keys missing in HTML, ${diff.inSchemaOnly.length} HTML keys missing in catalogue`);
  }
  return opts.strict && total ? 1 : 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) process.exitCode = main();
