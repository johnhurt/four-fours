
use std::sync::{
  Mutex,
  Arc,
  RwLock
};

use std::sync::atomic::{
  AtomicIsize
};

use std::{
  thread
};

use std::time::{
  Instant,
  Duration
};

use event::{
  EventBus,
  EventListener,
  ListenerRegistration,
  FourFoursEvent,
  Layout,
  Evaluate
};

use math::{
  MathEngine,
  MathResponse
};

use model::{
  Point,
  Size,
  Rect,
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

lazy_static!{
  static ref MIN_EVAL_SEPARATION : Duration = Duration::from_millis(500);
}

const BOUNDARY_FRACTION : f64 = 0.04;
const SPACING_FRACTION : f64 = 0.04;
const MAX_CARD_WIDTH_FRAC : f64 = 0.2;
const MAX_CARD_HEIGHT_FRAC : f64 = 0.35;

const MIN_SUPPLY_CARD_WIDTH_PTS : f64 = 45.0;
const MIN_PLAY_CARD_WIDTH_PTS : f64 = 35.0;

const TEX_AREA_HEIGHT_FRAC : f64 = 0.3;

pub struct GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  view: V,
  event_bus: Arc<EventBus>,
  runtime_resources: Arc<RuntimeResources<S>>,
  listener_registrations: Mutex<Vec<ListenerRegistration>>,
  handler_registrations: Mutex<Vec<Box<HandlerRegistration>>>,

  display_state: RwLock<GameDisplayState<V::S>>,
  math_engine: MathEngine,
  goal: AtomicIsize,

  last_eval: Mutex<Option<String>>,
  eval_queue: Mutex<Option<String>>,
  next_eval_time: Mutex<Option<Instant>>,
  last_eval_time: Mutex<Instant>
}

impl <V,S> EventListener<Evaluate> for GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  fn on_event(&self, _: &Evaluate) {
    let mut to_eval_opt : Option<String> = None;

    {
      to_eval_opt
          = self.eval_queue.lock().expect("Failed to lock eval queue").take();
    }

    info!("Handling evaluate event {:?}", to_eval_opt);
    match to_eval_opt {
      Some(to_eval) => {
        match (self.math_engine.evaluate(&to_eval)) {
          Ok(resp) => {
            info!("{} = {}", resp.tex, resp.value);
          },
          _ => { info!("Failed to parse as math"); }
        }
      }
      _ => ()
    }

  }
}

impl <V,S> EventListener<Layout> for GamePresenter<V,S>
    where
        S: SystemView,
        V: GameView<T = S::T> {
  fn on_event(&self, event: &Layout) {
    info!("Game view resized to : {}, {}", event.width, event.height);

    let mut display_state = self.display_state.write()
        .expect("Failed to lock display state for reading");

    let width = event.width as f64;
    let height = event.height as f64;

    let card_aspect_ratio
        = self.runtime_resources.textures().card().get_aspect_ratio();

    display_state.size_mut().width = width;
    display_state.size_mut().height = height;
    display_state.set_card_aspect_ratio(card_aspect_ratio);

    let max_card_height = height * MAX_CARD_HEIGHT_FRAC;
    let max_card_width = (card_aspect_ratio * max_card_height).min(
        width * MAX_CARD_WIDTH_FRAC);

    let border_thickness = width * BOUNDARY_FRACTION;
    let playing_area_width = width - 2.0 * border_thickness;

    let supply_card_count = display_state.supply_cards().len() as f64;

    let supply_card_width = MIN_SUPPLY_CARD_WIDTH_PTS.max(max_card_width.min(
          playing_area_width / ( supply_card_count
              + (supply_card_count - 1.0) * SPACING_FRACTION )));

    let min_supply_card_spacing = SPACING_FRACTION * supply_card_width;

    let supply_cards_per_row
        = ((playing_area_width + min_supply_card_spacing)
            / (supply_card_width + min_supply_card_spacing) + 0.1).floor();

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
    display_state.supply_card_size_mut().width = supply_card_width;
    display_state.supply_card_size_mut().height = supply_card_height;
    display_state.set_supply_card_spacing(supply_card_spacing);
    display_state.set_supply_cards_per_row(supply_cards_per_row);

    display_state.set_supply_row_count(supply_row_count);

    {
      let supply_rect = display_state.supply_rect_mut();
      supply_rect.top_left.x = first_supply_card_left;
      supply_rect.top_left.y = first_supply_card_top;
      supply_rect.size.width = supply_width;
      supply_rect.size.height = supply_height;
    }
    {
      let mut game_area_rect = Rect::default();
      game_area_rect.top_left.x = border_thickness;
      game_area_rect.top_left.y = border_thickness;
      game_area_rect.size.width = width - border_thickness * 2.;
      game_area_rect.size.height
          = first_supply_card_top - border_thickness * 2.;
      display_state.set_game_area_rect(game_area_rect.clone());

      let mut card_area_rect = display_state.game_area_rect().clone();
      card_area_rect.top_left.y = game_area_rect.top_left.y + border_thickness
          + game_area_rect.size.height * TEX_AREA_HEIGHT_FRAC;
      card_area_rect.size.height = (1. - TEX_AREA_HEIGHT_FRAC)
          * (game_area_rect.size.height - border_thickness);
      display_state.set_card_area_rect(card_area_rect);

      let mut tex_area_rect = display_state.game_area_rect().clone();
      tex_area_rect.size.height = TEX_AREA_HEIGHT_FRAC
          * (game_area_rect.size.height - border_thickness);
      display_state.set_tex_area_rect(tex_area_rect);
    }

    display_state
        .supply_cards()
        .iter()
        .enumerate()
        .for_each(|(i, card)| {

          info!("index: {}", i);
          card.set_rect(
              &display_state.get_supply_card_rect_by_index(&i).unwrap());
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

    let height = display_state.size().height;
    let card_aspect_ratio = display_state.card_aspect_ratio().clone();
    let card_area_rect = display_state.card_area_rect().clone();
    let card_area_center = card_area_rect.center();

    let available_play_area_width = card_area_rect.size.width;
    let max_card_height = height * MAX_CARD_HEIGHT_FRAC;
    let max_card_width = (card_aspect_ratio * max_card_height).min(
        available_play_area_width * MAX_CARD_WIDTH_FRAC);

    let play_card_slots
        = display_state.cards_in_play().len() as f64
        + if drag_card_ord.is_some() { 1. } else { 0. };

    let natural_card_width = available_play_area_width
        / ( play_card_slots + (play_card_slots - 1.0) * SPACING_FRACTION );

    let play_card_width = MIN_PLAY_CARD_WIDTH_PTS.max(
        max_card_width.min(natural_card_width));

    let min_play_card_spacing = SPACING_FRACTION * play_card_width;

    let play_cards_per_row
        = ((available_play_area_width + min_play_card_spacing)
            / (play_card_width + min_play_card_spacing) + 0.1).floor();

    let play_area_row_count = (play_card_slots / play_cards_per_row).ceil();

    let play_card_spacing
        = if play_cards_per_row > 1.0 {
          (available_play_area_width - play_cards_per_row * play_card_width)
              / (play_cards_per_row - 1.0)
        }
        else {
          min_play_card_spacing
        };

    let play_area_width
        = play_cards_per_row.min(play_card_slots) * play_card_width
            + (play_cards_per_row - 1.) * play_card_spacing;

    let play_card_height = play_card_width / card_aspect_ratio;

    let play_area_height = play_area_row_count * play_card_height
        + (play_area_row_count - 1.0) * play_card_spacing;

    let play_card_height = play_card_width / card_aspect_ratio;
    let first_play_card_top = card_area_center.y - play_area_height / 2.0;
    let first_play_card_left = card_area_center.x - play_area_width / 2.0;

    display_state.play_card_size_mut().width = play_card_width;
    display_state.play_card_size_mut().height = play_card_height;
    display_state.set_play_card_spacing(play_card_spacing);
    display_state.set_play_cards_per_row(play_cards_per_row);
    display_state.set_play_card_slots(play_card_slots);

    display_state.set_play_area_row_count(play_area_row_count);
    display_state.play_area_rect_mut().top_left.x = first_play_card_left;
    display_state.play_area_rect_mut().top_left.y = first_play_card_top;
    display_state.play_area_rect_mut().size.width = play_area_width;
    display_state.play_area_rect_mut().size.height = play_area_height;

    let mut bind_points : Vec<(f64,Vec<(f64,usize)>)> = Vec::default();

    for i in 0..(play_card_slots as usize) {
      let row : usize = i / (play_cards_per_row as usize);

      let card_index_opt = match drag_card_ord {
        None => Some(i),
        Some(drag_ord) => {
          if i != drag_ord {
            Some(i - if drag_ord < i {1} else {0})
          }
          else { None }
        }
      };

      let play_card_rect
          = display_state.get_play_card_rect_by_index(&i).unwrap();

      let bind_point = play_card_rect.center();

      if bind_points.len() < (row + 1) {
        bind_points.push((bind_point.y, Vec::default()));
      }

      let (_, col_points) = bind_points.get_mut(row).unwrap();

      col_points.push((bind_point.x, i));

      card_index_opt.map(|card_index| {
        let card_opt = display_state.cards_in_play_mut().get_mut(card_index);
        match card_opt {
          Some(card) => {
            card.set_rect_animated(&play_card_rect, animation_time_secs);
            card.set_visible(true);
          },
          _ => ()
        }
      });
    }

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

  fn on_play_card_drag_start(&self,
      ord: usize,
      drag_point_in_card: &Point,
      card_rect: &Rect,
      display_state: &mut GameDisplayState<V::S>)
          -> DraggedCardDisplayState<V::S> {

    info!("Dragging play card from {}", ord);

    let ui_card = display_state.cards_in_play_mut().remove(ord);

    DraggedCardDisplayState::new(
        ui_card,
        Some(ord),
        card_rect,
        drag_point_in_card)
  }

  /// Create a dragged card drag state using the given ui card as a sead
  fn on_supply_card_drag_start(&self,
      card: &Card,
      drag_point_in_card: &Point,
      card_rect: &Rect) -> DraggedCardDisplayState<V::S> {
    let new_ui_card = self.create_ui_card(card, None, false);

    let result = DraggedCardDisplayState::new(
        new_ui_card,
        None,
        card_rect,
        drag_point_in_card);

    result.card().set_visible(true);

    result
  }

  /// Creates a ui card based on the given card
  fn create_ui_card(&self,
      card: &Card,
      play_area_ord: Option<usize>,
      required_play_card: bool) -> UiCard<V::S> {
    UiCard::new(
        card.clone(),
        play_area_ord,
        required_play_card,
        self.runtime_resources.textures(),
        &self.view)
  }

  /// handle the initialization of a card being dragged.
  fn on_drag_start(&self, drag_point: &Point) {

    let mut display_state
        = self.display_state.write().unwrap();

    let drag_state = match display_state.get_card_at(drag_point) {
      Some(card_find_response) => {
        match card_find_response.play_area_ord {
          Some(ord) => {
            Some(self.on_play_card_drag_start(ord,
                &card_find_response.point_in_card,
                &card_find_response.card_rect,
                &mut *display_state))
          },
          None => {
            Some(self.on_supply_card_drag_start(
                &card_find_response.card,
                &card_find_response.point_in_card,
                &card_find_response.card_rect))
          }
        }
      },
      _ => None
    };

    display_state.set_card_in_flight(drag_state);
  }

  /// Calculate the width that the given dragged card should be based on its
  /// location and the overall display state
  fn get_dragged_card_width(&self,
      card_center: &Point,
      display_state: &GameDisplayState<V::S>)
          -> f64 {
    let y = card_center.y;

    let max_y = display_state.supply_rect().top_left.y
        + display_state.supply_card_size().height;
    let min_y = display_state.play_area_rect().top_left.y
        + display_state.play_area_rect().size.height;
    let clipped_y = min_y.max(max_y.min(y));
    let normalized_y = 1.0 - (clipped_y - min_y) / (max_y - min_y).max(1.);

    display_state.supply_card_size().width + normalized_y
        * (display_state.play_card_size().width
            - display_state.supply_card_size().width)
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
          .get_closest_bind_point(
              &Point::new(drag_x, drag_y),
              &old_drag_ord_opt);
    }

    // Change in drag state order
    if new_drag_ord_opt != old_drag_ord_opt {
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

  /// Enqueue the string representation of the given game state in the
  /// evaluation queue and post an eval event
  fn trigger_evaluation(&self, display_state: &GameDisplayState<V::S>) {
    {
      let mut q_lock = self.eval_queue.lock()
          .expect("Failed to lock event queue");

      *q_lock = Some(display_state.to_string());
    }

    self.event_bus.post(Evaluate{});
  }

  /// Called when a card is released into play
  fn on_card_dropped_into_play(&self,
      display_state: &mut GameDisplayState<V::S>,
      drag_ord: usize,
      drag_state: DraggedCardDisplayState<V::S>) {
    let ui_card = drag_state.take_card();
    display_state.cards_in_play_mut().insert(drag_ord, ui_card);
  }

  /// Called when a card is released but not released into play
  fn on_card_dropped_not_in_play(&self,
      display_state: &mut GameDisplayState<V::S>,
      drag_state: DraggedCardDisplayState<V::S>) {

    if drag_state.card().required_play_card().clone() {
      let ord = drag_state.orig_play_area_ord().as_ref().cloned();
      let card = drag_state.take_card();
      display_state.cards_in_play_mut().insert(ord.unwrap(), card);
    }
    else {
      if let Some(return_rect)
          = display_state.get_supply_card_rect_by_card(
              drag_state.card().card()) {
        drag_state.card().set_rect_animated(&return_rect, 0.1)
      }
    }
  }

  /// Handles the positive release of a dragged supply card
  fn on_drag_end(&self) {

    let mut display_state
        = self.display_state.write().unwrap();

    match display_state.card_in_flight_mut().take() {
      Some(mut drag_state) => {
        match drag_state.play_area_ord_mut().take() {
          Some(drag_ord) => {
            self.on_card_dropped_into_play(
                &mut *display_state,
                drag_ord,
                drag_state);
          }
          None => {
            self.on_card_dropped_not_in_play(
                &mut *display_state,
                drag_state);
          }
        }

      },
      _ => ()
    };

    self.trigger_evaluation(&display_state);
    self.layout_play_area_cards(&mut *display_state, 0.1);
  }

  /// Initialize the display state with the initial game state
  fn initialize_game_state(
      this: Arc<GamePresenter<V,S>>,
      game_state: GameState) {

    let mut new_display_state = GameDisplayState::default();

    *new_display_state.cards_in_play_mut() = game_state
        .cards_in_play()
        .iter()
        .enumerate()
        .map(|(i, card)| {
          this.create_ui_card(card,
          Some(i),
          card.is_required_in_play())
        })
        .collect();

    *new_display_state.supply_cards_mut()
        = game_state.setup().supply_cards().iter()
            .map(|card| {
              this.create_ui_card(card, None, true)
            })
            .collect();

    *this.display_state.write()
        .expect("Failed to get write lock on display state")
            = new_display_state;
  }

  fn bind(self) -> Arc<GamePresenter<V,S>> {
    let copied_event_bus = self.event_bus.clone();

    self.add_handler_registration(Box::new(self.view
        .add_layout_handler(create_layout_handler!(|w, h| {
            copied_event_bus.post(Layout{width: w, height: h})
        }))));

    let result = Arc::new(self);
    let result_drag_start = result.clone();
    let result_drag_move = result.clone();
    let result_drag_end = result.clone();

    result.add_handler_registration(Box::new(result.view
        .add_drag_handler(create_drag_handler!(
              on_drag_start(wx, wy, _lx, _ly) {
                result_drag_start.on_drag_start(&Point { x: wx, y: wy });
              },
              on_drag_move(wx, wy, _lx, _ly) {
                result_drag_move.on_drag_move(wx, wy);
              },
              on_drag_end(_wx, _wy, _lx, _ly) {
                result_drag_end.on_drag_end();
              }
          ))));

    result.add_listener_registration(
        result.event_bus.register_disambiguous(
            FourFoursEvent::Layout,
            &result,
            Some(Layout{width: 0, height: 0})));

    result.add_listener_registration(
        result.event_bus.register_disambiguous(
            FourFoursEvent::Evaluate,
            &result,
            Some(Evaluate{})));
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
      math_engine: MathEngine{},

      goal: AtomicIsize::new(0),

      last_eval: Mutex::new(None),
      eval_queue: Mutex::new(None),
      last_eval_time: Mutex::new(Instant::now()),
      next_eval_time: Mutex::new(None)
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