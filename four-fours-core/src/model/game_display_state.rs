
use native::{
  Texture
};

use model::{
  DraggedCardDisplayState,
  BindPoint
};

use ui::{
  UiCard,
  Sprite
};

const MAX_ROW_DISTANCE_HEIGHT_FRAC: f64 = 0.5;
const MAX_COL_DISTANCE_WIDTH_FRAC: f64 = 0.7;

#[derive(Getters, MutGetters, Setters)]
pub struct GameDisplayState<S> where S: Sprite {

  #[get = "pub"] #[set = "pub"] #[get_mut = "pub"] width: f64,
  #[get = "pub"] #[set = "pub"] #[get_mut = "pub"] height: f64,
  #[get = "pub"] #[set = "pub"] #[get_mut = "pub"] card_aspect_ratio: f64,
  #[get = "pub"] #[set = "pub"] #[get_mut = "pub"] border_thickness: f64,

  #[get = "pub"] #[get_mut = "pub"]
  cards_in_play: Vec<UiCard<S>>,

  #[get = "pub"] #[get_mut = "pub"]
  supply_cards: Vec<UiCard<S>>,

  #[get = "pub"] #[set = "pub"] #[get_mut = "pub"]
  card_in_flight: Option<DraggedCardDisplayState<S>>,

  /// List of points that represent the places that cards can be added
  /// to the playing area
  #[get = "pub"] #[set = "pub"] #[get_mut = "pub"]
  bind_points: Vec<(f64,Vec<(f64,BindPoint)>)>,

  #[get = "pub"] #[set = "pub"] supply_card_width: f64,
  #[get = "pub"] #[set = "pub"] supply_card_height: f64,
  #[get = "pub"] #[set = "pub"] supply_card_spacing: f64,

  #[get = "pub"] #[set = "pub"] supply_row_count: f64,
  #[get = "pub"] #[set = "pub"] supply_top: f64,
  #[get = "pub"] #[set = "pub"] supply_left: f64,
  #[get = "pub"] #[set = "pub"] supply_width: f64,
  #[get = "pub"] #[set = "pub"] supply_height: f64,

  #[get = "pub"] #[set = "pub"] play_card_width: f64,
  #[get = "pub"] #[set = "pub"] play_card_height: f64,
  #[get = "pub"] #[set = "pub"] play_card_spacing: f64,

  #[get = "pub"] #[set = "pub"] play_area_row_count: f64,
  #[get = "pub"] #[set = "pub"] play_area_top: f64,
  #[get = "pub"] #[set = "pub"] play_area_left: f64,
  #[get = "pub"] #[set = "pub"] play_area_width: f64,
  #[get = "pub"] #[set = "pub"] play_area_height: f64,
}

impl <S> Default for GameDisplayState<S> where S: Sprite {
  fn default() -> GameDisplayState<S> {
    GameDisplayState {
      width: 0.,
      height: 0.,

      card_aspect_ratio: 0.,
      border_thickness: 0.,

      cards_in_play: Vec::default(),
      supply_cards: Vec::default(),
      card_in_flight: Option::default(),

      bind_points: Vec::default(),

      supply_card_width: 0.,
      supply_card_height: 0.,
      supply_card_spacing: 0.,

      play_card_width: 0.,
      play_card_height: 0.,
      play_card_spacing: 0.,

      supply_row_count: 0.,
      supply_top: 0.,
      supply_left: 0.,
      supply_width: 0.,
      supply_height: 0.,

      play_area_row_count: 0.,
      play_area_top: 0.,
      play_area_left: 0.,
      play_area_width: 0.,
      play_area_height: 0.,
    }
  }
}

impl <S,T> GameDisplayState<S>
    where
        T: Texture,
        S: Sprite<T = T>,
          {

  pub fn get_closest_bind_point(&self, x: f64, y: f64) -> Option<BindPoint> {

    let max_row_dist = MAX_ROW_DISTANCE_HEIGHT_FRAC * self.play_area_height;
    let max_col_dist = MAX_COL_DISTANCE_WIDTH_FRAC * self.play_area_width;
    let mut min_row_dist : f64 = 1_000_000.;
    let mut best_row : Option<&Vec<(f64,BindPoint)>> = None;

    for (row, columns) in &self.bind_points {
      let row_dist = (row - y).abs();
      if row_dist < max_row_dist && row_dist < min_row_dist {
        min_row_dist = row_dist;
        best_row = Some(columns)
      }
    }

    match best_row {
      None => None,
      Some(columns) => {
        let mut min_col_dist : f64 = 1_000_000.;
        let mut best_point : Option<&BindPoint> = None;
        for (col, bind_point) in columns {
          let col_dist = (col - x).abs();

          info!("WAT? {}", bind_point.index());

          if col_dist < max_col_dist && col_dist < min_col_dist {
            min_col_dist = col_dist;
            best_point = Some(bind_point);

            info!("In Progress ({}) -> {:?}", min_col_dist, best_point);
          }
        }

        info!("Result ({},{}) -> {:?}", x, y, best_point);
        best_point.map(Clone::clone)
      }
    }
  }


}