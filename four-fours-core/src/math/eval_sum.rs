
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
  EvalNode
};

use num::{
  BigUint,
  One,
  ToPrimitive,
  Integer
};

#[derive(Clone, PartialEq, Default)]
pub struct EvalSum {
  terms: Vec<EvalProd>
}

impl EvalSum {
  pub fn push(&mut self, eval_prod: EvalProd) {

  }
}

impl Debug for EvalSum {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

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

impl Display for EvalSum {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

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