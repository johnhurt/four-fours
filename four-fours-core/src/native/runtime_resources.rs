
use native::{SystemView, Textures};

pub struct RuntimeResources<S: SystemView> {
  textures: Textures<S::TL>
}

impl <S: SystemView> RuntimeResources<S> {
  pub fn new(textures: Textures<S::TL>) -> RuntimeResources<S> {
    RuntimeResources {
      textures: textures
    }
  }
}