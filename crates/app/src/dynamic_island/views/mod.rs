use super::draw::{Anchor, Painter, WHITE, white};
use crate::prefs::tr;
use nexora_core::dynamic_island::glance::Item;
use nexora_core::dynamic_island::state::{Activity, Look, Tab, View};
use nexora_core::dynamic_island::timer::{Phase, Timer, format_mmss};
use nexora_core::symbols::Sym;
use windows::UI::Color;
use windows::UI::Composition::{CompositionDrawingSurface, ContainerVisual, SpriteVisual};
use windows_numerics::Vector3;

mod shelf;
mod countdown;
mod music;
mod glance;
mod lyrics;
mod agenda;

#[allow(unused_imports)]
use self::{shelf::*, countdown::*, music::*, glance::*, lyrics::*, agenda::*};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
  PlayPause,
  Next,
  Prev,
  Seek(f32, f32),
  Tab(Tab),
  ShelfItem(usize),
  ShelfClear,
  Zone(usize),
  Settings,
  TimerToggle,
  TimerReset,
  TimerPlus,
  TimerMinus,
  CalDay(i32),
  SendDone,
}

pub struct Region {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
  pub action: Action,
  pub vis: Option<ContainerVisual>,
}

fn kids(c: &ContainerVisual) -> i32 {
  c.Children().and_then(|k| k.Count()).unwrap_or(0)
}

fn group_since(c: &ContainerVisual, n0: i32) -> Option<ContainerVisual> {
  let col = c.Children().ok()?;
  let mut all = Vec::new();
  let it = col.First().ok()?;
  while it.HasCurrent().unwrap_or(false) {
    all.push(it.Current().ok()?);
    let _ = it.MoveNext();
  }
  let added = all.get(n0.max(0) as usize..)?;
  if added.is_empty() {
    return None;
  }
  let g = c.Compositor().ok()?.CreateContainerVisual().ok()?;
  let gk = g.Children().ok()?;
  for v in added {
    col.Remove(v).ok()?;
    gk.InsertAtTop(v).ok()?;
  }
  col.InsertAtTop(&g).ok()?;
  Some(g)
}

impl Region {
  pub fn hit(&self, x: f32, y: f32) -> bool {
    x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
  }
}

#[derive(Clone, Copy)]
pub struct Frame {
  pub x0: f32,
  pub w: f32,
  pub h: f32,
}

pub struct Data<'a> {
  pub battery: Option<(u8, bool)>,
  pub timer: &'a Timer,
  pub now_ms: i64,
  pub glance: Option<(Item, String)>,
  pub media: Option<MediaView<'a>>,
  pub lyrics: LyricsView<'a>,
  pub shelf: Vec<(&'a str, Option<&'a CompositionDrawingSurface>)>,
  pub drop_zone: Option<usize>,
  pub drop_count: usize,
  pub privacy: (bool, bool),
  pub download: Option<DownloadView<'a>>,
  pub cal: CalView<'a>,
  pub send: Option<SendView<'a>>,
  pub send_zone: bool,
}

pub struct SendView<'a> {
  pub qr: Option<&'a CompositionDrawingSurface>,
  pub qr_px: f32,
  pub title: String,
  pub status: String,
  pub frac: f32,
  pub done: bool,
  pub error: bool,
}

pub type Date = (i32, u32, u32);

pub struct CalView<'a> {
  pub today: Date,
  pub sel: Date,
  pub events: Events<'a>,
}

pub enum Events<'a> {
  Loading,
  Unavailable,
  List(&'a [nexora_platform::agenda::Event]),
}

pub struct DownloadView<'a> {
  pub icon: Option<&'a CompositionDrawingSurface>,
  pub label: String,
  pub frac: Option<f32>,
  pub done: bool,
  pub up: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LyrState {
  Idle,
  Loading,
  Ready,
  Missing,
}

pub struct LyricsView<'a> {
  pub lines: Option<&'a nexora_core::dynamic_island::lyrics::Lyrics>,
  pub state: LyrState,
  pub pos_ms: i64,
  pub lrclib: bool,
}

pub struct MediaView<'a> {
  pub title: &'a str,
  pub artist: &'a str,
  pub playing: bool,
  pub pos_ms: i64,
  pub dur_ms: i64,
  pub can_seek: bool,
  pub art: Option<&'a CompositionDrawingSurface>,
  pub tint: Option<[u8; 3]>,
  pub drag: Option<f32>,
}

#[derive(Default)]
pub struct Live {
  pub bars: Vec<SpriteVisual>,
  pub lyric: Option<SpriteVisual>,
  pub lyric_scroll: Option<windows::UI::Composition::ContainerVisual>,
  pub lyric_shift: f32,
}

const ORANGE: [u8; 3] = [255, 159, 10];
const GREEN: [u8; 3] = [48, 209, 88];
const INK: [u8; 3] = [22, 22, 24];

fn card(p: &Painter, c: &ContainerVisual, x: f32, y: f32, w: f32, h: f32) {
  p.rect(c, x, y, w, h, p.px(18.0), white(18));
}

pub fn draw(
  p: &mut Painter,
  c: &ContainerVisual,
  look: Look,
  f: Frame,
  d: &Data,
  regions: &mut Vec<Region>,
  live: &mut Live,
) {
  match look.view {
    View::Rest => {}
    View::Compact(Activity::Timer) | View::Compact(Activity::TimerEnding) => {
      let urgent = look.view == View::Compact(Activity::TimerEnding);
      compact_timer(p, c, f, d, urgent)
    }
    View::Compact(Activity::Charging) => compact_charging(p, c, f, d),
    View::Compact(Activity::Glance) => compact_glance(p, c, f, d),
    View::Compact(Activity::Music) => compact_music(p, c, f, d, live, f.h),
    View::Compact(Activity::Lyrics) => compact_lyrics(p, c, f, d, live),
    View::Compact(Activity::Shelf) => compact_shelf(p, c, f, d),
    View::Compact(Activity::Download) => compact_download(p, c, f, d),
    View::Compact(Activity::Player) => compact_player(p, c, f, d, regions, live),
    View::Open(tab) => {
      header(p, c, f, tab, d, regions);
      match tab {
        Tab::Play => play(p, c, f, d, regions, live),
        Tab::Shelf if d.send.is_some() => send_card(p, c, f, d, regions),
        Tab::Shelf if d.shelf.is_empty() => shelf_empty(p, c, f),
        Tab::Shelf => shelf_tab(p, c, f, d, regions),
        Tab::Timer => timer(p, c, f, d, regions),
      }
    }
    View::Drop => drop_zones(p, c, f, d, regions),
  }
  if matches!(look.view, View::Rest | View::Compact(_)) {
    privacy_dots(p, c, f, d.privacy);
  }
}

fn privacy_dots(p: &mut Painter, c: &ContainerVisual, f: Frame, (mic, cam): (bool, bool)) {
  let (r, cy) = (p.px(3.0), f.h / 2.0);
  let mut x = f.x0 + f.w - p.px(7.0);
  for (on, rgb) in [(mic, ORANGE), (cam, GREEN)] {
    if on {
      p.rect(
        c,
        x - r,
        cy - r,
        2.0 * r,
        2.0 * r,
        r,
        Color {
          A: 255,
          R: rgb[0],
          G: rgb[1],
          B: rgb[2],
        },
      );
      x -= p.px(9.0);
    }
  }
}

pub(super) fn count_text(n: usize) -> String {
  if crate::prefs::get().lang == nexora_core::i18n::Lang::En {
    format!("{n} {}", if n == 1 { "file" } else { "files" })
  } else {
    format!("{n} berkas")
  }
}
