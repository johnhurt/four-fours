use ui::HasSize;

pub trait Texture : HasSize + 'static {
  fn get_sub_texture(&self, left: i64, top: i64, width: i64, height: i64) -> Self;

  fn get_aspect_ratio(&self) -> f64 {
    (self.get_width() as f64) / (self.get_height() as f64)
  }
}