use nom;
use nom::{digit, multispace};
use nom::types::CompleteStr;

use num::{
  BigUint,
  One
};

use std::str;

use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use math::{
  Number,
  EvalExp,
  EvalProd,
  EvalProdTerm,
  Evaluable,
  EvalNode
};

#[derive(PartialEq)]
pub enum Expr {
  Num(String, Option<String>),
  Add(Box<Expr>, Box<Expr>),
  Sub(Box<Expr>, Box<Expr>),
  Mul(Box<Expr>, Box<Expr>),
  Div(Box<Expr>, Box<Expr>),
  Exp(Box<Expr>, Box<Expr>),
  Paren(Box<Expr>),
  Factorial(Box<Expr>),
  Radical(Box<Expr>)
}

#[derive(Debug)]
pub enum Oper {
  Add,
  Sub,
  Mul,
  Div
}

use self::Expr::*;

impl Display for Expr {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    use self::Expr::*;
    match *self {
      Num(ref prefix, ref repeat_opt) => {
        match repeat_opt {
          Some(repeat) => write!(format, "{}({})", prefix, repeat),
          None => write!(format, "{}", prefix)
        }
      },
      Add(ref left, ref right) => write!(format, "{} + {}", left, right),
      Sub(ref left, ref right) => write!(format, "{} - {}", left, right),
      Mul(ref left, ref right) => write!(format, "{} * {}", left, right),
      Div(ref left, ref right) => write!(format, "{} / {}", left, right),
      Paren(ref expr) => write!(format, "({})", expr),
      Exp(ref left, ref right) => write!(format, "{} ^ {}", left, right),
      Factorial(ref body) => write!(format, "{} !", body),
      Radical(ref body) => write!(format, "√ {}", body)
    }
  }
}

impl Debug for Expr {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    use self::Expr::*;
    match *self {
      Num(ref prefix, ref repeat_opt) => {
        match repeat_opt {
          Some(repeat) => write!(format, "{:?}({:?})", prefix, repeat),
          None => write!(format, "{:?}", prefix)
        }
      },
      Add(ref left, ref right) => write!(format, "({:?} + {:?})", left, right),
      Sub(ref left, ref right) => write!(format, "({:?} - {:?})", left, right),
      Mul(ref left, ref right) => write!(format, "({:?} * {:?})", left, right),
      Div(ref left, ref right) => write!(format, "({:?} / {:?})", left, right),
      Paren(ref expr) => write!(format, "[{:?}]", expr),
      Exp(ref left, ref right) => write!(format, "{:?} ^ {:?}", left, right),
      Factorial(ref body) => write!(format, "{:?} !", body),
      Radical(ref body) => write!(format, "√ {:?}", body)
    }
  }
}

#[macro_export]
macro_rules! complete_named (
  ($name:ident, $submac:ident!( $($args:tt)* )) => (
    fn $name( i: CompleteStr ) -> nom::IResult<CompleteStr, CompleteStr, u32> {
      $submac!(i, $($args)*)
    }
  );
  ($name:ident<$o:ty>, $submac:ident!( $($args:tt)* )) => (
    fn $name( i: CompleteStr ) -> nom::IResult<CompleteStr, $o, u32> {
      $submac!(i, $($args)*)
    }
  );
);

complete_named!(
  unsigned_float<(String,Option<String>)>,
  flat_map!(
    recognize!(alt!(
      delimited!(digit, tag!("."),
          tuple!(opt!(digit), tag!("("), digit, tag!(")")))
      | delimited!(opt!(digit), tag!("."),
          tuple!(opt!(digit), tag!("("), digit, tag!(")")))
      | delimited!(digit, tag!("."), opt!(digit))
      | delimited!(opt!(digit), tag!("."), digit)
      | digit
    )),
    parse_float
  )
);

complete_named!(
  float<(String,Option<String>)>,
  map!(
    pair!(opt!(alt!(tag!("+") | tag!("-"))), unsigned_float),
    |(sign, value): (Option<CompleteStr>, (String,Option<String>))| {
      let (prefix, repeat) = value;

      let pre_prefix = String::from(sign
          .and_then(|s| s.0.chars().next())
          .and_then(|c| if c == '-' { Some("-") } else { None })
          .unwrap_or(""));

      (pre_prefix + &prefix, repeat)
    }
  )
);

fn parentheses(inside: (Expr, Evaluable)) -> (Expr, EvalNode) {
  let (inside_expr, inside_eval) = inside;
  (Paren(Box::new(inside_expr)), EvalNode::from_statement(inside_eval))
}

named!(parens< CompleteStr, (Expr,EvalNode) >, delimited!(
    delimited!(opt!(multispace), tag!("("), opt!(multispace)),
    map!(expr, parentheses),
    delimited!(opt!(multispace), tag!(")"), opt!(multispace))
  )
);

named!(factor< CompleteStr, (Expr,EvalNode) >, alt_complete!(
  map!(
    delimited!(opt!(multispace), float, opt!(multispace)),
    to_num
  )
  | parens
  )
);

named!(possible_factorials_or_radicals< CompleteStr, (Expr,EvalNode) >,
  do_parse!(
    radicals: many0!(ws!(tag!("√"))) >>
    fac: factor >>
    exclamations: many0!(ws!(tag!("!"))) >>
    (wrap_radicals_and_factorials(radicals, fac, exclamations))
  )
);

fn to_num(tup: (String, Option<String>)) -> (Expr, EvalNode) {
  let (prefix, repeat) = tup;
  let number = Number::from_str(&prefix, &repeat);
  let num_node = EvalNode::Num(number);
  (Num(prefix, repeat), num_node)
}

fn parse_float(val: CompleteStr)
    -> nom::IResult<CompleteStr, (String,Option<String>)> {
  let decimal_idx_opt = val.0.find('.');

  let result = match val.0.find('(') {
    None => (String::from(val.0), None),
    Some(paren_idx) => {
      let prefix = val.get(0..paren_idx).unwrap();
      let repeat = val.get((paren_idx + 1)..(val.0.len() - 1)).unwrap();
      let prefix_places = paren_idx
          - decimal_idx_opt.clone().unwrap() - 1;
      let repeat_len = repeat.len();
      let repeats = ((16.0 - (prefix_places as f64))
          / ( repeat_len as f64 )).ceil() as usize;

      let mut result = String::new();
      result += prefix;

      for _ in 0..repeats {
        result += repeat;
      }

      (String::from(prefix), Some(String::from(repeat)))
    }
  };

  let (prefix, repeat) = result;

  let pre_prefix =
      decimal_idx_opt
          .map(|idx| if idx == 0 {"0"} else {""} )
          .unwrap_or("");

  Ok((CompleteStr(""), (String::from(pre_prefix) + &prefix, repeat)))
}

fn fold_plus_minus_expr(init: (Expr,EvalProd),
    remainder: Vec<(Oper,(Expr,EvalProd))>) -> (Expr, Evaluable) {

  let (init_expr, init_prod) = init;
  let mut init_sum = Evaluable::new();
  init_sum.push(init_prod);
  let init = (init_expr, init_sum);

  remainder.into_iter().fold(init, |acc, tuple| {
    let (expr_acc, mut eval_acc) = acc;
    let (oper, (expr, mut prod_term)) = tuple;

    match oper {
      Oper::Add => {
        let new_expr = Expr::Add(Box::new(expr_acc), Box::new(expr));
        eval_acc.push(prod_term);
        (new_expr, eval_acc)
      },
      Oper::Sub => {
        let new_expr = Expr::Sub(Box::new(expr_acc), Box::new(expr));
        prod_term.push_exp(EvalExp::new_just_base(
            EvalNode::Num(Number::negative_one())));
        eval_acc.push(prod_term);
        (new_expr, eval_acc)
      },
      _ => panic!("Only Addition and Subtraction operations allowed")
    }
  })
}

fn fold_mult_div_expr(init: (Expr,EvalProdTerm),
    remainder: Vec<(Oper,(Expr,EvalProdTerm))>) -> (Expr, EvalProd) {

  let (init_expr, init_prod_term) = init;
  let mut init_prod = EvalProd::new();
  init_prod.push(init_prod_term);
  let mut init = (init_expr, init_prod);

  remainder.into_iter().fold(init, |acc, tuple| {
    let (expr_acc, mut eval_acc) = acc;
    let (oper, (expr, mut eval)) = tuple;

    match oper {
      Oper::Mul => {
        let new_expr = Expr::Mul(Box::new(expr_acc), Box::new(expr));
        eval_acc.push(eval);
        (new_expr, eval_acc)
      },
      Oper::Div => {
        let new_expr = Expr::Div(Box::new(expr_acc), Box::new(expr));
        eval.reciprocate();
        eval_acc.push(eval);
        (new_expr, eval_acc)
      },
      _ => panic!("Only Addition and Subtraction operations allowed")
    }
  })
}

fn reverse_fold_exponents(initial: (Expr, EvalNode),
    mut remainder: Vec<(Expr,EvalNode)>)
        -> (Expr, EvalProdTerm) {

  remainder.insert(0, initial);
  let (init_exp, init_node) = remainder.pop().unwrap();

  let initial = (init_exp, EvalProdTerm::Exp(EvalExp::new_just_base(init_node)));

  remainder.into_iter().rev().fold(initial, |acc, (expr, node)| {
    let (expr_acc, eval_acc) = acc;
    let new_expr = Exp(Box::new(expr), Box::new(expr_acc));
    let new_eval = EvalProdTerm::Exp(EvalExp::new(node,
        EvalNode::from_statement(Evaluable::new_from_prod_tem(eval_acc))));
    (new_expr, new_eval)
  })
}

fn wrap_radicals_and_factorials(
    radicals: Vec<CompleteStr>,
    factor: (Expr, EvalNode),
    factorials: Vec<CompleteStr>) -> (Expr, EvalNode) {
  let (mut result_expr, mut result_node) = factor;

  for _ in 0..radicals.len() {
    result_expr = Radical(Box::new(result_expr));
    result_node = EvalNode::from_statement(Evaluable::new_from_exp(
        EvalExp::sqrt(result_node)));
  }

  for _ in 0..factorials.len() {
    result_expr = Factorial(Box::new(result_expr));
    result_node = EvalNode::from_statement(Evaluable::new_from_prod_tem(EvalProdTerm::new_factorial(result_node)));
  }

  (result_expr, result_node)
}

named!(exp_term<CompleteStr, (Expr,EvalProdTerm)>, do_parse!(
  initial: possible_factorials_or_radicals >>
  remainder: many0!(
    do_parse!(
      tag!("^")
      >> exp: possible_factorials_or_radicals >> (exp))
  )
  >> (reverse_fold_exponents(initial, remainder))
));

named!(term< CompleteStr, (Expr,EvalProd) >, do_parse!(
    initial: exp_term >>
    remainder: many0!(
           alt!(
             do_parse!(tag!("*") >> mul: exp_term >> (Oper::Mul, mul)) |
             do_parse!(tag!("/") >> div: exp_term >> (Oper::Div, div))
           )
         ) >>
    (fold_mult_div_expr(initial, remainder))
));

named!(expr< CompleteStr, (Expr,Evaluable) >, do_parse!(
    initial: term >>
    remainder: many0!(
           alt!(
             do_parse!(tag!("+") >> add: term >> (Oper::Add, add)) |
             do_parse!(tag!("-") >> sub: term >> (Oper::Sub, sub))
           )
         ) >>
    (fold_plus_minus_expr(initial, remainder))
));

pub fn parse(input: &str) -> Result<(Expr,Number),String> {
  match expr(CompleteStr(input)) {
    Ok((remainder, result)) => {
      if remainder.len() == 0 {
        let (exp, val) = result;
        Ok((exp, val.evaluate()))
      }
      else {
        Err(format!("Found a remainder after parsing: {:?}", remainder))
      }
    },
    Err(err) => {
      error!("Failure in parsing, {:?}", err);
      Err(String::from("Failed to parse input"))
    }
  }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn num(str_value: &'static str) -> Box<Expr> {
      Box::new(Num(str_value.to_string(), None))
    }

    fn num_repeat(prefix: &'static str, repeat: &'static str) -> Box<Expr> {
      Box::new(Num(
          String::from(prefix), Some(String::from(repeat))))
    }

    #[test]
    fn test_just_a_number() {
        let parsed = parse("2");

        match parsed {
          Ok((exp, eval)) => {
            assert_eq!(Box::new(exp), num("2"));
            assert_eq!(eval,
              Number::new_integer(false, BigUint::from(2usize)));
          }
          Err(err) => {
            println!("{:?}", err);
            panic!("Error in parsing");
          }
        }
    }

    #[test]
    fn test_repeats() {

      assert_eq!(
          float(CompleteStr("-123.4(56)")),
          Ok((CompleteStr(""),
              ("-123.4".to_string(), Some("56".to_string())))));

      assert_eq!(
          float(CompleteStr("0.(456)")),
          Ok((CompleteStr(""),
              ("0.".to_string(), Some("456".to_string())))));
    }

    #[test]
    fn float_tests() {
      assert_eq!(
        float(CompleteStr("123.456")),
          Ok((CompleteStr(""),
              ("123.456".to_string(), None)))
      );
      assert_eq!(
        float(CompleteStr("123")),
          Ok((CompleteStr(""),
              ("123".to_string(), None)))
      );
      assert_eq!(
        float(CompleteStr("-.456")),
          Ok((CompleteStr(""),
              ("-0.456".to_string(), None)))
      );
      assert_eq!(
        float(CompleteStr("+123.456")),
          Ok((CompleteStr(""),
              ("123.456".to_string(), None)))
      );
      assert_eq!(
        float(CompleteStr("-123.456")),
          Ok((CompleteStr(""),
              ("-123.456".to_string(), None)))
      );
      assert!(
        float(CompleteStr(".")).is_err()
      );
    }

    #[test]
    fn test_parse_add_statement() {
        let parsed = parse("1 + 2");

        match parsed {
          Ok((exp, eval)) => {
            assert_eq!(exp, Add(num("1"), num("2")));
            assert_eq!(eval,
              Number::new_integer(false, BigUint::from(3usize)));
          }
          Err(err) => {
            println!("{:?}", err);
            panic!("Error in parsing");
          }
        }
    }

    #[test]
    fn test_parse_subtraction_statement() {
        let parsed = parse("1 - 2").unwrap();
        assert_eq!(parsed.0, Sub(num("1"), num("2")));
        assert_eq!(parsed.1, Number::new_integer(true, One::one()));
    }

    #[test]
    fn test_parse_multiplication_statement() {
        let parsed = parse("1 * 2").unwrap();
        assert_eq!(parsed.0, Mul(num("1"), num("2")));
        assert_eq!(parsed.1,
            Number::new_integer(false, BigUint::from(2usize)));
    }

    #[test]
    fn test_power() {
        let parsed = parse("4 ^ 6").unwrap();
        let expected = Exp(num("4"), num("6"));
        assert_eq!(parsed.0, expected);
        assert_eq!(parsed.1, Number::new_integer(
            false,
            BigUint::from(4_096usize)));
    }

    #[test]
    fn test_power_order() {
        let parsed = parse("1024 / 4 ^ 6").unwrap();
        let expected = Div(
            num("1024"),
            Box::new(Exp(num("4"), num("6"))),
        );
        assert_eq!(parsed.0, expected);
        assert_eq!(parsed.1, Number::new_rational(
            false,
            BigUint::from(1usize),
            BigUint::from(4usize)));
    }
    #[test]
    fn test_power_order_2() {
        let parsed = parse("4 ^ 6 / 1024").unwrap();
        let expected = Div(
            Box::new(Exp(num("4"), num("6"))),
            num("1024")
        );
        assert_eq!(parsed.0, expected);
        assert_eq!(parsed.1, Number::new_rational(
            false,
            BigUint::from(4usize),
            BigUint::from(1usize)));
    }

    #[test]
    fn test_parse_multi_level_expression() {
        let parsed = parse("3 * 2 / 8 + 1024 / 4 ^ 6").unwrap();
        let expected = Add(
            Box::new(Div(Box::new(Mul(num("3"), num("2"))), num("8"))),
            Box::new(Div(
                num("1024"),
                Box::new(Exp(num("4"), num("6"))),
            )),
        );
        assert_eq!(parsed.0, expected);
        assert_eq!(parsed.1, Number::one());

    }

    #[test]
    fn test_parse_expression_with_parantheses() {
        let parsed = parse("(1 + 2) * 4.5").unwrap();
        let expected = Mul(
            Box::new(Paren(Box::new(Add(num("1"), num("2"))))),
            num("4.5"),
        );

        assert_eq!(parsed.0, expected);
        assert_eq!(parsed.1, Number::new_rational(
            false,
            BigUint::from(27usize),
            BigUint::from(2usize)))
    }

    #[test]
    fn test_parse_division_statement() {
        let parsed = parse("1 / 2").unwrap();
        assert_eq!(parsed.0, Div(num("1"), num("2")));
        assert_eq!(parsed.1, Number::new_rational(
            false,
            BigUint::from(1usize),
            BigUint::from(2usize)))

    }

    #[test]
    fn test_parse_exp_statement() {
        let parsed = parse("1 ^ 2").unwrap().0;
        assert_eq!(parsed, Exp(num("1"), num("2")));
    }

    #[test]
    fn test_parse_exp_statement_2() {
        let parsed = parse("1 ^ 2 ^ 3").unwrap();
        assert_eq!(parsed.0,
            Exp(num("1"),
                Box::new(
                    Exp(num("2"),
                        num("3")))));
        assert_eq!(parsed.1, Number::one());
    }

    #[test]
    fn test_parse_exp_statement_3() {
        let parsed = parse("1 ^ 2 ^ 3 ^ 4 ^ 5").unwrap();
        assert_eq!(parsed.0,
            Exp(num("1"),
                Box::new(
                    Exp(num("2"),
                        Box::new(
                            Exp(num("3"),
                                Box::new(
                                    Exp(num("4"),
                                        num("5")))))))));
        assert_eq!(parsed.1, Number::one());
    }

    #[test]
    fn test_simple_sqrt() {
        let parsed = parse("√ 2").unwrap();
        assert_eq!(parsed.0, Radical(num("2")));
        assert_eq!(parsed.1, Number::new_rounded(2.0_f64.sqrt()));
    }

    #[test]
    fn test_simple_sqrt_2() {
        let parsed = parse("√ 4").unwrap();
        assert_eq!(parsed.0, Radical(num("4")));
        assert_eq!(parsed.1, Number::new_integer(false, BigUint::from(2usize)));
    }

    #[test]
    fn test_sqrt_1() {
        let parsed = parse("(√ 2 + 3) / 5.6").unwrap();
        assert_eq!(parsed.0,
            Div(Box::new(
                Paren(Box::new(
                    Add(
                        Box::new(Radical(num("2"))),
                        num("3")
                )))),
                num("5.6")));

        let expected = ( 2.0_f64.sqrt() + 3. ) / 5.6;
        let delta = match parsed.1 {
          Number::Rounded(val) => {
            (expected - val).abs()
          }
          _ => panic!("Expected a rounded number and found something else")
        };

        assert!(delta < 0.0000001);
    }

    #[test]
    fn test_sqrt_2() {
        let parsed = parse("√ (2 ^ 3) / 5.6").unwrap();
        assert_eq!(parsed.0,
            Div(Box::new(
                Radical(Box::new(
                    Paren(Box::new(
                        Exp(
                            num("2"),
                            num("3")
                    )))))),
                num("5.6")));

        let expected = 8.0_f64.sqrt() / 5.6;
        let delta = match parsed.1 {
          Number::Rounded(val) => {
            (expected - val).abs()
          }
          _ => panic!("Expected a rounded number and found something else")
        };

        assert!(delta < 0.0000001);
    }

    #[test]
    fn test_simple_sqrt_3() {
        let parsed = parse("√ √ √ 2").unwrap();
        assert_eq!(parsed.0,
          Radical(Box::new(
              Radical(Box::new(
                  Radical(num("2")))))));

        let expected = 2.0_f64.sqrt().sqrt().sqrt();
        let delta = match parsed.1 {
          Number::Rounded(val) => {
            (expected - val).abs()
          }
          _ => panic!("Expected a rounded number and found something else")
        };

        assert!(delta < 0.0000001);
    }

    #[test]
    fn test_simple_factorial() {
        let parsed = parse(" 2. !").unwrap();
        assert_eq!(parsed.0, Factorial(num("2.")));
        assert_eq!(parsed.1, Number::new_integer(false, BigUint::from(2usize)));
    }

    #[test]
    fn test_simple_factorial_2() {
        let parsed = parse(" 4 !").unwrap();
        assert_eq!(parsed.0, Factorial(num("4")));
        assert_eq!(parsed.1, Number::new_integer(false, BigUint::from(24usize)));
    }

    #[test]
    fn test_factorial_1() {
        let parsed = parse("( 2 + 3 ! ) / 5.6").unwrap();
        assert_eq!(parsed.0,
            Div(Box::new(
                Paren(Box::new(
                    Add(
                        num("2"),
                        Box::new(Factorial(num("3")))
                )))),
                num("5.6")));

        assert_eq!(parsed.1, Number::new_rational(false,
            BigUint::from(10usize),
            BigUint::from(7usize)));

    }

    #[test]
    fn test_factorial_2() {
        let parsed = parse("(2 ^ 3) ! / 5.6").unwrap();
        assert_eq!(parsed.0,
            Div(Box::new(
                Factorial(Box::new(
                    Paren(Box::new(
                        Exp(
                            num("2"),
                            num("3")
                    )))))),
                num("5.6")));

        assert_eq!(parsed.1, Number::new_rational(false,
            BigUint::from(50400usize),
            BigUint::from(7usize)));
    }

    #[test]
    fn test_factorial_3() {
        let parsed = parse("2 !! !").unwrap();
        assert_eq!(parsed.0,
            Factorial(Box::new(
                Factorial(Box::new(
                    Factorial(num("2")))))));
        assert_eq!(parsed.1, Number::new_integer(false,
            BigUint::from_str("2").unwrap()));
    }

    #[test]
    fn test_factorial_4() {
        let parsed = parse("4 ! !").unwrap();
        assert_eq!(parsed.0,
            Factorial(Box::new(
                Factorial(num("4")))));
        assert_eq!(parsed.1, Number::new_integer(false,
            BigUint::from_str("620448401733239439360000").unwrap()));
    }

    #[test]
    fn test_square_of_root() {
        let parsed = parse(" ( √ 2 ) ^ 2").unwrap();
        assert_eq!(parsed.0,
            Exp(Box::new(Paren(Box::new(Radical(num("2"))))), num("2")));
        assert_eq!(parsed.1, Number::new_integer(false, BigUint::from(2usize)));
    }
}