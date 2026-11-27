//! Value/position mapping for numeric settings sliders.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericSlider {
  min: f32,
  max: f32,
  step: f32,
}

impl NumericSlider {
  pub fn new(min: f32, max: f32, step: f32) -> Option<Self> {
    (min.is_finite() && max.is_finite() && step.is_finite() && max > min && step > 0.0).then_some(Self {
      min,
      max,
      step,
    })
  }

  pub fn min(self) -> f32 {
    self.min
  }

  pub fn max(self) -> f32 {
    self.max
  }

  pub fn step(self) -> f32 {
    self.step
  }

  pub fn value_to_position(self, value: f32) -> f32 {
    ((value.clamp(self.min, self.max) - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
  }

  pub fn position_to_value(self, position: f32) -> f32 {
    let raw = self.min + position.clamp(0.0, 1.0) * (self.max - self.min);
    let stepped = self.min + ((raw - self.min) / self.step).round() * self.step;
    stepped.clamp(self.min, self.max)
  }
}

impl Default for NumericSlider {
  fn default() -> Self {
    Self {
      min: 0.0,
      max: 1.0,
      step: 1.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn wide_slider_maps_zero_to_fifteen_hundred() {
    let slider = NumericSlider::new(0.0, 1500.0, 10.0).unwrap();
    assert_eq!(slider.value_to_position(0.0), 0.0);
    assert_eq!(slider.value_to_position(1500.0), 1.0);
    assert_eq!(slider.position_to_value(0.5), 750.0);
    assert_eq!(slider.position_to_value(2.0), 1500.0);
  }

  #[test]
  fn fractional_values_round_trip_to_the_nearest_step() {
    let slider = NumericSlider::new(0.75, 1.5, 0.05).unwrap();
    let value = slider.position_to_value(slider.value_to_position(1.23));
    assert!((value - 1.25).abs() < 0.0001);
  }

  #[test]
  fn invalid_ranges_are_rejected() {
    assert!(NumericSlider::new(1.0, 1.0, 0.1).is_none());
    assert!(NumericSlider::new(0.0, 1.0, 0.0).is_none());
    assert!(NumericSlider::new(f32::NAN, 1.0, 0.1).is_none());
  }
}
