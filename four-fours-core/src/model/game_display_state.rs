
use native::{
  Texture
};

use model::{
  DraggedCardDisplayState
};

use ui::{
  UiCard,
  Sprite
};

#[derive(Getters, MutGetters, Setters)]
pub struct GameDisplayState<S> where S: Sprite {
  #[get = "pub"] #[get_mut = "pub"]
  cards_in_play: Vec<UiCard<S>>,

  #[get = "pub"] #[get_mut = "pub"]
  supply_cards: Vec<UiCard<S>>,

  #[get = "pub"] #[set = "pub"] #[get_mut = "pub"]
  card_in_flight: Option<DraggedCardDisplayState<S>>,

  #[get = "pub"] #[set = "pub"]
  supply_card_width: f64,
  #[get = "pub"] #[set = "pub"]
  supply_card_height: f64,
  #[get = "pub"] #[set = "pub"]
  supply_card_spacing: f64,

  #[get = "pub"] #[set = "pub"]
  card_in_play_width: f64,
  #[get = "pub"] #[set = "pub"]
  card_in_play_height: f64,
  #[get = "pub"] #[set = "pub"]
  card_in_play_spacing: f64
}

impl <S> Default for GameDisplayState<S> where S: Sprite {
  fn default() -> GameDisplayState<S> {
    GameDisplayState {
      cards_in_play: Vec::default(),
      supply_cards: Vec::default(),
      card_in_flight: Option::default(),

      supply_card_width: 0.,
      supply_card_height: 0.,
      supply_card_spacing: 0.,

      card_in_play_width: 0.,
      card_in_play_height: 0.,
      card_in_play_spacing: 0.,
    }
  }
}

impl <S,T> GameDisplayState<S>
    where
        T: Texture,
        S: Sprite<T = T>,
          {

}