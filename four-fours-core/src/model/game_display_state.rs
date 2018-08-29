
use model::{
  Card,
  GameState
}

use ui::{
  UiCard,
  Sprite
}

pub struct GameDisplayState<S> where S: Sprite {
  cards_in_play: Vec<UiCard<S>>,
  supply_cards: Vec<UiCard<S>>,
  card_in_flight: Vec<UiCard<S>>
}
