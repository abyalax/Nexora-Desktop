import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { parseCatalog, schemaKeys, diffKeys, extractObjectLiteral, defaultSettingsFromHtml, formatReport } from "../../scripts/schema_parity.mjs";

const md = `
## System ▸ General (2)

| ID     | Setting | Design key        | Type   |
| ------ | ------- | ----------------- | ------ |
| GEN-01 | Menu    | shared.enableMenu | toggle |
| GEN-02 | Dock    | shared.enableDock | toggle |

## Dropped (ADR-0007)

* FDI-02 — Show screen mirroring control
`;

test("catalogue rows come from the Design key column, skipping header and separator", () => {
  assert.deepEqual(parseCatalog(md), [
    { id: "GEN-01", key: "shared.enableMenu", section: "System ▸ General (2)" },
    { id: "GEN-02", key: "shared.enableDock", section: "System ▸ General (2)" },
  ]);
});

test("the Design key column is found by name, not by position", () => {
  const rows = parseCatalog("## A\n\n| Design key | ID |\n| --- | --- |\n| dock.size | X-1 |\n");
  assert.deepEqual(rows, [{ id: "X-1", key: "dock.size", section: "A" }]);
});

test("schema keys are section.name pairs", () => {
  assert.deepEqual(schemaKeys({ shared: { a: 1, b: 2 }, dock: { c: 3 } }), ["shared.a", "shared.b", "dock.c"]);
});

test("diff reports keys missing on each side", () => {
  const rows = parseCatalog(md);
  const diff = diffKeys(rows, ["shared.enableDock", "shared.extra", "finder.extra"]);
  assert.deepEqual(diff.inCatalogOnly.map(r => r.id), ["GEN-01"]);
  assert.deepEqual(diff.inSchemaOnly, ["finder.extra", "shared.extra"]);
});

test("identical sides report nothing", () => {
  const diff = diffKeys(parseCatalog(md), ["shared.enableMenu", "shared.enableDock"]);
  assert.deepEqual(diff, { inCatalogOnly: [], inSchemaOnly: [] });
});

test("object literal extraction ignores braces and quotes inside strings", () => {
  const src = `const x = 1;\nconst defaultSettings = { a: { s: "}{", t: "Don't" }, b: { n: 2 } };\nfunction f() {}`;
  assert.equal(extractObjectLiteral(src, "defaultSettings"), `{ a: { s: "}{", t: "Don't" }, b: { n: 2 } }`);
  assert.deepEqual(defaultSettingsFromHtml(src), { a: { s: "}{", t: "Don't" }, b: { n: 2 } });
});

test("a missing object is an error, not an empty schema", () => {
  assert.throws(() => extractObjectLiteral("const other = {};", "defaultSettings"), /not found/);
});

test("report names ids for catalogue-only keys and groups schema-only keys by section", () => {
  const catalogRows = parseCatalog(md);
  const keys = ["shared.enableDock", "shared.x", "finder.y"];
  const text = formatReport({ catalogRows, keys, diff: diffKeys(catalogRows, keys), source: "fixture" });
  assert.match(text, /GEN-01\s+shared\.enableMenu/);
  assert.match(text, /finder \(1\): y/);
  assert.match(text, /shared \(1\): x/);
});

test("the real catalogue parses to well-formed section.name keys", () => {
  const real = fs.readFileSync(fileURLToPath(new URL("../../docs/settings_catalog.md", import.meta.url)), "utf8");
  const rows = parseCatalog(real);
  assert.ok(rows.length > 150);
  for (const r of rows) assert.match(r.key, /^[a-z]+\.[A-Za-z0-9]+$/, r.id);
  assert.equal(new Set(rows.map(r => r.id)).size, rows.length, "catalogue ids are unique");
});
