// Pure island activity logic. Mirrors crates/core/src/dynamic_island/activities.rs (I-14.1).
// No DOM access, so Node can import it (see design/tests/island-activities.test.mjs).
(function (root, factory) {
  if (typeof module === "object" && module.exports) module.exports = factory();
  else root.NexoraIslandActivities = factory();
})(typeof self !== "undefined" ? self : this, function () {
  // Same order as Tab::ALL in core: Play, Shelf, Timer.
  const TABS = ["play", "shelf", "timer"];

  // Settings keys (design) -> activity flags (core `Activities`).
  function fromSettings(island) {
    return {
      media: !!island.mediaLyrics,
      downloads: !!island.downloads,
      shelf: !!island.fileShelf,
      send: !!island.sendToPhone,
      timer: !!island.timerCalendar,
    };
  }

  function tabEnabled(a, tab) {
    if (tab === "play") return a.media;
    if (tab === "shelf") return a.shelf;
    if (tab === "timer") return a.timer;
    return false;
  }

  const visibleTabs = a => TABS.filter(t => tabEnabled(a, t));
  const firstTab = a => visibleTabs(a)[0] || null;
  const acceptsDrops = a => a.shelf || a.send;

  // A tab that was switched off hands over to the first enabled tab, or null (idle only).
  function retarget(current, a) {
    if (current && !tabEnabled(a, current)) return firstTab(a);
    return current;
  }

  return { TABS, fromSettings, tabEnabled, visibleTabs, firstTab, acceptsDrops, retarget };
});
