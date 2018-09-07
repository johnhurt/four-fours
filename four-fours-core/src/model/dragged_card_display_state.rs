
use ui::{
  Sprite,
  UiCard
};

#[derive(Getters,Setters)]
pub struct DraggedCardDisplayState<S : Sprite> {

  #[get = "pub"]
  card: UiCard<S>,

  #[get = "pub"] left_orig: f64,
  #[get = "pub"] top_orig: f64,
  #[get = "pub"] width_orig: f64,
  #[get = "pub"] height_orig: f64,

  #[get = "pub"] left: f64,
  #[get = "pub"] top: f64,
  #[get = "pub"] width: f64,
  #[get = "pub"] height: f64,


  #[get = "pub"] left_from_drag_point_frac: f64,
  #[get = "pub"] top_from_drag_point_frac: f64,

}

impl <S:Sprite> DraggedCardDisplayState<S> {

  pub fn new(card: UiCard<S>,
      left: f64,
      top: f64,
      width: f64,
      height: f64,
      drag_x_in_card: f64,
      drag_y_in_card: f64) -> DraggedCardDisplayState<S> {

    let left_from_drag_point_frac = drag_x_in_card / width;
    let top_from_drag_point_frac = drag_y_in_card / height;

    let mut result = DraggedCardDisplayState {
      card: card,

      left_orig: left,
      top_orig: top,
      width_orig: width,
      height_orig: height,

      left: left,
      top: top,
      width: width,
      height: height,

      left_from_drag_point_frac: left_from_drag_point_frac,
      top_from_drag_point_frac: top_from_drag_point_frac
    };

    result.update_card();

    result
  }

  pub fn drag_move(&mut self, drag_x: f64, drag_y: f64) {
    self.left = drag_x - self.left_from_drag_point_frac * self.width;
    self.top = drag_y - self.top_from_drag_point_frac * self.height;

    self.update_card();
  }

  fn update_card(&mut self) {
    self.card.set_location_and_size(
        self.left,
        self.top,
        self.width,
        self.height);
  }

  fn extract_card(self) -> UiCard<S> {
    self.card
  }
}