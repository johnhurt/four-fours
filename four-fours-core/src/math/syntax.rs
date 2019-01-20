use nom;
use nom::{digit, multispace};
use nom::types::CompleteStr;

use num::{
  BigUint,
  One,
  ToPrimitive,
  Integer
};

use std::str;

use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use math::Number;


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

#[derive(Clone)]
pub enum EvalNode {
  Num(Number),
  Statement(Evaluable),
  Factorial(Box<EvalNode>)
}

#[derive(Clone)]
pub struct EvalExp {
  base: EvalNode,
  power: EvalNode
}

#[derive(Clone)]
pub struct EvalProd {
  terms: Vec<EvalExp>,
  number_collector: Number
}

#[derive(Clone)]
pub struct Evaluable {
  terms: Vec<EvalProd>,
  number_collector: Number
}

impl Debug for EvalNode {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      EvalNode::Num(ref val) => Debug::fmt(val, format),
      EvalNode::Statement(ref stmt) => Debug::fmt(stmt, format),
      EvalNode::Factorial(ref node) => write!(format, "({:?})!", *node)
    }
  }
}

impl Debug for EvalExp {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {

    let show_pow = if let EvalNode::Num(pow_val) = &self.power {
      !pow_val.is_one()
    }
    else {
      true
    };

    if show_pow {
      write!(format, "{:?}^{:?}", self.base, self.power)
    }
    else {
      write!(format, "{:?}", self.base)
    }
  }
}

impl Debug for EvalProd {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

    if !self.number_collector.is_one() {
      Debug::fmt(&self.number_collector, format)?;
      count += 1;
    }

    for term in &self.terms {
      if count > 0 {
        write!(format, "*")?;
      }

      Debug::fmt(term, format)?;

      count += 1;
    }

    Ok(())
  }
}

impl Debug for Evaluable {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

    if !self.number_collector.is_one() {
      Debug::fmt(&self.number_collector, format)?;
      count += 1;
    }

    for term in &self.terms {
      if count > 0 {
        write!(format, "+")?;
      }

      Debug::fmt(term, format)?;

      count += 1;
    }

    Ok(())
  }
}

impl Display for EvalNode {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      EvalNode::Num(ref val) => Debug::fmt(val, format),
      EvalNode::Statement(ref stmt) => Display::fmt(stmt, format),
      EvalNode::Factorial(ref node) => write!(format, "({:?})!", *node)
    }
  }
}

impl Display for EvalExp {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {

    let show_pow = if let EvalNode::Num(pow_val) = &self.power {
      !pow_val.is_one()
    }
    else {
      true
    };

    if show_pow {
      write!(format, "{:?}^{:?}", self.base, self.power)
    }
    else {
      write!(format, "{:?}", self.base)
    }
  }
}

impl Display for EvalProd {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

    if !self.number_collector.is_one() {
      Debug::fmt(&self.number_collector, format)?;
      count += 1;
    }

    for term in &self.terms {
      if count > 0 {
        write!(format, "*")?;
      }

      Debug::fmt(term, format);

      count += 1;
    }

    Ok(())
  }
}

impl Display for Evaluable {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

    if !self.number_collector.is_one() {
      Debug::fmt(&self.number_collector, format)?;
      count += 1;
    }

    for term in &self.terms {
      if count > 0 {
        write!(format, "+")?;
      }

      Debug::fmt(term, format)?;

      count += 1;
    }

    Ok(())
  }
}

impl EvalNode {

  pub fn new_factorial(node: EvalNode) -> EvalNode {
    match node {
      EvalNode::Num(val) => {
        EvalNode::Num(val.factorial())
      },
      _ => {
        EvalNode::Factorial(Box::new(node))
      }
    }
  }

  /// Create a new node from the the given evaluable statement
  pub fn from_statement(stmt: Evaluable) -> EvalNode {
    match stmt.as_prod() {
      Ok(prod) => {
        match prod.as_exp() {
          Ok(exp) => {
            match exp.as_number() {
              Ok(num) => {
                EvalNode::Num(num)
              },
              Err(exp) => {
                EvalNode::Statement(Evaluable::new_from_exp(exp))
              }
            }
          },
          Err(prod) => {
            EvalNode::Statement(Evaluable::new_from_prod(prod))
          }
        }
      },
      Err(stmt) => {
        EvalNode::Statement(stmt)
      }
    }
  }

  pub fn negate(&mut self) {
    let replacement = match self {
      EvalNode::Num(ref mut val) => {
        *val = val.multiply(&Number::negative_one());
        None
      },
      EvalNode::Statement(ref mut stmt) => {
        stmt.negate();
        None
      },
      EvalNode::Factorial(node) => {
        let mut stmt = Evaluable::new_from_exp(
            EvalExp::new((**node).clone(), None));
        stmt.negate();
        Some(EvalNode::Statement(stmt))
      }
    };

    if let Some(new_self) = replacement {
      *self = new_self
    }
  }

}

impl Evaluable {

  pub fn new() -> Evaluable {
    Evaluable {
      terms: Vec::new(),
      number_collector: Number::zero()
    }
  }

  pub fn new_from_prod(prod: EvalProd) -> Evaluable {
    Evaluable {
      terms: vec![prod],
      number_collector: Number::zero()
    }
  }

  pub fn new_from_exp(exp: EvalExp) -> Evaluable {
    let mut result = Evaluable::new();
    let mut prod = EvalProd::new();

    prod.push(exp);
    result.push(prod);

    result
  }

  pub fn push(&mut self, term: EvalProd) {
    match term.as_exp() {
      Ok(exp) => {
        self.push_exp(exp);
      },
      Err(term) => self.terms.push(term)
    }
  }

  pub fn push_exp(&mut self, exp: EvalExp) {
    match exp.as_number() {
      Ok(val) => {
        self.number_collector = self.number_collector.add(&val);
      },
      Err(exp) => {
        self.terms.push(EvalProd::new_from_exp(exp));
      }
    }
  }

  pub fn len(&self) -> usize {
    self.terms.len()
        + if self.number_collector.is_zero() { 0usize } else { 1usize }
  }

  pub fn negate(&mut self) {
    self.number_collector
        = self.number_collector.multiply(&Number::negative_one());

    self.terms.iter_mut().for_each(|term| term.negate())
  }

  /// Get this evaluable as a single product if possible otherwise
  /// return self as the error of the result
  pub fn as_prod(mut self) -> Result<EvalProd,Evaluable> {
    if self.terms.is_empty() {
      Ok(EvalProd::new_from_exp(
          EvalExp::from_number(self.number_collector)))
    }
    else if self.number_collector.is_zero() && self.terms.len() == 1 {
      Ok(self.terms.pop().unwrap())
    }
    else {
      Err(self)
    }
  }

  pub fn evaluate(self) -> Number {

    if self.terms.is_empty() {
      self.number_collector
    }
    else {
      println!("{}", self);
      Number::Unknown
    }
  }
}

impl EvalProd {

  pub fn new() -> EvalProd {
    EvalProd {
      terms: Vec::new(),
      number_collector: Number::one()
    }
  }

  /// Create a new product with the give exp
  pub fn new_from_exp(exp: EvalExp) -> EvalProd {
    let mut result = EvalProd::new();
    result.push(exp);
    result
  }

  pub fn push(&mut self, term: EvalExp) {
    if self.number_collector.is_zero() {
      return;
    }

    match term.as_number() {
      Ok(val) => {
        if val.is_zero() {
          self.number_collector = Number::zero();
          self.terms.clear();
        }
        else {
          self.number_collector
              = self.number_collector.multiply(&val);
        }
      },
      Err(term) => {
        self.terms.push(term)
      }
    }
  }

  pub fn len(&self) -> usize {
    self.terms.len()
        + if self.number_collector.is_zero() { 0usize } else { 1usize }
  }

  /// Get the this product as a single exp term or an err with this
  /// in it
  pub fn as_exp(mut self) -> Result<EvalExp,EvalProd> {
    if self.number_collector.is_zero() || self.terms.is_empty() {
      Ok(EvalExp::new(EvalNode::Num(self.number_collector), None))
    }
    else if self.number_collector.is_one() && self.terms.len() == 1 {
      Ok(self.terms.pop().unwrap())
    }
    else {
      Err(self)
    }
  }

  pub fn negate(&mut self) {
    self.number_collector
        = self.number_collector.multiply(&Number::negative_one());
  }
}

impl EvalExp {

  fn from_number(num: Number) -> EvalExp {
    EvalExp::new(EvalNode::Num(num), None)
  }

  fn raw(base: EvalNode, power: EvalNode) -> EvalExp {
    EvalExp {
      base: base,
      power: power
    }
  }

  fn new(base_node: EvalNode, power_opt: Option<EvalNode>) -> EvalExp {

    match power_opt {
      Some(power_node) => {
        let result = match (&base_node, &power_node) {
          (EvalNode::Num(base), EvalNode::Num(power)) => {
            Some(pow(&base, &power))
          },
          (_, _) => None
        };

        match result {
          Some(exp) => exp,
          None => EvalExp::raw(base_node, power_node)
        }
      }
      None => {
        EvalExp {
          base: base_node,
          power: EvalNode::Num(Number::one())
        }
      }
    }


  }

  pub fn sqrt(of: EvalNode) -> EvalExp {
    EvalExp::new(of, Some(EvalNode::Num(Number::new_rational(
        false,
        One::one(),
        BigUint::from(2usize)))))
  }

  /// try to get this exp as a number or else just return the same
  pub fn as_number(self) -> Result<Number, EvalExp> {
    let extract_base = match (&self.base, &self.power) {
      (EvalNode::Num(_), EvalNode::Num(power)) => {
        power.is_one()
      },
      _ => false
    };

    if extract_base {
      if let EvalNode::Num(result) = self.base {
        Ok(result)
      }
      else {
        unimplemented!()
      }
    }
    else {
      Err(self)
    }
  }

  pub fn reciprocate(&mut self) {
    let replacement = match (&self.base, &self.power) {
      (EvalNode::Num(base_num), EvalNode::Num(pow_num)) => {
        if pow_num.is_one() {
          Some(base_num.recip())
        }
        else {
          None
        }
      },
      _ => None
    };

    match replacement {
      Some(num) => {
        self.base = EvalNode::Num(num)
      },
      None => {
        self.power.negate()
      }
    };
  }

}

/// Evaluate raising one number to the power of another.
fn pow(base: &Number, power: &Number) -> EvalExp {

  if power.is_zero() {
    return match base {
      Number::Infinity(_) | Number::NaN => {
        EvalExp::from_number(Number::NaN)
      },
      _ => EvalExp::from_number(Number::one())
    }
  }

  if base.is_zero() {
    return if let Some(pow_neg) = power.is_negative() {
      if pow_neg {
        EvalExp::from_number(Number::Infinity(false))
      }
      else {
        EvalExp::from_number(Number::zero())
      }
    }
    else {
      EvalExp::from_number(power.clone())
    }
  }

  match base {
    Number::Integer(base_neg, base_val) => {

      match power {

        // Integer ^ Integer
        Number::Integer(pow_neg, pow_val) => {
          match int_pow(*base_neg, base_val, *pow_neg, pow_val) {
            Ok((neg, val, recip)) => {
              if recip {
                EvalExp::from_number(
                    Number::new_rational(neg, One::one(), val))
              }
              else {
                EvalExp::from_number(Number::new_integer(neg, val))
              }
            },
            Err(neg) => {
              EvalExp::from_number(Number::ReallyBig(neg))
            }
          }
        },

        // Integer ^ Rational
        Number::Rational(pow_neg, pow_num, pow_denom) => {
          match int_nth_root(*base_neg, base_val, pow_denom) {
            Ok((neg, val)) => {
              match int_pow(neg, &val, *pow_neg, pow_num) {
                Ok((neg, val, recip)) => {
                  if recip {
                    EvalExp::from_number(
                        Number::new_rational(neg, One::one(), val))
                  }
                  else {
                    EvalExp::from_number(
                        Number::new_integer(neg, val))
                  }
                },
                Err(neg) => {
                  EvalExp::from_number(Number::ReallyBig(neg))
                }
              }
            },
            _ => {
              match int_pow(*base_neg, base_val, *pow_neg, pow_num) {
                Ok((neg, val, recip)) => {
                  let base = if recip {
                    Number::new_rational(neg, One::one(), val)
                  }
                  else {
                    Number::new_integer(neg, val)
                  };

                  EvalExp::raw(EvalNode::Num(base),
                      EvalNode::Num(Number::new_rational(false,
                          One::one(),
                          pow_denom.clone())))
                },
                Err(neg) => {
                  let result_base = if *pow_neg {
                    Number::ReallySmall(neg)
                  }
                  else {
                    Number::ReallyBig(neg)
                  };
                  EvalExp::raw(EvalNode::Num(result_base),
                      EvalNode::Num(Number::new_rational(false,
                          One::one(), pow_denom.clone())))
                }
              }
            }
          }
        },

        // Integer ^ Rounded
        Number::Rounded(pow_val) => {
          match base_val.to_f64() {
            Some(base_val) => {
              EvalExp::from_number(Number::new_rounded(
                  base_val.powf(*pow_val)))
            },
            None => {
              EvalExp::from_number(Number::Unknown)
            }
          }
        },

        // Integer ^ Really Big
        Number::ReallyBig(pow_neg) => {
          if *base_neg {
            EvalExp::from_number(Number::Unknown)
          }
          else if *pow_neg {
            EvalExp::from_number(Number::ReallySmall(false))
          }
          else {
            EvalExp::from_number(Number::ReallyBig(false))
          }
        },

        // Integer ^ Really Small or Integer to Unknown
        Number::ReallySmall(_)
            | Number::Unknown=> {
          EvalExp::from_number(Number::Unknown)
        },

        // Integer ^ NaN or Integer ^ Infinity
        Number::NaN | Number::Infinity(_) => {
          EvalExp::from_number(Number::NaN)
        }
      }
    },

    // All cases where base is a rational
    Number::Rational(base_neg, base_num, base_denom) => {

      match power {

        // Rational ^ Integer
        Number::Integer(pow_neg, pow_val) => {
          match int_pow(*base_neg, base_num, *pow_neg, pow_val) {
            Ok((neg, num, recip)) => {
              match int_pow(false, base_denom, false, pow_val) {
                Ok((_, denom, _)) => {
                  if recip {
                    EvalExp::from_number(Number::new_rational(
                        neg, denom, num))
                  }
                  else {
                    EvalExp::from_number(Number::new_rational(
                        neg, num, denom))
                  }
                },
                Err(_) => EvalExp::from_number(Number::Unknown)
              }
            },
            Err(_) => EvalExp::from_number(Number::Unknown)
          }
        },

        // Rational ^ Rational
        Number::Rational(pow_neg, pow_num, pow_denom) => {

          let num_root_denom = match int_nth_root(
              *pow_neg, base_num, pow_denom) {
            Ok((neg, val)) => Some((neg, val)),
            _ => None
          };

          let denom_root_denom = match int_nth_root(
              false, base_denom, pow_denom) {
            Ok((_, val)) => Some(val),
            _ => None
          };

          let num_to_num = match (&num_root_denom, &denom_root_denom) {
            (Some((neg, num_root)), Some(_)) => {
              match int_pow(*neg, num_root, *pow_neg, pow_num) {
                Ok(tup) => Some(tup),
                _ => None
              }
            },
            _ => {
              match int_pow(*base_neg, base_num, *pow_neg, pow_num) {
                Ok(tup) => Some(tup),
                _ => None
              }
            }
          };

          let denom_to_num = match (&num_root_denom, &denom_root_denom) {
            (Some(_), Some(denom)) => {
              match int_pow(false, denom, *pow_neg, pow_denom) {
                Ok((_, val, _)) => Some(val),
                _ => None
              }
            },
            _ => {
              match int_pow(false, base_denom, *pow_neg, pow_denom) {
                Ok((_, val, _)) => Some(val),
                _ => None
              }
            }
          };

          match (
              num_root_denom,
              denom_root_denom,
              num_to_num,
              denom_to_num) {

            (Some(_), Some(_), Some((neg, num, recip)), Some(denom)) => {
              if recip {
                EvalExp::from_number(
                    Number::new_rational(neg, denom, num))
              }
              else {
                EvalExp::from_number(
                    Number::new_rational(neg, num, denom))
              }
            },
            (_, _, Some((neg, num, recip)), Some(denom)) => {
              let base = if recip {
                Number::new_rational(neg, denom, num)
              }
              else {
                Number::new_rational(neg, num, denom)
              };
              let pow = Number::new_rational(
                  false,
                  One::one(),
                  pow_denom.clone());

              EvalExp::raw(EvalNode::Num(base),
                  EvalNode::Num(pow))
            },
            _ => {
              EvalExp::from_number(Number::Unknown)
            }
          }
        },

        // Rational ^ Rounded
        Number::Rounded(pow_val) => {
          match base_num.to_f64() {
            Some(num) => {
              match base_denom.to_f64() {
                Some(denom) => {
                  let base
                      = if *base_neg { -1. } else { 1. } * num / denom;
                  EvalExp::from_number(
                      Number::new_rounded(base.powf(*pow_val)))
                },
                _ => EvalExp::from_number(Number::Unknown)
              }
            },
            _ => EvalExp::from_number(Number::Unknown)
          }
        },

        // Rational ^ ReallyBig, ReallySmall, or Unkown
        Number::ReallyBig(_)
            | Number::ReallySmall(_)
            | Number::Unknown => {
          EvalExp::from_number(Number::Unknown)
        },

        // Rational ^ Infinitiy, NaN
        Number::Infinity(_) | Number::NaN => {
          EvalExp::from_number(Number::NaN)
        }
      }
    },

    // Handle all cases where we raise a rounded number to a power
    Number::Rounded(base_val) => {

      match power {

        // Rounded ^ Integer
        Number::Integer(pow_neg, pow_val) => {

          match pow_val.to_f64() {
            Some(pow_f64) => {
              EvalExp::from_number(Number::new_rounded(
                  base_val.powf(pow_f64
                      * if *pow_neg { -1. } else { 1. })))
            },
            _ => EvalExp::from_number(Number::Unknown)
          }
        },

        // Rounded * Rational
        Number::Rational(pow_neg, pow_num, pow_denom) => {
          match (pow_num.to_f64(), pow_denom.to_f64()) {
            (Some(pow_num_f64), Some(pow_denom_f64)) => {
              let pow = if *pow_neg { -1. } else { 1. }
                  * pow_num_f64 / pow_denom_f64;
              EvalExp::from_number(Number::new_rounded(
                  base_val.powf(pow)))
            }
            _ => EvalExp::from_number(Number::Unknown)
          }
        },

        // Rounded ^ Rounded
        Number::Rounded(pow_val) => {
          EvalExp::from_number(Number::new_rounded(
              base_val.powf(*pow_val)))
        },

        // Rounded ^ ReallyBig, ReallySmall, Unknown
        Number::ReallyBig(_)
            | Number::ReallySmall(_)
            | Number::Unknown => {
          EvalExp::from_number(Number::Unknown)
        },

        // Rounded ^ Infinity, NaN
        Number::Infinity(_) | Number::NaN => {
          EvalExp::from_number(Number::NaN)
        }
      }
    },

    // Handle all cases where we raise a Really big number to a power
    Number::ReallyBig(base_neg) => {

      match power {
        // Really Big ^ Integer, Really Big
        Number::Integer(pow_neg, pow_val) => {
          let neg = *base_neg && pow_val.is_odd();

          EvalExp::from_number(if *pow_neg {
            Number::ReallySmall(neg)
          }
          else {
            Number::ReallyBig(neg)
          })
        },

        // Really Big ^ Rational, Rounded, Really Small, Unknown
        Number::Rational(_, _, _)
            | Number::Rounded(_)
            | Number::ReallySmall(_)
            | Number::Unknown => {
          EvalExp::from_number(Number::Unknown)
        },

        // Really Big ^ Really Big
        Number::ReallyBig(pow_neg) => {
          if *base_neg {
            EvalExp::from_number(Number::Unknown)
          }
          else {
            EvalExp::from_number(if *pow_neg {
              Number::ReallySmall(false)
            }
            else {
              Number::ReallyBig(false)
            })
          }
        },

        // Really Big ^ NaN, Infinity
        Number::Infinity(_) | Number::NaN => {
          EvalExp::from_number(Number::NaN)
        }
      }

    },

    // Handle raising a really small number to a power
    Number::ReallySmall(base_neg) => {

      match power {

        // Really Small ^ Integer
        Number::Integer(pow_neg, pow_val) => {
          let neg = *base_neg && pow_val.is_odd();

          EvalExp::from_number(if *pow_neg {
            Number::ReallyBig(neg)
          }
          else {
            Number::ReallySmall(neg)
          })
        },

        // Really Small ^ Really Big
        Number::ReallyBig(pow_neg) => {
          if *base_neg {
            EvalExp::from_number(Number::Unknown)
          }
          else {
            EvalExp::from_number(if *pow_neg {
              Number::ReallyBig(false)
            }
            else {
              Number::ReallySmall(false)
            })
          }
        },

        // Really Small ^ Rational, Rounded, Really Small, Unknown
        Number::Rational(_, _, _)
            | Number::Rounded(_)
            | Number::ReallySmall(_)
            | Number::Unknown => {
          EvalExp::from_number(Number::Unknown)
        },

        // Really Small ^ NaN, Infinity
        Number::Infinity(_) | Number::NaN => {
          EvalExp::from_number(Number::NaN)
        }
      }
    },

    // Handle raising an infinite number to a power
    Number::Infinity(base_neg) => {

      match power {

        // Infinity ^ Integer
        Number::Integer(pow_neg, pow_val) => {
          if *pow_neg {
            EvalExp::from_number(Number::zero())
          }
          else {
            EvalExp::from_number(Number::Infinity(
              *base_neg && pow_val.is_odd()))
          }
        },

        // Infinity ^ Rational
        Number::Rational(pow_neg, _, _) => {
          if *pow_neg {
            EvalExp::from_number(Number::zero())
          }
          else if *base_neg {
            EvalExp::from_number(Number::Unknown)
          }
          else {
            EvalExp::from_number(Number::Infinity(false))
          }
        },

        // Infinity ^ Rounded
        Number::Rounded(pow_val) => {
          if *pow_val < 0. {
            EvalExp::from_number(Number::zero())
          }
          else if *base_neg {
            EvalExp::from_number(Number::Unknown)
          }
          else {
            EvalExp::from_number(Number::Infinity(false))
          }
        },

        // Infinity ^ Really Big
        Number::ReallyBig(pow_neg)
            | Number::ReallySmall(pow_neg) => {

          if *base_neg {
            EvalExp::from_number(Number::Unknown)
          }
          else if *pow_neg {
            EvalExp::from_number(Number::zero())
          }
          else {
            EvalExp::from_number(Number::Infinity(false))
          }
        },

        // Infinity ^ Unkown
        Number::Unknown => EvalExp::from_number(Number::Unknown),

        // Infinity ^ Infinity, NaN
        Number::Infinity(_) | Number::NaN => {
          EvalExp::from_number(Number::NaN)
        }
      }
    },

    Number::Unknown => EvalExp::from_number(Number::Unknown),
    Number::NaN => EvalExp::from_number(Number::NaN)
  }
}

fn int_nth_root(base_neg: bool, base_val: &BigUint, root_val: &BigUint)
        -> Result<(bool, BigUint), ()> {

  let mut neg = false;

  if base_neg {
    if root_val.is_odd() {
      return Err(());
    }
    neg = true;
  }

  if let Some(root32) = root_val.to_u32() {
    if let Some(base64) = base_val.to_u64() {
      let approx_root = num::integer::nth_root(base64, root32);

      if num::pow(approx_root, root32 as usize) == base64 {
        Ok((neg, BigUint::from(approx_root)))
      }
      else {
        Err(())
      }
    }
    else {
      Err(())
    }
  }
  else {
    Err(())
  }

}

/// Handle the power between two big ints that have their sign split
/// out.  lhs ^ rhs = (overall negative, value, reciproccal) where
/// both l_val and r_val > 0
fn int_pow(l_neg: bool, l_val: &BigUint, r_neg: bool, r_val: &BigUint)
    -> Result<(bool, BigUint, bool), bool> {

  // Overall sign is negative iff the left side is negative and if the
  // right side is odd
  let neg = !l_neg && (r_val % 2usize).is_one();

  if l_val.is_one() {
    Ok((neg, One::one(), false))
  }
  else {
    r_val.to_usize().map(move |pow_val| {
      let val = num::pow(l_val.clone(), pow_val);
      Ok((neg, val, r_neg))
    })
    .unwrap_or(Err(neg))
  }

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
    excamations: many0!(ws!(tag!("!"))) >>
    (wrap_radicals_and_factorials(radicals, fac, excamations))
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
        prod_term.push(EvalExp::new(
            EvalNode::Num(Number::negative_one()), None));
        eval_acc.push(prod_term);
        (new_expr, eval_acc)
      },
      _ => panic!("Only Addition and Subtraction operations allowed")
    }
  })
}

fn fold_mult_div_expr(init: (Expr,EvalExp),
    remainder: Vec<(Oper,(Expr,EvalExp))>) -> (Expr, EvalProd) {

  let (init_expr, init_exp) = init;
  let mut init_prod = EvalProd::new();
  init_prod.push(init_exp);
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
        -> (Expr, EvalExp) {

  remainder.insert(0, initial);
  let (init_exp, init_node) = remainder.pop().unwrap();

  let initial = (init_exp, EvalExp::new(init_node, None));

  remainder.into_iter().rev().fold(initial, |acc, (expr, node)| {
    let (expr_acc, eval_acc) = acc;
    let new_expr = Exp(Box::new(expr), Box::new(expr_acc));
    let new_eval = EvalExp::new(node,
        Some(EvalNode::from_statement(Evaluable::new_from_exp(eval_acc))));
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
    result_node = EvalNode::new_factorial(result_node);
  }

  (result_expr, result_node)
}

named!(exp_term<CompleteStr, (Expr,EvalExp)>, do_parse!(
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
    fn test_sqrt_1() {
        let parsed = parse("(√ 2 + 3) / 5.6").unwrap().0;
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
        let parsed = parse("√ (2 ^ 3) / 5.6").unwrap().0;
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
        let parsed = parse("√ √ √ 2").unwrap().0;
        assert_eq!(parsed,
          Radical(Box::new(
              Radical(Box::new(
                  Radical(num("2")))))));
    }

    #[test]
    fn test_simple_factorial() {
        let parsed = parse(" 2. !").unwrap().0;
        assert_eq!(parsed, Factorial(num("2.")));
    }

    #[test]
    fn test_factorial_1() {
        let parsed = parse("( 2 + 3 ! ) / 5.6").unwrap().0;
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
        let parsed = parse("(2 ^ 3) ! / 5.6").unwrap().0;
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
        let parsed = parse("2 !! !").unwrap().0;
        assert_eq!(parsed,
            Factorial(Box::new(
                Factorial(Box::new(
                    Factorial(num("2")))))));
    }
}