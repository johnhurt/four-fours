
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use std::ops::{
  Add
};

use math::{
  Number,
  EvalExp,
  EvalFunc,
  EvalProdTerm,
  EvalProd,
  EvalSum,
};

use num::{
  BigUint,
  One,
  ToPrimitive,
  Integer
};

use statrs::function::gamma::gamma;

#[derive(Clone, PartialEq, PartialOrd)]
pub enum EvalNode {
  Num(Number),
  Exp(EvalExp),
  Func(EvalFunc),
  Prod(EvalProd),
  Sum(EvalSum)
}


#[derive(Clone)]
pub enum EvalProdTerm {
  NumTerm(Number),
  ExpTerm(EvalExp),
  FuncTerm(EvalFunc)
}

#[derive(Clone)]
pub enum EvalFunc {
  Factorial(Box<EvalNode>)
}


use self::Number::*;
use self::EvalNode::*;
use self::EvalProdTerm::*;
use self::EvalFunc::*;

impl Default for EvalNode {
  fn default() -> EvalNode {
    Num(Number::zero())
  }
}

/********************** Display / Debug implementations ***************/

impl Debug for EvalFunc {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      Factorial(ref inner) => write!(format, "({:?})!", *inner)
    }
  }
}

impl Debug for EvalNode {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      Num(ref inner) => Debug::fmt(inner, format),
      Exp(ref inner) => Debug::fmt(inner, format),
      Func(ref inner) => Debug::fmt(inner, format),
      Prod(ref inner) => Debug::fmt(inner, format),
      Sum(ref inner) => Debug::fmt(inner, format)
    }
  }
}

impl Debug for EvalProdTerm {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      NumTerm(ref inner) => Debug::fmt(inner, format),
      ExpTerm(ref inner) => Debug::fmt(inner, format),
      FuncTerm(ref inner) => Debug::fmt(inner, format)
    }
  }
}

impl Display for EvalFunc {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      EvalFunc::Factorial(ref inner) => write!(format, "({:?})!", *inner)
    }
  }
}

impl Display for EvalNode {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      Num(ref inner) => Debug::fmt(inner, format),
      Exp(ref inner) => Debug::fmt(inner, format),
      Func(ref inner) => Debug::fmt(inner, format),
      Prod(ref inner) => Debug::fmt(inner, format),
      Sum(ref inner) => Debug::fmt(inner, format)
    }
  }
}

impl Display for EvalProdTerm {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    match *self {
      NumTerm(ref inner) => Debug::fmt(inner, format),
      ExpTerm(ref inner) => Debug::fmt(inner, format),
      FuncTerm(ref inner) => Debug::fmt(inner, format)
    }
  }
}


/************************ Routing Implementations *************************/

/// Okay, I think I got it this time, this is an implementation of add for all
/// node to nodes
impl Add for EvalNode {
  type Output = EvalNode;

  fn add(self, otherNode: EvalNode) -> EvalNode {
    match otherNode {
      Num(num) => self.add(num),
      Exp(exp) => self.add(exp),
      Func(func) => self.add(func),
      Prod(prod) => self.add(prod),
      Sum(sum) => self.add(sum)
    }
  }
}

/// Impl for add between node and Sum
impl Add<EvalSum> for EvalNode {
  type Output = EvalNode;

  fn add(self, otherS: EvalSum) -> EvalNode {
    unimplemented!();
  }
}

/// Impl for add between node and Product
impl Add<EvalProd> for EvalNode {
  type Output = EvalNode;

  fn add(self, other: EvalProd) -> EvalNode {
    unimplemented!();
  }
}

/// Impl for add between node and ProdTerm
impl Add<EvalProdTerm> for EvalNode {
  type Output = EvalNode;

  fn add(self, other: EvalProdTerm) -> EvalNode {
    match other {
      NumTerm(numTerm) => self.add(numTerm),
      FuncTerm(funcTerm) => self.add(funcTerm),
      ExpTerm(expTerm) => self.add(expTerm)
    }
  }
}

/// Impl for add between node and Exponent
impl Add<EvalExp> for EvalNode {
  type Output = EvalNode;

  fn add(self, other: EvalExp) -> EvalNode {
    unimplemented!();
  }
}

/// Impl for add between node and Function call
impl Add<EvalFunc> for EvalNode {
  type Output = EvalNode;

  fn add(self, other: EvalSum) -> EvalNode {
    unimplemented!();
  }
}

/// Impl for add between node and Number
impl Add<Number> for EvalNode {
  type Output = EvalNode;

  fn add(self, other: Number) -> EvalNode {
    unimplemented!();
  }
}
