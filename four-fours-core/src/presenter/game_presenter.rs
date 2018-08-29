
use std::cmp;
use std::sync::{Mutex,Arc};

use itertools::Itertools;

use event::{
  EventBus,
  EventListener,
  ListenerRegistration,
  FourFoursEvent,
  Layout
};

use model::{
  GameState,
  Card,
  GameSetup
};

use native::{
  Texture,
  RuntimeResources,
  SystemView
};

use ui::{
  GameView,
  LayoutHandler,
  HandlerRegistration,
  Sprite,
  SpriteSource,
  UiCard,
  HasMutableSize,
  HasMutableLocation
};

const BOUNDARY_FRACTION : f64 = 0.05;
const SPACING_FRACTION : f64 = 0.03;
const MAX_CARD_WIDTH_FRAC : f64 = 0.2;
const MAX_CARD_HEIGHT_FRAC : f64 = 0.35;

pub struct GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  view: V,
  event_bus: Arc<EventBus>,
  runtime_resources: Arc<RuntimeResources<S>>,
  listener_registrations: Mutex<Vec<ListenerRegistration>>,
  handler_registrations: Mutex<Vec<Box<HandlerRegistration>>>,

  game_state: GameState,
  card_sprites_in_play: Vec<UiCard<V::S>>
}

impl <V,S> EventListener<Layout> for GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  fn on_event(&self, event: &Layout) {
    info!("Game view resized to : {}, {}", event.width, event.height);

    let width = event.width as f64;
    let height = event.height as f64;

    let card_aspect_ratio
        = self.runtime_resources.textures().card().get_aspect_ratio();

    let max_card_height = height * MAX_CARD_HEIGHT_FRAC;
    let max_card_width = (card_aspect_ratio * max_card_height).min(
        width * MAX_CARD_WIDTH_FRAC);

    let border_thickness = width * BOUNDARY_FRACTION;
    let cards_in_play_count = self.card_sprites_in_play.len() as f64;

    let playing_area_width = width - 2.0 * border_thickness;

    if cards_in_play_count > 0.0 {

      let card_in_play_width = max_card_width.min(
          playing_area_width / ( cards_in_play_count
              + (cards_in_play_count - 1.0) * SPACING_FRACTION ));

      let card_in_play_height = card_in_play_width / card_aspect_ratio;

      let card_in_play_top = height / 2.0 - card_in_play_height / 2.0;

      let cards_in_play_plus_spacing_width
          = cards_in_play_count * card_in_play_width
          + (cards_in_play_count - 1.) * card_in_play_width * SPACING_FRACTION;

      let first_card_in_play_left
          = width / 2.0 - cards_in_play_plus_spacing_width / 2.0;

      self.card_sprites_in_play.iter().enumerate().for_each(|(i, card)| {
        let idx = i as f64;

        let card_in_play_left = first_card_in_play_left
            + idx * (1.0 + SPACING_FRACTION) * card_in_play_width;

        card.set_location_and_size(
            card_in_play_left,
            card_in_play_top,
            card_in_play_width,
            card_in_play_height);
      });
    }


  }
}

impl <V,S> GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {

  fn add_listener_registration(&self, lr: ListenerRegistration) {
    if let Ok(mut locked_list) = self.listener_registrations.lock() {
      locked_list.push(lr);
    }
  }

  fn add_handler_registration(&self, hr: Box<HandlerRegistration>) {
    if let Ok(mut locked_list) = self.handler_registrations.lock() {
      locked_list.push(hr);
    }
  }

  fn bind(self) -> Arc<GamePresenter<V,S>> {
    let copied_event_bus = self.event_bus.clone();

    self.add_handler_registration(Box::new(self.view
        .add_layout_handler(create_layout_handler!(|w, h| {
            copied_event_bus.post(Layout{width: w, height: h})
        }))));

    let result = Arc::new(self);

    result.add_listener_registration(
        result.event_bus.register(FourFoursEvent::Layout, &result));

    result
  }

  pub fn new(
      view: V,
      event_bus: Arc<EventBus>,
      runtime_resources: Arc<RuntimeResources<S>>)
          -> Arc<GamePresenter<V,S>> {

    let game_state = GameState::new(GameSetup::simple_new(1, vec![4, 4, 4, 4]));
    let card_sprites_in_play = game_state.cards_in_play().iter()
        .map(|card| {
          UiCard::new(card, &runtime_resources.textures(), &view)
        })
        .collect();

    let result = GamePresenter{
      view: view,
      event_bus: event_bus,
      runtime_resources: runtime_resources,
      listener_registrations: Mutex::new(Vec::new()),
      handler_registrations: Mutex::new(Vec::new()),

      game_state: game_state,
      card_sprites_in_play: card_sprites_in_play
    };

    result.bind()
  }

}

impl <V,S> Drop for GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  fn drop(&mut self) {
    info!("Dropping Game Presenter")
  }
}