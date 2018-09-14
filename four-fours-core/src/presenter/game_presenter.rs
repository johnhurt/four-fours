
use std::sync::{Mutex,Arc,RwLock};

use event::{
  EventBus,
  EventListener,
  ListenerRegistration,
  FourFoursEvent,
  Layout
};

use model::{
  BindPoint,
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

const MIN_SUPPLY_CARD_WIDTH_PTS : f64 = 45.0;
const MIN_PLAY_CARD_WIDTH_PTS : f64 = 35.0;

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

    display_state.set_width(width);
    display_state.set_height(height);
    display_state.set_card_aspect_ratio(card_aspect_ratio);

    let max_card_height = height * MAX_CARD_HEIGHT_FRAC;
    let max_card_width = (card_aspect_ratio * max_card_height).min(
        width * MAX_CARD_WIDTH_FRAC);

    let border_thickness = width * BOUNDARY_FRACTION;
    let playing_area_width = width - 2.0 * border_thickness;

    let supply_card_count = game_state.setup().supply_cards().len() as f64;

    let supply_card_width = MIN_SUPPLY_CARD_WIDTH_PTS.max(max_card_width.min(
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

    let supply_width
        = supply_cards_per_row.min(supply_card_count) * supply_card_width
            + (supply_cards_per_row - 1.) * supply_card_spacing;

    let supply_card_height = supply_card_width / card_aspect_ratio;

    let supply_height = supply_row_count * supply_card_height
        + (supply_row_count - 1.0) *supply_card_spacing;

    let supply_card_height = supply_card_width / card_aspect_ratio;
    let first_supply_card_top = height - border_thickness - supply_height;
    let first_supply_card_left = width / 2.0 - supply_width / 2.0;

    display_state.set_border_thickness(border_thickness);
    display_state.set_supply_card_width(supply_card_width);
    display_state.set_supply_card_height(supply_card_height);
    display_state.set_supply_card_spacing(supply_card_spacing);

    display_state.set_supply_row_count(supply_row_count);
    display_state.set_supply_left(first_supply_card_left);
    display_state.set_supply_top(first_supply_card_top);
    display_state.set_supply_width(supply_width);
    display_state.set_supply_height(supply_height);

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
    self.layout_play_area_cards(&mut *display_state, 0.);
  }
}

impl <V,S> GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {

  fn layout_play_area_cards(&self,
      display_state: &mut GameDisplayState<V::S>,
      animation_time_secs: f64) {

    let drag_card_ord = match display_state.card_in_flight() {
      Some(drag_state) => {
        drag_state.play_area_ord().as_ref().cloned()
      }
      _ => None
    };

    let width = display_state.width().clone();
    let height = display_state.height().clone();
    let card_aspect_ratio = display_state.card_aspect_ratio().clone();
    let border_thickness = display_state.border_thickness().clone();
    let supply_top = display_state.supply_top().clone();

    let available_play_area_width = width - 2.0 * border_thickness;
    let max_card_height = height * MAX_CARD_HEIGHT_FRAC;
    let max_card_width = (card_aspect_ratio * max_card_height).min(
        available_play_area_width * MAX_CARD_WIDTH_FRAC);

    let play_card_count
        = display_state.cards_in_play().len() as f64
        + if drag_card_ord.is_some() { 1. } else { 0. };

    let play_card_width = MIN_PLAY_CARD_WIDTH_PTS.max(max_card_width.min(
          available_play_area_width / ( play_card_count
              + (play_card_count - 1.0) * SPACING_FRACTION )));

    let min_play_card_spacing = SPACING_FRACTION * play_card_width;

    let play_cards_per_row
        = ((available_play_area_width + min_play_card_spacing)
            / (play_card_width + min_play_card_spacing)).floor();

    let play_area_row_count = (play_card_count / play_cards_per_row).ceil();

    let play_card_spacing
        = if play_cards_per_row > 1.0 {
          (available_play_area_width - play_cards_per_row * play_card_width)
              / (play_cards_per_row - 1.0)
        }
        else {
          min_play_card_spacing
        };

    let play_area_width
        = play_cards_per_row.min(play_card_count) * play_card_width
            + (play_cards_per_row - 1.) * play_card_spacing;

    let play_card_height = play_card_width / card_aspect_ratio;

    let play_area_height = play_area_row_count * play_card_height
        + (play_area_row_count - 1.0) * play_card_spacing;

    let play_card_height = play_card_width / card_aspect_ratio;
    let first_play_card_top
        = (supply_top - 2.0 * border_thickness) / 2.0
            - play_area_height / 2.0 + border_thickness;
    let first_play_card_left = width / 2.0 - play_area_width / 2.0;

    let mut bind_points : Vec<(f64,Vec<(f64,BindPoint)>)> = Vec::default();

    for i in 0..(play_card_count as usize) {
      let row : usize = i / (play_cards_per_row as usize);
      let col : usize = i % (play_cards_per_row as usize);
      let card_index_opt = match drag_card_ord {
        None => Some(i),
        Some(drag_ord) => {
          if i != drag_ord {
            Some(i - if drag_ord < i {1} else {0})
          }
          else { None }
        }
      };

      let play_card_top = first_play_card_top
          + (play_card_height + play_card_spacing)
          * (row as f64);

      let play_card_left = first_play_card_left
          + (play_card_width + play_card_spacing)
          * (col as f64);

      let bind_point_x = play_card_left + play_card_width / 2.;
      let bind_point_y = play_card_top + play_card_height / 2.;

      if bind_points.len() < (row + 1) {
        bind_points.push((bind_point_y, Vec::default()));
      }

      let (_, col_points) = bind_points.get_mut(row).unwrap();

      col_points.push((bind_point_x, BindPoint::new(i, bind_point_x)));

      card_index_opt.map(|card_index| {
        let card_opt = display_state.cards_in_play_mut().get_mut(card_index);
        match card_opt {
          Some(card) => {
            card.set_location_and_size_animated(
                play_card_left,
                play_card_top,
                play_card_width,
                play_card_height,
                animation_time_secs);
            card.set_visible(true);
          },
          _ => ()
        }
      });
    }

    display_state.set_play_card_width(play_card_width);
    display_state.set_play_card_height(play_card_height);
    display_state.set_play_card_spacing(play_card_spacing);

    display_state.set_play_area_row_count(play_area_row_count);
    display_state.set_play_area_left(first_play_card_left);
    display_state.set_play_area_top(first_play_card_top);
    display_state.set_play_area_width(play_area_width);
    display_state.set_play_area_height(play_area_height);

    display_state.set_bind_points(bind_points);

  }

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
  fn create_ui_card(&self,
      card: Card,
      required_play_card: bool,
      drag_handler: Option<DragHandler>) -> UiCard<V::S> {
    UiCard::new(
        card,
        required_play_card,
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
    let new_card = self.create_ui_card(card.clone(), false, None);
    new_card.set_visible(true);

    let drag_display_state = DraggedCardDisplayState::new(
        new_card,
        window_x - card_x,
        window_y - card_y,
        display_state.supply_card_width().clone(),
        display_state.supply_card_height().clone(),
        card_x,
        card_y);

    display_state.set_card_in_flight(Some(drag_display_state));
  }

  /// Calculate the width that the given dragged card should be based on its
  /// location and the overall display state
  fn get_dragged_card_width(&self,
      card_center: &(f64, f64),
      display_state: &GameDisplayState<V::S>)
          -> f64 {
    let y = card_center.1;

    let max_y = *display_state.supply_top()
        + *display_state.supply_card_height() / 2.;
    let min_y = *display_state.play_area_top()
        + *display_state.play_area_height();
    let clipped_y = min_y.max(max_y.min(y));
    let normalized_y = 1.0 - (clipped_y - min_y) / (max_y - min_y).max(1.);

    *display_state.supply_card_width() + normalized_y
        * (*display_state.play_card_width()
            - *display_state.supply_card_width())
  }

  /// Handle the movement of a drag while a card is in flight.  This will
  /// handle the determination of where within the cards in play the card
  /// in flight will be placed
  fn on_drag_move(&self, drag_x: f64, drag_y: f64) {
    let mut display_state
        = self.display_state.write().unwrap();

    let mut new_drag_ord_opt = None;
    let mut old_drag_ord_opt = None;
    let mut new_drag_card_width = 0.;

    if let Some(drag_state) = display_state.card_in_flight() {
      let card_center = drag_state.card_center();
      new_drag_card_width = self.get_dragged_card_width(
          &card_center,
          &*display_state);

      old_drag_ord_opt = drag_state.play_area_ord().clone();

      new_drag_ord_opt = display_state
          .get_closest_bind_point(drag_x, drag_y)
          .map(|bp| {
            bp.index().clone()
          });
    }

    // Change in drag state order
    if new_drag_ord_opt != old_drag_ord_opt {
      info!("Drag ord changed from {:?} to {:?}",
          old_drag_ord_opt,
          new_drag_ord_opt);
      self.on_dragged_card_ordinal_changed(
          &mut *display_state,
          &new_drag_ord_opt);
    }

    match display_state.card_in_flight_mut() {
      Some(drag_state) => {
        drag_state.drag_move(drag_x, drag_y);
        drag_state.scale_width_to(new_drag_card_width);
      },
      _ => ()
    }
  }

  /// Handles the event where a card being dragged's ordinal in the play
  /// area changes. This will cause the played cards to shift around an
  /// opening that will (probably) be close to where the card is being
  /// dragged.  If the new drag ordinal not present, then the opening that
  /// was present will close
  fn on_dragged_card_ordinal_changed(&self,
      display_state: &mut GameDisplayState<V::S>,
      new_drag_ordinal: &Option<usize>) {

    match display_state.card_in_flight_mut().as_mut() {
      Some(drag_state) => {
        drag_state.set_play_area_ord(new_drag_ordinal.clone());
      }
      _ => ()
    };

    self.layout_play_area_cards(display_state, 0.1);
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

        card.set_location_and_size_animated(left, top, width, height, 0.1);
      }
    }

    display_state.set_card_in_flight(None);

  }

  /// Initialize the display state with the initial game state
  fn initialize_game_state(
      this: Arc<GamePresenter<V,S>>,
      game_state: GameState) {

    let mut new_display_state = GameDisplayState::default();

    for card in game_state.cards_in_play() {
      let copied_self_start = this.clone();
      let copied_self_move = this.clone();
      let copied_self_end = this.clone();
      let copied_card = card.clone();
      this.create_ui_card(card.clone(),
          copied_card.is_required_in_play(),
          Some(create_drag_handler!(
              on_drag_start(wx, wy, lx, ly) {
                copied_self_start.on_supply_card_drag_start(
                    copied_card.clone(), wx, wy, lx, ly);
              },
              on_drag_move(wx, wy, _lx, _ly) {
                copied_self_move.on_drag_move(wx, wy);
              },
              on_drag_end(wx, wy, _lx, _ly) {
                copied_self_end.on_supply_card_drag_end(wx, wy);
              }
          )));
    }

    *new_display_state.cards_in_play_mut()
        = game_state.setup().required_cards().iter()
            .map(|card| {
              this.create_ui_card(card.clone(), true, None)
            })
            .collect();

    for card in game_state.setup().supply_cards() {
      let copied_self_start = this.clone();
      let copied_self_move = this.clone();
      let copied_self_end = this.clone();
      let copied_card = card.clone();
      let ui_card = this.create_ui_card(
          card.clone(),
          false,
          Some(create_drag_handler!(
              on_drag_start(wx, wy, lx, ly) {
                copied_self_start.on_supply_card_drag_start(
                    copied_card.clone(), wx, wy, lx, ly);
              },
              on_drag_move(wx, wy, _lx, _ly) {
                copied_self_move.on_drag_move(wx, wy);
              },
              on_drag_end(wx, wy, _lx, _ly) {
                copied_self_end.on_supply_card_drag_end(wx, wy);
              }
          )));
      new_display_state.supply_cards_mut().push(ui_card);
    }

    *this.display_state.write()
        .expect("Failed to get write lock on display state")
            = new_display_state;

    *this.game_state.write()
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