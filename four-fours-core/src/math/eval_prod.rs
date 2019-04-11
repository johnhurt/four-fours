
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
  EvalSum,
  EvalNode
};

#[derive(Clone, PartialEq, Default)]
pub struct EvalProd {
  terms: Vec<EvalProdTerm>
}

impl EvalProd {

  fn push(&mut self, prod_term: EvalProdTerm) {
    self.terms.push(prod_term)
  }

}

impl Display for EvalProd {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

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

impl Debug for EvalProd {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {
    let mut count = 0usize;

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