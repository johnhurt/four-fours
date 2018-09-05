
use ui::{
  HasLayoutHandlers,
  SpriteSource
};

pub trait GameView : SpriteSource + HasLayoutHandlers + 'static {

}