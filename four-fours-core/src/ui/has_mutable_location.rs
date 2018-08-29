
pub trait HasMutableLocation {
  fn set_location_animated(&self, left: i64, top: i64, duration_seconds: f64);
  fn set_location(&self, left: i64, top: i64) {
    self.set_location_animated(left, top, 0.0)
  }
}

