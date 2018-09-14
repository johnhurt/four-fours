
#[derive(Clone, Default, Getters, Debug)]
pub struct BindPoint {
  #[get = "pub"] index: usize,
  #[get = "pub"] x: f64
}

impl BindPoint {
  pub fn new(index: usize, x: f64) -> BindPoint {
    BindPoint {
      index: index,
      x: x
    }
  }
}