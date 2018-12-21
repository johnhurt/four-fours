
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;


use cached::SizedCache;
use statrs::function::gamma::gamma;

use num::{
  Zero,
  One,
  ToPrimitive,
};

use num;

use num::{
  BigUint
};

lazy_static! {
  static ref ONE : Number = Number::Integer(false, One::one());
  static ref NEGATIVE_ONE : Number = Number::Integer(true, One::one());
  static ref ZERO : Number = Number::Integer(false, Zero::zero());
  static ref TWO : Number = Number::Integer(false, BigUint::from(2usize));

  static ref MAX_FACTORIAL : BigUint = BigUint::from(200usize);
}

cached! {
  POWER_OF_TEN: SizedCache<(usize), BigUint> = SizedCache::with_size(1000);
  fn power_of_ten(p: usize) -> BigUint = {
    let result : BigUint = if p == 0 {
      One::one()
    }
    else {
      power_of_ten(p - 1) * 10usize
    };

    result
  }
}

cached_key! {
  FACTORIAL: SizedCache<(BigUint), Option<BigUint>>
      = SizedCache::with_size(MAX_FACTORIAL.to_usize().unwrap() + 1);
  Key = { n.clone() };
  fn factorial(n: &BigUint) -> Option<BigUint> = {
    if n.is_zero() {
      Some(One::one())
    }
    else if n > &*MAX_FACTORIAL {
      None
    }
    else {
      Some(n * factorial(&(n - 1usize)).unwrap())
    }
  }
}


#[derive(PartialEq,Clone)]
pub enum Number {
  Integer(bool,BigUint),
  Rational(bool,BigUint,BigUint),
  Rounded(f64),
  ReallyBig(bool),
  ReallySmall(bool),
  Infinity(bool),
  Unknown,
  NaN
}

impl Display for Number {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    use self::Number::*;
    match *self {
      Integer(negative, ref value) => {
        write!(format, "{}{}", if negative { "-" } else { "" }, value)
      },
      Rational(negative, ref num, ref denom) => {
        write!(format, "{}{}/{}", if negative { "-" } else { "" }, num, denom)
      },
      Rounded(value) => write!(format, "{}", value),
      ReallyBig(negative) => {
        write!(format, "{} Really Big", if negative { "Negative " } else { "" })
      },
      ReallySmall(negative) => {
        write!(format, "{} Really Small",
            if negative { "Negative " } else { "" })
      },
      Infinity(negative) => {
        write!(format, "{} Infinity", if negative { "Negative " } else { "" })
      },
      Unknown => write!(format, "Unknown"),
      NaN => write!(format, "NaN")
    }
  }
}

impl Debug for Number {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    use self::Number::*;
    match *self {
      Integer(negative, ref value) => {
        write!(format, "{}{}",
            if negative { "-" } else { "" },
            value.to_str_radix(10))
      },
      Rational(negative, ref num, ref denom) => {
        write!(format, "{}{}/{}",
            if negative { "-" } else { "" },
            num.to_str_radix(10),
            denom.to_str_radix(10))
      },
      Rounded(value) => write!(format, "{:?}", value),
      ReallyBig(negative) => {
        write!(format, "{} Really Big", if negative { "Negative " } else { "" })
      },
      ReallySmall(negative) => {
        write!(format, "{} Really Small",
            if negative { "Negative " } else { "" })
      },
      Infinity(negative) => {
        write!(format, "{} Infinity", if negative { "Negative " } else { "" })
      },
      Unknown => write!(format, "Unknown"),
      NaN => write!(format, "NaN")
    }
  }
}

impl Number {

  pub fn zero() -> Number {
    ZERO.clone()
  }

  pub fn one() -> Number {
    ONE.clone()
  }

  pub fn two() -> Number {
    TWO.clone()
  }

  pub fn negative_one() -> Number {
    NEGATIVE_ONE.clone()
  }

  pub fn is_zero(&self) -> bool {
    match self {
      Number::Integer(_, val) => val.is_zero(),
      _ => false
    }
  }

  pub fn is_one(&self) -> bool {
    match self {
      Number::Integer(neg, val) => val.is_one() && !neg,
      _ => false
    }
  }

  pub fn is_negative(&self) -> Option<bool> {
    match self {
      Number::Integer(neg, val) => Some(*neg && !val.is_zero()),
      Number::Rational(neg, _, _) => Some(*neg),
      Number::Rounded(val) => Some(*val < 0.),
      Number::ReallyBig(neg) => Some(*neg),
      Number::ReallySmall(neg) => Some(*neg),
      Number::Infinity(neg) => Some(*neg),
      _ => None
    }
  }

  /// Get the reciprocal of this number
  pub fn recip(&self) -> Number {
    if self.is_one() {
      return Number::one();
    }

    if self.is_zero() {
      return Number::Infinity(false);
    }

    match self {
      Number::Integer(neg, val) => {
        Number::new_rational(*neg, One::one(), val.clone())
      },
      Number::Rational(neg, num, denom) => {
        Number::new_rational(*neg, denom.clone(), num.clone())
      },
      Number::Rounded(val) => {
        Number::new_rounded(1.0 / *val)
      },
      Number::ReallyBig(neg) => {
        Number::ReallySmall(*neg)
      },
      Number::ReallySmall(neg) => {
        Number::ReallyBig(*neg)
      },
      Number::Infinity(_) => {
        Number::zero()
      },
      _ => self.clone()
    }
  }

  fn repeated_nines(l: usize) -> BigUint {
    power_of_ten(l) - 1usize
  }

  /// Construct a rational number from a repeating decimal
  fn from_repeating_decimal(
      negative: bool,
      whole: &str,
      decimal: &str,
      non_repeating_places: usize,
      repeat: &str)
          -> Number {

    let fixed_denom = power_of_ten(non_repeating_places);
    let fixed_decimal_value = if non_repeating_places == 0 {
        Zero::zero()
      }
      else {
        Number::expect_big_uint(decimal)
      };

    let fixed_num = Number::expect_big_uint(whole) * &fixed_denom
        + &fixed_decimal_value;

    let repeat_denom = Number::repeated_nines(repeat.len());
    let repeat_num = Number::expect_big_uint(repeat);

    Number::new_rational(
        negative,
        &repeat_denom * fixed_num + repeat_num,
        repeat_denom * fixed_denom)
  }

  fn expect_big_uint(to_parse: &str) -> BigUint {
    BigUint::from_str(to_parse)
        .expect(&format!("Failed to parse {} as uint", to_parse))
  }

  pub fn from_str(mut prefix: &str, repeat_opt: &Option<String>) -> Number {
    if prefix.is_empty() {
      return Number::NaN;
    }

    let negative = prefix.starts_with('-');

    if negative {
      prefix = prefix.get(1..).unwrap();
    }

    match prefix.find('.') {
      Some(decimal_pos) => {

        let whole = prefix.get(0..decimal_pos).unwrap();
        let decimal = prefix.get((decimal_pos + 1)..).unwrap();
        let decimal_digits = decimal.len();

        match repeat_opt {
          Some(repeat) => {
            Number::from_repeating_decimal(
                negative,
                whole,
                decimal,
                decimal_digits,
                repeat)
          },
          _ => {
            if decimal_digits == 0 {
              Number::new_integer(negative, Number::expect_big_uint(whole))
            }
            else {
              if decimal.trim_matches('0').is_empty() {
                Number::new_integer(negative, Number::expect_big_uint(whole))
              }
              else {
                let denom = power_of_ten(decimal_digits);
                let num = Number::expect_big_uint(whole) * &denom
                    + Number::expect_big_uint(decimal);
                Number::new_rational(negative, num, denom)
              }
            }
          }
        }
      },
      _ => {
        Number::new_integer(negative, Number::expect_big_uint(prefix))
      }
    }
  }

  /// Create a new rounded number based on the given value, this will
  /// consider all infinite values as unknown because they can either be
  /// really big or actually infinite ... which is literally unknown
  pub fn new_rounded(val: f64) -> Number {
    if val.is_normal() {
      Number::Rounded(val)
    }
    else if val.is_nan() {
      Number::NaN
    }
    else {
      Number::Unknown
    }
  }

  pub fn new_integer(negative: bool, val: BigUint) -> Number {
     if val.is_zero() {
      Number::Integer(false, Zero::zero())
    }
    else {
      Number::Integer(negative, val)
    }
  }

  pub fn new_integer_tuple((neg, val): (bool, BigUint)) -> Number {
    Number::new_integer(neg, val)
  }

  pub fn new_rational(negative: bool, mut num: BigUint, mut denom: BigUint)
      -> Number {

    if num.is_zero() {
      if denom.is_zero() {
        return Number::NaN;
      }
      else {
        return Number::new_integer(negative, Zero::zero());
      }
    }

    if (denom.is_zero()) {
      return Number::Infinity(negative);
    }

    if num == denom {
      return Number::new_integer(negative, One::one());
    }

    let mut min = num.clone();
    let mut max = denom.clone();

    if min > max {
      let temp = min.clone();
      min = max;
      max = temp;
    }


    while !min.is_zero() {
        let temp = min.clone();
        min = max % &min;
        max = temp;
    }

    num /= &max;
    denom /= &max;

    if denom.is_one() {
      Number::new_integer(negative, num)
    }
    else {
      Number::Rational(negative, num, denom)
    }
  }

  pub fn variant_ordinal(&self) -> usize {
    use self::Number::*;
    match self {
      Integer(_,_) => 0,
      Rational(_,_,_) => 1,
      Rounded(_) => 2,
      ReallyBig(_) => 3,
      ReallySmall(_) => 4,
      Infinity(_) => 5,
      Unknown => 6,
      NaN => 7
    }
  }

  /// Compare the variants of the given numbers to determine if their order
  /// should be reversed (when being used with a commutable operator)
  /// This is determined by creating a cross matrix of the variants in their
  /// with lhs going down and rhs going right.  True will be returned from the
  /// matrix cells that fall in the bottom left (IE: Integer + Rational will
  /// not be reversed, but Ration + Integer will be)
  fn should_swap_order(lhs: &Number, rhs: &Number) -> bool {
    lhs.variant_ordinal() > rhs.variant_ordinal()
  }

  /// get the result of adding this number with another number
  pub fn add(&self, rhs: &Number) -> Number {

    if Number::should_swap_order(self, rhs) {
      return rhs.add(self);
    }

    match self {

      // Handle the cases where lhs is an integer
      Number::Integer(l_neg, l_val) => {

        if (l_val.is_zero()) {
          return rhs.clone();
        }

        match rhs {

          // Handle Integer + Integer
          Number::Integer(r_neg, r_val) => {
            Number::new_integer_tuple(add(*l_neg, l_val, *r_neg, &r_val))
          },

          // Handle Integer + Rational
          Number::Rational(r_neg, r_num, r_denom) => {
            let l_num = l_val * r_denom;
            let (sign, val) = add(*l_neg, &l_num, *r_neg, r_num);
            Number::new_rational(sign, val, r_denom.clone())
          },

          // Handle Integer + Rounded
          Number::Rounded(r_val) => {
            match l_val.to_f64() {
              Some(l_val_f64) => {
                Number::new_rounded(r_val
                  + (if *l_neg { -1. } else { 1. }) * l_val_f64)
              },
              None => {
                // This happens when the left val is too big to fit in a f64
                // Let's classify this as really big
                Number::add(rhs, &Number::ReallyBig(*l_neg))
              }
            }
          },

          // Handle Integer + Really Big
          Number::ReallyBig(neg) => {
            if l_neg == neg {
              Number::ReallyBig(neg.clone())
            }
            else {
              Number::Unknown
            }
          },

          // Handle Integer + Really Small
          Number::ReallySmall(_) => {
            match l_val.to_f64() {
              Some(l_val_f64) => {
                Number::new_rounded(
                  (if *l_neg { -1. } else { 1. }) * l_val_f64)
              },
              None => {
                // This happens when the left val is too big to fit in a f64
                // Let's classify this as really big
                Number::ReallyBig(l_neg.clone())
              }
            }
          },

          // Unkonwn, Nan, and Inifity are unaffected by addition
          _ => rhs.clone()
        }
      },

      // Handle All cases where lhs is Rational
      Number::Rational(l_neg, l_num, l_denom) => {

        match rhs {

          // Handle Rational + Rational
          Number::Rational(r_neg, r_num, r_denom) => {
            let denom = l_denom * r_denom;
            let r_num_final = r_num * l_denom;
            let l_num_final = l_num * r_denom;
            let (neg, num) = add(*l_neg, &l_num_final, *r_neg, &r_num_final);
            Number::new_rational(neg, num, denom)
          },

          // Handle Rational + Rounded
          Number::Rounded(r_val) => {
            match l_num.to_f64() {
              Some(l_num_f64) => {
                match l_denom.to_f64() {
                  Some(l_denom_f64) => {
                    Number::new_rounded(l_num_f64 / l_denom_f64 + r_val)
                  },
                  None => Number::Unknown
                }
              },
              None => Number::Unknown
            }
          },

          // Handle Rational + Really Big
          Number::ReallyBig(r_neg) => {
            if l_neg == r_neg {
              Number::ReallyBig(r_neg.clone())
            }
            else {
              Number::Unknown
            }
          },

          // Handle Rational + Really Small
          Number::ReallySmall(_) => Number::Unknown,

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is Rounded
      Number::Rounded(l_val) => {

        match rhs {

          // Handle Rounded + Rounded
          Number::Rounded(r_val) => {
            Number::new_rounded(l_val + r_val)
          },

          // Handle Rounded + Really Big
          Number::ReallyBig(r_neg) => {
            if (*l_val >= 0.) && *r_neg {
              Number::ReallyBig(r_neg.clone())
            }
            else {
              Number::Unknown
            }
          },

          // Handle Rounded + Really Small
          Number::ReallySmall(_) => Number::Unknown,

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is Really Big
      Number::ReallyBig(l_neg) => {
        match rhs {

          // Handle Really Big + Really Big
          Number::ReallyBig(r_neg) => {
            if r_neg == l_neg {
              Number::ReallyBig(l_neg.clone())
            }
            else {
              Number::Unknown
            }
          },

          // Handle Really Big + Really Small
          Number::ReallySmall(_) => self.clone(),

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is really small
      Number::ReallySmall(l_neg) => {
        match rhs {

          // Handle Really Small + Really Small
          Number::ReallySmall(r_neg) => {
            if l_neg == r_neg {
              self.clone()
            }
            else {
              Number::Unknown
            }
          },

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      }

      // Handle all cases where lhs is infinite
      Number::Infinity(l_neg) => {
        match rhs {
          Number::Infinity(r_neg) => {
            if l_neg == r_neg {
              self.clone()
            }
            else {
              Number::Unknown
            }
          },

          // Handle Unknown and NaN which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is Unknown
      Number::Unknown => {
        match rhs {
          Number::Unknown => Number::Unknown,
          _ => Number::NaN
        }
      },

      // NaN + anything = NaN
      Number::NaN => Number::NaN
    }
  }

  pub fn factorial(&self) -> Number {
    match self {
      Number::Integer(neg, val) => {
        if *neg {
          Number::NaN
        }
        else {
          factorial(val)
              .map(|v| Number::new_integer(false, v))
              .unwrap_or(Number::ReallyBig(false))
        }
      },
      _ => Number::NaN
    }
  }

  /// get the result of multiplying this number with another number
  pub fn multiply(&self, rhs: &Number) -> Number {

    if Number::should_swap_order(self, rhs) {
      return rhs.add(self);
    }

    match self {

      // Handle the cases where lhs is an integer
      Number::Integer(l_neg, l_val) => {

        if (l_val.is_zero()) {
          return ZERO.clone();
        }

        if (l_val.is_one()) {
          return rhs.clone();
        }

        match rhs {

          // Handle Integer * Integer
          Number::Integer(r_neg, r_val) => {
            Number::new_integer_tuple(multiply(*l_neg, l_val, *r_neg, &r_val))
          },

          // Handle Integer * Rational
          Number::Rational(r_neg, r_num, r_denom) => {
            let (neg, num) = multiply(*l_neg, l_val, *r_neg, r_num);
            Number::new_rational(neg, num, r_denom.clone())
          },

          // Handle Integer * Rounded
          Number::Rounded(r_val) => {
            match l_val.to_f64() {
              Some(l_val_f64) => {
                Number::new_rounded(r_val
                  * (if *l_neg { -1. } else { 1. }) * l_val_f64)
              },
              None => {
                // This happens when the left val is too big to fit in a f64
                // Let's classify this as really big
                Number::add(rhs, &Number::ReallyBig(*l_neg))
              }
            }
          },

          // Handle Integer * Really Big
          Number::ReallyBig(neg) => {
            Number::ReallyBig(l_neg == neg)
          },

          // Handle Integer * Really Small
          Number::ReallySmall(_) => {
            Number::Unknown
          },

          // Unkonwn, Nan, and Inifity are unaffected by addition
          _ => rhs.clone()
        }
      },

      // Handle All cases where lhs is Rational
      Number::Rational(l_neg, l_num, l_denom) => {

        match rhs {

          // Handle Rational * Rational
          Number::Rational(r_neg, r_num, r_denom) => {
            let denom = l_denom * r_denom;
            let (neg, num) = multiply(*l_neg, l_num, *r_neg, r_num);
            Number::new_rational(neg, num, denom)
          },

          // Handle Rational * Rounded
          Number::Rounded(r_val) => {
            match l_num.to_f64() {
              Some(l_num_f64) => {
                match l_denom.to_f64() {
                  Some(l_denom_f64) => {
                    Number::new_rounded(l_num_f64 / l_denom_f64 * r_val)
                  },
                  None => Number::Unknown
                }
              },
              None => Number::Unknown
            }
          },

          // Handle Rational * Really Big
          Number::ReallyBig(_) => Number::Unknown,

          // Handle Rational * Really Small
          Number::ReallySmall(_) => Number::Unknown,

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is Rounded
      Number::Rounded(l_val) => {

        match rhs {

          // Handle Rounded * Rounded
          Number::Rounded(r_val) => {
            Number::new_rounded(l_val * r_val)
          },

          // Handle Rounded * Really Big
          Number::ReallyBig(r_neg) => Number::Unknown,

          // Handle Rounded * Really Small
          Number::ReallySmall(_) => Number::Unknown,

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is Really Big
      Number::ReallyBig(l_neg) => {
        match rhs {

          // Handle Really Big * Really Big
          Number::ReallyBig(r_neg) => {
            Number::ReallyBig(r_neg != l_neg)
          },

          // Handle Really Big * Really Small
          Number::ReallySmall(_) => Number::Unknown,

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is really small
      Number::ReallySmall(l_neg) => {
        match rhs {

          // Handle Really Small * Really Small
          Number::ReallySmall(r_neg) => {
            Number::ReallySmall(l_neg != r_neg)
          },

          // Handle Unknown, NaN, and Infinities which are unchanged
          _ => rhs.clone()
        }
      }

      // Handle all cases where lhs is infinite
      Number::Infinity(l_neg) => {
        match rhs {
          Number::Infinity(r_neg) => Number::Infinity(l_neg != r_neg),

          // Handle Unknown and NaN which are unchanged
          _ => rhs.clone()
        }
      },

      // Handle all cases where lhs is Unknown
      Number::Unknown => {
        match rhs {
          Number::Unknown => Number::Unknown,
          _ => Number::NaN
        }
      },

      // NaN + anything = NaN
      Number::NaN => Number::NaN
    }
  }

}


/// Handle the addition between two big ints that have their sign split out
fn add(l_neg: bool, l_val: &BigUint, r_neg: bool, r_val: &BigUint)
    -> (bool, BigUint) {
  if l_neg == r_neg {
    (l_neg.clone(), r_val + l_val)
  }
  else if l_val >= r_val {
    (l_neg.clone(), l_val - r_val)
  }
  else {
    (r_neg.clone(), r_val - l_val)
  }
}

/// Handle the multiplication between two big ints that have their sign
/// split out
fn multiply(l_neg: bool, l_val: &BigUint, r_neg: bool, r_val: &BigUint)
    -> (bool, BigUint) {
  (l_neg != r_neg, r_val * l_val)
}


#[test]
fn test_number_from_str() {
  assert_eq!(
      Number::Integer(false, BigUint::from(4usize)),
      Number::from_str("4", &None));

  assert_eq!(
      Number::Integer(true, BigUint::from(44usize)),
      Number::from_str("-44.000000", &None));

  assert_eq!(
      Number::Integer(true, BigUint::from(1usize)),
      Number::from_str("-0.", &Some("9".to_string())));

  assert_eq!(
      Number::Rational(false, BigUint::from(13usize), BigUint::from(8usize)),
      Number::from_str("1.625", &None));

  assert_eq!(
      Number::Rational(false, BigUint::from(47usize), BigUint::from(45usize)),
      Number::from_str("1.0", &Some("44".to_string())));
}

#[test]
fn test_new_rational_number() {
  assert_eq!(Number::Rational(
          false,
          BigUint::from(3usize),
          BigUint::from(5usize)),
      Number::new_rational(
          false,
          BigUint::from(6usize),
          BigUint::from(10usize)));

  assert_eq!(Number::Integer(
          true,
          BigUint::from(2usize)),
      Number::new_rational(
          true,
          BigUint::from(12usize),
          BigUint::from(6usize)));

  assert_eq!(Number::Rational(
          false,
          BigUint::from(7usize),
          BigUint::from(5usize)),
      Number::new_rational(
          false,
          BigUint::from(7usize),
          BigUint::from(5usize)));

  assert_eq!(Number::Infinity(true),
      Number::new_rational(
          true,
          BigUint::from(7usize),
          BigUint::from(0usize)));

  assert_eq!(Number::NaN,
      Number::new_rational(
          true,
          BigUint::from(0usize),
          BigUint::from(0usize)));

  assert_eq!(Number::Integer(
          false,
          BigUint::from(0usize)),
      Number::new_rational(
          true,
          BigUint::from(0usize),
          BigUint::from(6usize)));
}