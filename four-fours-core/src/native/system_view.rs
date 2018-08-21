

use native::{
    TextureLoader
};

pub trait SystemView : 'static + Sized {
  type TL : TextureLoader;

  fn get_texture_loader(&self) -> Self::TL;
}
