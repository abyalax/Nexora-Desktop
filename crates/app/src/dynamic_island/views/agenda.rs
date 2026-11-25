use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn calendar_card(
  p: &mut Painter,
  c: &ContainerVisual,
  x: f32,
  y: f32,
  w: f32,
  h: f32,
  d: &Data,
  regions: &mut Vec<Region>,
) {
  use nexora_core::panels::calendar::{add_days, month_short, weekday, weekday_short};
  card(p, c, x, y, w, h);
  let en = crate::prefs::get().lang == nexora_core::i18n::Lang::En;
  let cal = &d.cal;
  let accent = crate::prefs::accent();
  let (sy, sm, sd) = cal.sel;
  p.text(
    c,
    month_short(sm, en),
    x + p.px(16.0),
    y + p.px(12.0),
    18.0,
    700,
    WHITE,
    1.0,
    Anchor::Left,
    p.px(60.0),
  );
  p.text(
    c,
    &sy.to_string(),
    x + p.px(16.0),
    y + p.px(36.0),
    11.0,
    500,
    WHITE,
    0.45,
    Anchor::Left,
    p.px(60.0),
  );
  let (left, right) = (x + p.px(70.0), x + w - p.px(10.0));
  let cell = (right - left) / 5.0;
  let (wy, ny, dia) = (y + p.px(10.0), y + p.px(34.0), p.px(28.0));
  for i in -2i32..=2 {
    let (yy, mm, dd) = add_days(sy, sm, sd, i);
    let cx = left + cell * (i + 2) as f32 + cell / 2.0;
    let selected = i == 0;
    let today = (yy, mm, dd) == cal.today;
    let wd = weekday_short(weekday(yy, mm, dd), en);
    let n0 = kids(c);
    p.text(
      c,
      wd,
      cx,
      wy,
      9.0,
      600,
      if selected { accent } else { WHITE },
      if selected { 1.0 } else { 0.4 },
      Anchor::Center,
      cell,
    );
    if selected {
      p.rect(
        c,
        cx - dia / 2.0,
        ny,
        dia,
        dia,
        dia / 2.0,
        Color {
          A: 255,
          R: accent[0],
          G: accent[1],
          B: accent[2],
        },
      );
    }
    let th = p.px(14.0) * 1.3;
    let (rgb, op) = if selected {
      (WHITE, 1.0)
    } else if today {
      (accent, 1.0)
    } else {
      (WHITE, 0.8)
    };
    p.text(
      c,
      &format!("{dd:02}"),
      cx,
      ny + dia / 2.0 - th / 2.0,
      14.0,
      if selected || today { 700 } else { 500 },
      rgb,
      op,
      Anchor::Center,
      cell,
    );
    if !selected {
      regions.push(Region {
        x: cx - cell / 2.0,
        y: wy,
        w: cell,
        h: ny + dia - wy,
        action: Action::CalDay(i),
        vis: group_since(c, n0),
      });
    }
  }
  let is_today = cal.sel == cal.today;
  let line = match &cal.events {
    Events::Loading => "\u{2026}".to_string(),
    Events::Unavailable => String::new(),
    Events::List([]) => tr(if is_today {
      "Tidak ada acara hari ini"
    } else {
      "Tidak ada acara"
    })
    .to_string(),
    Events::List(list) => {
      let e = &list[0];
      let when = match e.minute {
        Some(m) if crate::prefs::get().clock_24h => {
          format!("{:02}.{:02}  ", m / 60, m % 60)
        }
        Some(m) => format!(
          "{}.{:02} {}  ",
          (m / 60 + 11) % 12 + 1,
          m % 60,
          if m < 720 { "AM" } else { "PM" }
        ),
        None => String::new(),
      };
      let more = if list.len() > 1 {
        format!("  +{}", list.len() - 1)
      } else {
        String::new()
      };
      format!("{when}{}{more}", e.title)
    }
  };
  if !line.is_empty() {
    let ly = y + h - p.px(26.0);
    p.sym(c, Sym::Calendar, x + p.px(22.0), ly + p.px(7.5), 12.0, WHITE, 0.55);
    let maxw = w - p.px(44.0);
    let shown = p.fit(&line, 11.5, 500, maxw);
    p.text(
      c,
      &shown,
      x + p.px(34.0),
      ly,
      11.5,
      500,
      WHITE,
      0.75,
      Anchor::Left,
      maxw + p.px(4.0),
    );
  }
}
