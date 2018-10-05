use nom;
use nom::{digit, multispace};
use nom::types::CompleteStr;

use std::str;
use std::str::FromStr;

use std::fmt;
use std::fmt::{Debug, Display, Formatter};

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

named!(parens< CompleteStr, Expr >, delimited!(
    delimited!(opt!(multispace), tag!("("), opt!(multispace)),
    map!(map!(expr, Box::new), Expr::Paren),
    delimited!(opt!(multispace), tag!(")"), opt!(multispace))
  )
);

named!(factor< CompleteStr, Expr >, alt_complete!(
  map!(
      map!(
        delimited!(ws!(tag!("√")), factor, opt!(multispace)),
        Box::new),
      Expr::Radical
    )
  | map!(
    delimited!(opt!(multispace), float, opt!(multispace)),
    to_num
  )
  | parens
  )
);

named!(possible_factorial< CompleteStr, Expr >,
  do_parse!(
    fac: factor >>
    excl: many0!(ws!(tag!("!"))) >>
    (wrap_factorials(fac, excl))
  )
);

fn to_num(tup: (String, Option<String>)) -> Expr {
  let (prefix, repeat) = tup;
  Num(prefix, repeat)
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

fn fold_exprs(initial: Expr, remainder: Vec<(Oper, Expr)>) -> Expr {
  remainder.into_iter().fold(initial, |acc, pair| {
    let (oper, expr) = pair;
    match oper {
      Oper::Add => Expr::Add(Box::new(acc), Box::new(expr)),
      Oper::Sub => Expr::Sub(Box::new(acc), Box::new(expr)),
      Oper::Mul => Expr::Mul(Box::new(acc), Box::new(expr)),
      Oper::Div => Expr::Div(Box::new(acc), Box::new(expr))
    }
  })
}

fn reverse_fold_exponents(initial: Expr, mut remainder: Vec<Expr>) -> Expr {

  remainder.insert(0, initial);
  let mut initial = remainder.pop().unwrap();

  remainder.into_iter().rev().fold(initial, |acc, expr| {
    Exp(Box::new(expr), Box::new(acc))
  })
}

fn wrap_factorials(initial: Expr, factorials: Vec<CompleteStr>) -> Expr {
  let mut result = initial;

  for _ in 0..factorials.len() {
    result = Factorial(Box::new(result));
  }

  result
}

named!(exp_term<CompleteStr, Expr>, do_parse!(
  initial: possible_factorial >>
  remainder: many0!(
    do_parse!(
      tag!("^")
      >> exp: possible_factorial >> (exp))
  )
  >> (reverse_fold_exponents(initial, remainder))
));

named!(term< CompleteStr, Expr >, do_parse!(
    initial: exp_term >>
    remainder: many0!(
           alt!(
             do_parse!(tag!("*") >> mul: exp_term >> (Oper::Mul, mul)) |
             do_parse!(tag!("/") >> div: exp_term >> (Oper::Div, div))
           )
         ) >>
    (fold_exprs(initial, remainder))
));

named!(expr< CompleteStr, Expr >, do_parse!(
    initial: term >>
    remainder: many0!(
           alt!(
             do_parse!(tag!("+") >> add: term >> (Oper::Add, add)) |
             do_parse!(tag!("-") >> sub: term >> (Oper::Sub, sub))
           )
         ) >>
    (fold_exprs(initial, remainder))
));

pub fn parse(input: &str) -> Result<Expr,String> {
  match expr(CompleteStr(input)) {
    Ok((remainder, exp)) => {
      if remainder.len() == 0 {
        Ok(exp)
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
    use super::*;

    fn num(str_value: &'static str) -> Box<Expr> {
      Box::new(Num(str_value.to_string(), None))
    }

    fn num_repeat(prefix: &'static str, repeat: &'static str) -> Box<Expr> {
      Box::new(Num(String::from(prefix), Some(String::from(repeat))))
    }

    #[test]
    fn test_just_a_number() {
        let parsed = parse("2");

        match parsed {
          Ok(exp) => {
            assert_eq!(Box::new(exp), num("2"));
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
          Ok(exp) => {
            assert_eq!(exp, Add(num("1"), num("2")));
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
        assert_eq!(parsed, Sub(num("1"), num("2")));
    }

    #[test]
    fn test_parse_multiplication_statement() {
        let parsed = parse("1 * 2").unwrap();
        assert_eq!(parsed, Mul(num("1"), num("2")));
    }

    #[test]
    fn test_parse_multi_level_expression() {
        let parsed = parse("1 * 2 + 3 / 4 ^ 6").unwrap();
        let expected = Add(
            Box::new(Mul(num("1"), num("2"))),
            Box::new(Div(
                num("3"),
                Box::new(Exp(num("4"), num("6"))),
            )),
        );
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_parse_expression_with_parantheses() {
        let parsed = parse("(1 + 2) * 4.5").unwrap();
        let expected = Mul(
            Box::new(Paren(Box::new(Add(num("1"), num("2"))))),
            num("4.5"),
        );
        println!("parsed: {}", parsed);

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_parse_division_statement() {
        let parsed = parse("1 / 2").unwrap();
        assert_eq!(parsed, Div(num("1"), num("2")));
    }

    #[test]
    fn test_parse_exp_statement() {
        let parsed = parse("1 ^ 2").unwrap();
        assert_eq!(parsed, Exp(num("1"), num("2")));
    }

    #[test]
    fn test_parse_exp_statement_2() {
        let parsed = parse("1 ^ 2 ^ 3").unwrap();
        assert_eq!(parsed,
            Exp(num("1"),
                Box::new(
                    Exp(num("2"),
                        num("3")))));
    }

    #[test]
    fn test_parse_exp_statement_3() {
        let parsed = parse("1 ^ 2 ^ 3 ^ 4 ^ 5").unwrap();
        assert_eq!(parsed,
            Exp(num("1"),
                Box::new(
                    Exp(num("2"),
                        Box::new(
                            Exp(num("3"),
                                Box::new(
                                    Exp(num("4"),
                                        num("5")))))))));
    }

    #[test]
    fn test_simple_sqrt() {
        let parsed = parse("√ 2").unwrap();
        assert_eq!(parsed, Radical(num("2")));
    }

    #[test]
    fn test_sqrt_1() {
        let parsed = parse("(√ 2 + 3) / 5.6").unwrap();
        assert_eq!(parsed,
            Div(Box::new(
                Paren(Box::new(
                    Add(
                        Box::new(Radical(num("2"))),
                        num("3")
                )))),
                num("5.6")));
    }

    #[test]
    fn test_sqrt_2() {
        let parsed = parse("√ (2 ^ 3) / 5.6").unwrap();
        assert_eq!(parsed,
            Div(Box::new(
                Radical(Box::new(
                    Paren(Box::new(
                        Exp(
                            num("2"),
                            num("3")
                    )))))),
                num("5.6")));
    }

    #[test]
    fn test_simple_sqrt_3() {
        let parsed = parse("√ √ √ 2").unwrap();
        assert_eq!(parsed,
          Radical(Box::new(
              Radical(Box::new(
                  Radical(num("2")))))));
    }

    #[test]
    fn test_simple_factorial() {
        let parsed = parse(" 2. !").unwrap();
        assert_eq!(parsed, Factorial(num("2.")));
    }

    #[test]
    fn test_factorial_1() {
        let parsed = parse("( 2 + 3 ! ) / 5.6").unwrap();
        assert_eq!(parsed,
            Div(Box::new(
                Paren(Box::new(
                    Add(
                        num("2"),
                        Box::new(Factorial(num("3")))
                )))),
                num("5.6")));
    }

    #[test]
    fn test_factorial_2() {
        let parsed = parse("(2 ^ 3) ! / 5.6").unwrap();
        assert_eq!(parsed,
            Div(Box::new(
                Factorial(Box::new(
                    Paren(Box::new(
                        Exp(
                            num("2"),
                            num("3")
                    )))))),
                num("5.6")));
    }

    #[test]
    fn test_factorial_3() {
        let parsed = parse("2 !! !").unwrap();
        assert_eq!(parsed,
            Factorial(Box::new(
                Factorial(Box::new(
                    Factorial(num("2")))))));
    }
}