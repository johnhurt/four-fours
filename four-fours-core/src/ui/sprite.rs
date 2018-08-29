use native::Texture;

use ui::{
  HasMutableSize,
  HasMutableLocation
};

pub trait Sprite : HasMutableSize + HasMutableLocation + 'static {
  type T : Texture;

  fn set_texture(&self, texture: &Self::T);
}