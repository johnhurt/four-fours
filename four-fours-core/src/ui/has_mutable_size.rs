
pub trait HasMutableSize {
  fn set_size_animated(&self, width: i64, height: i64, duraction_seconds: f64);

  fn set_size(&self, width: i64, height: i64) {
    self.set_size_animated(width, height, 0.0);
  }
}

