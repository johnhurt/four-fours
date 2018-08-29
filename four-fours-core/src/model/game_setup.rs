
use model::Card;

pub struct GameSetup {
  goal: i64,
  required_cards: Vec<Card>,
  card_stacks: Vec<Card>,
}

impl GameSetup {
  pub fn simple_new(goal: i64, required_cards: Vec<i64>) -> GameSetup {
    GameSetup::new(
        goal,
        required_cards.iter().map(|v| number_card!(*v)).collect(),
        all_non_number_cards!())
  }

  pub fn new(goal: i64, required_cards: Vec<Card>, card_stacks: Vec<Card>)
      -> GameSetup {
    GameSetup {
      goal: goal,
      required_cards: required_cards,
      card_stacks:card_stacks
    }
  }

  pub fn goal(&self) -> i64 {
    self.goal
  }

  pub fn required_cards(&self) -> &Vec<Card> {
    &self.required_cards
  }

  pub fn card_stacks(&self) -> &Vec<Card> {
    &self.card_stacks
  }
}