
macro_rules! number_card {
  ($num:expr) => {
    Card::Number(Card::verify_number($num));
  };
}

macro_rules! define_cards {
  ($($op:ident),*) => {

    #[derive(Clone)]
    pub enum Card {
      Number(i64),
      $(
        $op,
      )*
    }

    $(
    macro_rules! $op {
      () => {
        Card::$op
      };
    }
    )*

    macro_rules! all_non_number_cards {
      () => {
        vec![$( $op!()),*]
      };
    }
  };
}

define_cards!(
  Plus,
  Minus,
  Times,
  Divide,
  ParenL,
  ParenR,
  ZeroPoint,
  Power,
  Radical,
  Inverse,
  Factorial
);

impl Card {
  pub fn verify_number(num: i64) -> i64 {
    if num > 9 || num < 1 {
      panic!("Number cards can only have a value between 1 and 9 (inclusive)");
    }
    num
  }
}