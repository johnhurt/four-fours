
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use std::ops::{
  Add
};

use math::{
  Number,
  EvalNode
};

use math::EvalNode::*;

#[derive(Clone, PartialEq, Default)]
pub struct EvalExp {
  base: Box<EvalNode>,
  power: Box<EvalNode>
}

impl Display for EvalExp {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {

    let show_pow = if let Num(pow_val) = *self.power {
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

impl Debug for EvalExp {
  fn fmt(&self, format: &mut Formatter) -> fmt::Result {

    let show_pow = if let Num(pow_val) = *self.power {
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

/********************** Add Implementations **********************/

/// EvalExp + EvalExp
impl Add<EvalExp> for EvalExp {
  type Output = EvalNode;

  fn add(self, otherExp: EvalExp) -> EvalNode {
    if self == otherExp {
      self * Number::two()
    }
    else {
      EvalSum::default()
          + (EvalProd::default() * self)
          + (EvalProd::default() * self)
    }
  }
}