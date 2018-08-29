use native::{
  SystemView
};
use ui::{
  HasLayoutHandlers,
  Sprite,
  SpriteSource
};

pub trait GameView : SpriteSource + HasLayoutHandlers + 'static {

}