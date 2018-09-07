
use std::sync::{Mutex,Arc,RwLock};

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
  GameSetup,
  GameDisplayState,
  DraggedCardDisplayState
};

use native::{
  HasIntSize,
  RuntimeResources,
  SystemView
};

use ui::{
  GameView,
  LayoutHandler,
  HandlerRegistration,
  HasMutableVisibility,
  UiCard,
  DragHandler
};

const BOUNDARY_FRACTION : f64 = 0.04;
const SPACING_FRACTION : f64 = 0.04;
const MAX_CARD_WIDTH_FRAC : f64 = 0.2;
const MAX_CARD_HEIGHT_FRAC : f64 = 0.35;

const MIN_CARD_WIDTH_PTS : f64 = 45.0;

pub struct GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  view: V,
  event_bus: Arc<EventBus>,
  runtime_resources: Arc<RuntimeResources<S>>,
  listener_registrations: Mutex<Vec<ListenerRegistration>>,
  handler_registrations: Mutex<Vec<Box<HandlerRegistration>>>,

  game_state: RwLock<GameState>,
  display_state: RwLock<GameDisplayState<V::S>>
}

impl <V,S> EventListener<Layout> for GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  fn on_event(&self, event: &Layout) {
    info!("Game view resized to : {}, {}", event.width, event.height);

    let game_state = self.game_state.read()
        .expect("Failed to get read lock on game_state");

    let mut display_state = self.display_state.write()
        .expect("Failed to lock display state for reading");

    let width = event.width as f64;
    let height = event.height as f64;

    let card_aspect_ratio
        = self.runtime_resources.textures().card().get_aspect_ratio();

    let max_card_height = height * MAX_CARD_HEIGHT_FRAC;
    let max_card_width = (card_aspect_ratio * max_card_height).min(
        width * MAX_CARD_WIDTH_FRAC);

    let border_thickness = width * BOUNDARY_FRACTION;
    let playing_area_width = width - 2.0 * border_thickness;


    let supply_card_count = game_state.setup().supply_cards().len() as f64;

    let supply_card_width = MIN_CARD_WIDTH_PTS.max(max_card_width.min(
          playing_area_width / ( supply_card_count
              + (supply_card_count - 1.0) * SPACING_FRACTION )));

    let min_supply_card_spacing = SPACING_FRACTION * supply_card_width;

    let supply_cards_per_row
        = ((playing_area_width + min_supply_card_spacing)
            / (supply_card_width + min_supply_card_spacing)).floor();

    let supply_row_count = (supply_card_count / supply_cards_per_row).ceil();

    let supply_card_spacing
        = if supply_cards_per_row > 1.0 {
          (playing_area_width - supply_cards_per_row * supply_card_width)
              / (supply_cards_per_row - 1.0)
        }
        else {
          min_supply_card_spacing
        };

    let supply_width = supply_cards_per_row * supply_card_width
          + (supply_cards_per_row - 1.) * supply_card_spacing;

    let supply_card_height = supply_card_width / card_aspect_ratio;

    let supply_height = supply_row_count * supply_card_height
        + (supply_row_count - 1.0) *supply_card_spacing;

    let supply_card_height = supply_card_width / card_aspect_ratio;
    let first_supply_card_top = height - border_thickness - supply_height;
    let first_supply_card_left = width / 2.0 - supply_width / 2.0;

    display_state.set_supply_card_width(supply_card_width);
    display_state.set_supply_card_height(supply_card_height);
    display_state.set_supply_card_spacing(supply_card_spacing);

    display_state
        .supply_cards()
        .iter()
        .enumerate()
        .for_each(|(i, card)| {
          let supply_card_top = first_supply_card_top
              + (supply_card_height + supply_card_spacing)
              * ((i / (supply_cards_per_row as usize)) as f64);
          let supply_card_left = first_supply_card_left
              + (supply_card_width + supply_card_spacing)
              * ((i % (supply_cards_per_row as usize)) as f64);
          card.set_location_and_size(
              supply_card_left,
              supply_card_top,
              supply_card_width,
              supply_card_height);
          card.set_visible(true);
        });

    let cards_in_play_count
        = display_state.cards_in_play().len() as f64;

    if cards_in_play_count > 0.0 {

      let card_in_play_width = max_card_width.min(
          playing_area_width / ( cards_in_play_count
              + (cards_in_play_count - 1.0) * SPACING_FRACTION ));

      let card_in_play_height = card_in_play_width / card_aspect_ratio;

      let card_in_play_top = first_supply_card_top / 2.0
          - card_in_play_height / 2.0;

      let cards_in_play_plus_spacing_width
          = cards_in_play_count * card_in_play_width
          + (cards_in_play_count - 1.) * card_in_play_width * SPACING_FRACTION;

      let first_card_in_play_left
          = width / 2.0 - cards_in_play_plus_spacing_width / 2.0;

      display_state.cards_in_play()
          .iter()
          .enumerate()
          .for_each(|(i, card)| {
            let idx = i as f64;

            let card_in_play_left = first_card_in_play_left
                + idx * (1.0 + SPACING_FRACTION) * card_in_play_width;

            card.set_location_and_size(
                card_in_play_left,
                card_in_play_top,
                card_in_play_width,
                card_in_play_height);
            card.set_visible(true);
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

  /// Creates a ui card based on the given card
  fn create_ui_card(&self, card: Card,
      drag_handler: Option<DragHandler>) -> UiCard<V::S> {
    UiCard::new(
        card,
        self.runtime_resources.textures(),
        &self.view,
        drag_handler)
  }

  /// handle the initialization of a supply card being dragged.  This will
  /// create a new ui card and load it into the display state's dragged-card
  /// display state
  fn on_supply_card_drag_start(&self,
      card: Card,
      window_x: f64,
      window_y: f64,
      card_x: f64,
      card_y: f64) {
    info!("Supply card drag started: {}, {}, {}, {}",
        window_x,
        window_y,
        card_x,
        card_y);

    let mut display_state
        = self.display_state.write().unwrap();
    let new_card = self.create_ui_card(card.clone(), None);
    new_card.set_visible(true);

    let mut drag_display_state = DraggedCardDisplayState::new(
        new_card,
        window_x - card_x,
        window_y - card_y,
        display_state.supply_card_width().clone(),
        display_state.supply_card_height().clone(),
        card_x,
        card_y);

    display_state.set_card_in_flight(Some(drag_display_state));
  }

  /// Handle the movement of a drag while a card is in flight.  This will
  /// handle the determination of where within the cards in play the card
  /// in flight will be placed
  fn on_drag_move(&self, drag_x: f64, drag_y: f64) {
    info!("Drag moved: {}, {}", drag_x, drag_y);
    let mut display_state
        = self.display_state.write().unwrap();

    if let Some(drag_state) = display_state.card_in_flight_mut() {
      drag_state.drag_move(drag_x, drag_y);
    }
  }

  /// Handles the positive release of a dragged supply card
  fn on_supply_card_drag_end(&self, drag_x: f64, drag_y: f64) {
    info!("Supply card drag end: {}, {}", drag_x, drag_y);

    let mut display_state
        = self.display_state.write().unwrap();
    {
      let drag_state_opt = display_state.card_in_flight_mut();

      if let Some(drag_state) = drag_state_opt {
        let left = drag_state.left_orig().clone();
        let top= drag_state.top_orig().clone();
        let width = drag_state.width_orig().clone();
        let height = drag_state.height_orig().clone();

        let card  = drag_state.card();

        card.set_location_and_size_animated(left, top, width, height, 0.3);
      }
    }

    display_state.set_card_in_flight(None);

  }

  /// Update the display state of the presenter to match the given state.
  /// This completely rewrites the display state, so this should only be
  /// used when starting a new game
  fn initialize_game_state(
      _self: Arc<GamePresenter<V,S>>,
      game_state: GameState) {

    let mut new_display_state = GameDisplayState::default();

    *new_display_state.cards_in_play_mut()
        = game_state.cards_in_play().iter()
            .map(|card| {
              _self.create_ui_card(card.clone(), None)
            })
            .collect();

    for card in game_state.setup().supply_cards() {
      let copied_self_start = _self.clone();
      let copied_self_move = _self.clone();
      let copied_self_end = _self.clone();
      let copied_card = card.clone();
      let ui_card = _self.create_ui_card(
          card.clone(),
          Some(create_drag_handler!(
              on_drag_start(wx, wy, lx, ly) {
                copied_self_start.on_supply_card_drag_start(
                    copied_card.clone(), wx, wy, lx, ly);
              },
              on_drag_move(wx, wy, _lx, _ly) {
                copied_self_move.on_drag_move(wx, wy);
              },
              on_drag_end(wx, wy, lx, ly) {
                copied_self_end.on_supply_card_drag_end(wx, wy);
              }
            )));
      new_display_state.supply_cards_mut().push(ui_card);
    }

    *_self.display_state.write()
        .expect("Failed to get write lock on display state")
            = new_display_state;

    *_self.game_state.write()
        .expect("Failed to get write lock on game state")
            = game_state;
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

    let result = GamePresenter{

      view: view,
      event_bus: event_bus,
      runtime_resources: runtime_resources,
      listener_registrations: Mutex::new(Vec::new()),
      handler_registrations: Mutex::new(Vec::new()),

      display_state: RwLock::new(GameDisplayState::default()),
      game_state: RwLock::new(GameState::default())
    };

    let game_state
        = GameState::new(GameSetup::simple_new(1, vec![4, 4, 4, 4]));

    let arc_result = result.bind();

    GamePresenter::initialize_game_state(arc_result.clone(), game_state);

    arc_result
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