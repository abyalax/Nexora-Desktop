pub fn target_index(centers: &[f32], x: f32) -> usize {
  centers.iter().filter(|c| **c < x).count()
}

pub fn move_to<T>(v: &mut Vec<T>, from: usize, to: usize) -> bool {
  if from >= v.len() {
    return false;
  }
  let to = to.min(v.len());
  let dest = if to > from { to - 1 } else { to };
  if dest == from {
    return false;
  }
  let item = v.remove(from);
  v.insert(dest, item);
  true
}

/// DAD-04: with icon dragging locked, a drag never starts and nothing is reordered or unpinned by dropping.
/// Unpinning from the context menu is a separate path and is not gated.
pub fn may_start_drag(locked: bool) -> bool {
  !locked
}

/// [`move_to`] behind the lock (D-3.1). Returns `true` only when the order changed.
pub fn reorder<T>(v: &mut Vec<T>, from: usize, to: usize, locked: bool) -> bool {
  may_start_drag(locked) && move_to(v, from, to)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn target_counts_centers_to_the_left() {
    let c = [10.0, 30.0, 50.0];
    assert_eq!(target_index(&c, 0.0), 0);
    assert_eq!(target_index(&c, 35.0), 2);
    assert_eq!(target_index(&c, 999.0), 3);
  }

  #[test]
  fn moving_right_and_left_works() {
    let mut v = vec!['a', 'b', 'c', 'd'];
    assert!(move_to(&mut v, 0, 3));
    assert_eq!(v, ['b', 'c', 'a', 'd']);
    assert!(move_to(&mut v, 3, 0));
    assert_eq!(v, ['d', 'b', 'c', 'a']);
  }

  #[test]
  fn dropping_in_place_changes_nothing() {
    let mut v = vec![1, 2, 3];
    assert!(!move_to(&mut v, 1, 1));
    assert!(!move_to(&mut v, 1, 2));
    assert_eq!(v, [1, 2, 3]);
    assert!(!move_to(&mut v, 9, 0));
  }

  #[test]
  fn to_beyond_the_end_appends() {
    let mut v = vec![1, 2, 3];
    assert!(move_to(&mut v, 0, 99));
    assert_eq!(v, [2, 3, 1]);
  }

  #[test]
  fn reorder_is_noop_when_locked() {
    let mut v = vec![1, 2, 3];
    assert!(!reorder(&mut v, 0, 3, true));
    assert_eq!(v, [1, 2, 3]);
    assert!(!may_start_drag(true));
  }

  #[test]
  fn reorder_moves_when_unlocked() {
    let mut v = vec![1, 2, 3];
    assert!(reorder(&mut v, 0, 3, false));
    assert_eq!(v, [2, 3, 1]);
    assert!(may_start_drag(false));
  }
}
