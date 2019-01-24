
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use math::Number;

use num::{
  BigUint,
  One,
  ToPrimitive,
  Integer
};

use statrs::function::gamma::gamma;

#[derive(Clone)]
pub enum EvalNode {
  Num(Number),
  Statement(Evaluable)
}

#[derive(Clone)]
pub struct EvalExp {
  base: EvalNode,
  power: EvalNode
}

#[derive(Clone)]
pub struct EvalProd {
  exp_terms: Vec<EvalExp>,
  eval_terms: Vec<EvalNode>,
  number_collector: Number
}

pub enum EvalFunc {
  Factorial(Box<EvalNode>)
}

#[derive(Clone)]
pub struct Evaluable {
  terms: Vec<EvalProd>,
  number_collector: Number
}

impl Debug for EvalFunc {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      EvalFunc::Factorial(ref inner) => write!(format, "({:?})!", *inner)
    }
  }
}

impl Debug for EvalNode {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      EvalNode::Num(ref val) => Debug::fmt(val, format),
      EvalNode::Statement(ref stmt) => Debug::fmt(stmt, format)
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

    for term in &self.exp_terms {
      if count > 0 {
        write!(format, "*")?;
      }

      Debug::fmt(term, format)?;

      count += 1;
    }

    for term in &self.eval_terms {
      if count > 0 {
        write!(format, "*")?;
      }
      write!(format, "(");
      Debug::fmt(term, format);
      write!(format, ")");

      count += 1;
    }

    Ok(())
  }
}

impl Debug for Evaluable {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

    if !self.number_collector.is_zero() {
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

    for term in &self.exp_terms {
      if count > 0 {
        write!(format, "*")?;
      }

      Debug::fmt(term, format);

      count += 1;
    }

    for term in &self.eval_terms {
      if count > 0 {
        write!(format, "*")?;
      }
      write!(format, "(");
      Debug::fmt(term, format);
      write!(format, ")");

      count += 1;
    }

    Ok(())
  }
}

impl Display for Evaluable {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

    if !self.number_collector.is_zero() {
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

  /// modify this node into it's negative
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

  pub fn product(left: EvalNode, right: EvalNode) -> EvalNode {

    match (left, right) {
      (EvalNode::Num(left_val), EvalNode::Num(right_val)) => {
        EvalNode::Num(left_val.multiply(&right_val))
      },
      (EvalNode::Num(val), EvalNode::Statement(stmt))
          | (EvalNode::Statement(stmt), EvalNode::Num(val)) => {
        EvalNode::Statement(stmt.scale(val))
      },
      (EvalNode::Num(val), EvalNode::Factorial(inner))
          | (EvalNode::Factorial(inner), EvalNode::Num(val)) => {
        let mut prod = EvalProd::new_from_exp(EvalExp::from_number(val))
        prod.
        EvalNode::Statement(stmt.scale(val))
      },


    }
  }

  pub fn to_f64(&self) -> f64 {
    match self {
      EvalNode::Statement(stmt) => {
        stmt.to_f64()
      },
      EvalNode::Num(val) => {
        val.to_f64()
      },
      EvalNode::Factorial(inner) => gamma(inner.to_f64())
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

  /// Scale this evaluable by the given factor
  pub fn scale(mut self, by: Number) -> Evaluable {
    self.number_collector = self.number_collector.multiply(&by);
    self.terms = self.terms.into_iter()
        .map(|term| {
          term.push(EvalExp::from_number(by.clone()));
          term
        })
        .collect();
    self
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

  pub fn to_f64(&self) -> f64 {
    let result : f64 = self.terms.iter().map(EvalProd::to_f64).sum();
    result + self.number_collector.to_f64()
  }

  pub fn evaluate(self) -> Number {

    if self.terms.is_empty() {
      self.number_collector
    }
    else {
      println!("{}", self);
      Number::new_rounded(self.to_f64())
    }
  }
}

impl EvalProd {

  pub fn new() -> EvalProd {
    EvalProd {
      exp_terms: Vec::new(),
      eval_terms: Vec::new(),
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
          self.eval_terms.clear();
          self.exp_terms.clear();
        }
        else {
          self.number_collector
              = self.number_collector.multiply(&val);
        }
      },
      Err(term) => {
        self.exp_terms.push(term)
      }
    }
  }

  pub fn push(self, eval: EvalNode) ->

  /// Get the this product as a single exp term or an err with this
  /// in it
  pub fn as_exp(mut self) -> Result<EvalExp,EvalProd> {
    if self.number_collector.is_zero()
        || ( self.exp_terms.is_empty() && self.eval_terms.is_empty() ) {
      Ok(EvalExp::new(EvalNode::Num(self.number_collector), None))
    }
    else if self.number_collector.is_one()
        && self.exp_terms.len() == 1
        && self.eval_terms.is_empty() {
      Ok(self.exp_terms.pop().unwrap())
    }
    else {
      Err(self)
    }
  }

  pub fn negate(&mut self) {
    self.number_collector
        = self.number_collector.multiply(&Number::negative_one());
  }

  /// Get the best f64 representation of this product
  pub fn to_f64(&self) -> f64 {
    let exp_prod : f64 = self.exp_terms.iter().map(EvalExp::to_f64).product();
    let eval_prod : f64 = self.eval_terms.iter().map(EvalNode::to_f64).product();
    exp_prod * eval_prod * self.number_collector.to_f64()
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

  pub fn new(base_node: EvalNode, power_opt: Option<EvalNode>) -> EvalExp {

    match base_node {
      EvalNode::Statement(stmt) => {
        match stmt.as_prod() {
          Ok(prod) => {
            match prod.as_exp() {
              Ok(exp) => {
                match exp.as_number() {
                  Ok(base_val) => {
                    match power_opt {
                      Some(power) => {
                        match &power {
                          EvalNode::Num(pow_val) => {
                            pow(&base_val, pow_val)
                          },
                          _ => {
                            EvalExp::raw(EvalNode::Num(base_val), power)
                          }
                        }
                      },
                      None => {
                        EvalExp::from_number(base_val)
                      }
                    }
                  },
                  Err(exp) => {
                    match power_opt {
                      Some(power) => {

                      },
                      None => {
                        exp
                      }
                    }
                  }
                }
              },
              Err(prod) => {
                EvalExp {
                  base: EvalNode::Statement(Evaluable::new_from_prod(prod)),
                  power: EvalNode::Num(Number::one())
                }
              }
            }
          },
          Err(stmt) => {
            EvalExp {
              base: EvalNode::Statement(stmt),
              power: EvalNode::Num(Number::one())
            }
          }
        }
      },
      _ => {
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

  pub fn to_f64(&self) -> f64 {
    self.base.to_f64().powf(self.power.to_f64())
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

  if base.is_one() {
    return EvalExp::from_number(Number::one());
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
  let neg = l_neg && (r_val % 2usize).is_one();

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
