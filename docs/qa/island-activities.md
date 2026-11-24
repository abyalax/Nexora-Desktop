# QA — Island activity switches (I-14, IAC-01/05/07)

Needs Windows 11, an interactive session, and a build of this branch. Not verified by automated tests: the pure logic is
covered by `cargo test -p nexora-core activities`; this checks the window behavior.

## Preconditions

- nexora running with the Dynamic Island enabled; Settings ▸ Dynamic Island ▸ Activities open.
- Media playing in any player, a timer running, one file on the shelf, a download in progress (any browser).
- Note the taskbar state and work area before starting (restore check below).

## Steps and expected results

| # | Do | Expect |
| - | -- | ------ |
| 1 | Turn **Media and lyrics** off | Compact island no longer shows the player, lyrics or music; the **Play** tab disappears from the open island; clicking the island opens the next enabled tab (Shelf). |
| 2 | Turn it back on | Play tab and compact media return without restart. |
| 3 | Turn **File shelf** off | The **Shelf** tab disappears; dragging a file shows only the send zone (if *Send to phone* is on) and dropping on the shelf zone does nothing. Known gap: the shelf zone is still drawn in the drop overlay. |
| 4 | Turn **Timer and calendar strip** off with a timer running | No timer activity in the compact island, no Timer tab. The timer itself keeps counting (turn the switch on again: it shows the correct time). |
| 5 | Turn all five activity switches off | Collapsed island shows only the idle style; clicking it does not open anything; mouse wheel on the island does nothing. |
| 6 | Mouse wheel over the open island with one tab off | Cycles only through the enabled tabs. |

## Restore check

Taskbar visibility and the work area are the same as before the test; all switches back to on.
