import test from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
const A = createRequire(import.meta.url)("../island-activities.js");

const on = { mediaLyrics: true, downloads: true, fileShelf: true, sendToPhone: true, timerCalendar: true };
const off = Object.fromEntries(Object.keys(on).map(k => [k, false]));

test("all on shows every tab in core order", () => {
  assert.deepEqual(A.visibleTabs(A.fromSettings(on)), ["play", "shelf", "timer"]);
});

test("a disabled activity never appears", () => {
  assert.deepEqual(A.visibleTabs(A.fromSettings({ ...on, fileShelf: false })), ["play", "timer"]);
  assert.deepEqual(A.visibleTabs(A.fromSettings({ ...on, mediaLyrics: false })), ["shelf", "timer"]);
  assert.deepEqual(A.visibleTabs(A.fromSettings({ ...on, timerCalendar: false })), ["play", "shelf"]);
});

test("all off leaves idle only", () => {
  const a = A.fromSettings(off);
  assert.deepEqual(A.visibleTabs(a), []);
  assert.equal(A.firstTab(a), null);
  assert.equal(A.acceptsDrops(a), false);
});

test("downloads and send are not tabs", () => {
  assert.deepEqual(A.visibleTabs(A.fromSettings({ ...off, downloads: true, sendToPhone: true })), []);
});

test("drops are accepted by shelf or send", () => {
  assert.equal(A.acceptsDrops(A.fromSettings({ ...off, sendToPhone: true })), true);
  assert.equal(A.acceptsDrops(A.fromSettings({ ...off, fileShelf: true })), true);
});

test("retarget moves a disabled tab to the first enabled one", () => {
  const a = A.fromSettings({ ...on, mediaLyrics: false });
  assert.equal(A.retarget("play", a), "shelf");
  assert.equal(A.retarget("timer", a), "timer");
  assert.equal(A.retarget("play", A.fromSettings(off)), null);
  assert.equal(A.retarget(null, a), null);
});
