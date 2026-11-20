#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShortText {
  len: u8,
  buf: [u8; 96],
}

impl ShortText {
  pub const EMPTY: ShortText = ShortText { len: 0, buf: [0; 96] };

  pub fn new(s: &str) -> ShortText {
    let clean: String = s.chars().filter(|c| !c.is_control()).collect();
    let clean = clean.trim();
    let mut end = clean.len().min(96);
    while !clean.is_char_boundary(end) {
      end -= 1;
    }
    let mut buf = [0u8; 96];
    buf[..end].copy_from_slice(&clean.as_bytes()[..end]);
    ShortText { len: end as u8, buf }
  }

  pub fn as_str(&self) -> &str {
    std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
  }

  pub fn is_empty(&self) -> bool {
    self.len == 0
  }
}

impl Default for ShortText {
  fn default() -> Self {
    ShortText::EMPTY
  }
}

impl std::fmt::Debug for ShortText {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self.as_str())
  }
}
