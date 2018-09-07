use model::{Card};
use native::{ Texture, Textures };
use ui::{
  Sprite,
  SpriteSource,
  HasMutableVisibility,
  DragHandler
};

const SYMBOL_WIDTH_FRAC : f64 = 0.6;

pub struct UiCard<S>
    where
        S: Sprite {
  card_sprite: S,
  symbol_sprite: S,
  symbol_texture_aspect_ratio: f64,
  card: Card,
  _drag_handler_registration: Option<S::R>,
}

impl <T,S> UiCard<S>
    where
        T: Texture,
        S: Sprite<T = T> {

  pub fn new(
      card: Card,
      textures: &Textures<T>,
      sprite_source: &SpriteSource<T = T, S = S>,
      drag_handler_opt: Option<DragHandler>)
          -> UiCard<S> {

    let card_sprite = sprite_source.create_sprite();
    let symbol_sprite = sprite_source.create_sprite();

    card_sprite.set_texture(textures.card());

    let symbol_texture = match card {
      Card::Number(val) => {
        match val {
          1 => textures.symbols().one(),
          2 => textures.symbols().two(),
          3 => textures.symbols().three(),
          4 => textures.symbols().four(),
          5 => textures.symbols().five(),
          6 => textures.symbols().six(),
          7 => textures.symbols().seven(),
          8 => textures.symbols().eight(),
          9 => textures.symbols().nine(),
          _ => panic!("Invalid number {}", val)
        }
      },
      Card::ZeroPoint => textures.symbols().zero_point(),
      Card::Plus => textures.symbols().plus(),
      Card::Minus => textures.symbols().minus(),
      Card::Times => textures.symbols().times(),
      Card::Divide => textures.symbols().divide(),
      Card::Power => textures.symbols().power(),
      Card::Radical => textures.symbols().radical(),
      Card::ParenL => textures.symbols().paren_l(),
      Card::ParenR => textures.symbols().paren_r(),
      Card::Inverse => textures.symbols().inverse(),
      Card::Factorial => textures.symbols().factorial()
    };

    symbol_sprite.set_texture(symbol_texture);

    if drag_handler_opt.is_some() {
      symbol_sprite.propagate_events_to(&card_sprite);
    }

    UiCard {
      card: card,
      _drag_handler_registration: drag_handler_opt
          .map(|drag_handler| (&card_sprite).add_drag_handler(drag_handler)),
      card_sprite: card_sprite,
      symbol_sprite: symbol_sprite,
      symbol_texture_aspect_ratio: symbol_texture.get_aspect_ratio()
    }

  }

  /// Set the location and size of this card immediately.  This will set the
  /// location and shape of the card background sprite and the symbol sprite
  /// of this card will be aspet scaled to fit in the middle
  pub fn set_location_and_size(&self,
      left: f64, top: f64, width: f64, height: f64) {
    self.set_location_and_size_animated(left, top, width, height, 0.);
  }

  /// Animate the movement of this card from its current location to the
  /// given location after the given number of seconds.  This will set the
  /// location and shape of the card background sprite and the symbol sprite
  /// of this card will be aspet scaled to fit in the middle
  pub fn set_location_and_size_animated(&self,
      left: f64, top: f64, width: f64, height: f64, duration_seconds: f64) {
    self.card_sprite.set_size_animated(width, height, duration_seconds);
    self.card_sprite.set_location_animated(left, top, duration_seconds);

    let sym_ar = self.symbol_texture_aspect_ratio;

    let sym_width = width * SYMBOL_WIDTH_FRAC;
    let sym_height = sym_width / sym_ar;
    let sym_left = left + width / 2.0 - sym_width / 2.0;
    let sym_top = top + height / 2.0 - sym_height / 2.0;

    self.symbol_sprite.set_size_animated(
        sym_width,
        sym_height,
        duration_seconds);
    self.symbol_sprite.set_location_animated(
        sym_left,
        sym_top,
        duration_seconds);
  }
}

impl <S> HasMutableVisibility for UiCard<S> where S : Sprite {

  fn set_visible(&self, visible: bool) {
    self.card_sprite.set_visible(visible);
    self.symbol_sprite.set_visible(visible);
  }

}
