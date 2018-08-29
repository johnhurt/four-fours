use model::{Card};
use native::{ Texture, Textures };
use ui::{
  Sprite,
  SpriteSource
};

const SYMBOL_WIDTH_FRAC : f64 = 0.6;

pub struct UiCard<S>
    where
        S: Sprite {
  card_sprite: S,
  symbol_sprite: S,
  symbol_texture_aspect_ratio: f64
}

impl <T,S> UiCard<S>
    where
        T: Texture,
        S: Sprite<T = T> {

  pub fn new(
      card: &Card,
      textures: &Textures<T>,
      sprite_source: &SpriteSource<T = T, S = S>)
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
      Card::Plus => textures.symbols().plus(),
      Card::Minus => textures.symbols().minus()
    };

    symbol_sprite.set_texture(symbol_texture);

    UiCard {
      card_sprite: card_sprite,
      symbol_sprite: symbol_sprite,
      symbol_texture_aspect_ratio: symbol_texture.get_aspect_ratio()
    }

  }

  pub fn set_location_and_size(&self,
      left: f64, top: f64, width: f64, height: f64) {
    self.card_sprite.set_size(width.round() as i64, height.round() as i64);
    self.card_sprite.set_location(left.round() as i64, top.round() as i64);

    let sym_ar = self.symbol_texture_aspect_ratio;

    let sym_width = width * SYMBOL_WIDTH_FRAC;
    let sym_height = sym_width / sym_ar;
    let sym_left = left + width / 2.0 - sym_width / 2.0;
    let sym_top = top + height / 2.0 - sym_height / 2.0;

    self.symbol_sprite.set_size(
        sym_width.round() as i64,
        sym_height.round() as i64);
    self.symbol_sprite.set_location(
        sym_left.round() as i64,
        sym_top.round() as i64);
  }

}
