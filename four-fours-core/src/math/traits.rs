
/// Trait for entities that can be negated, but not in place
pub trait Negate : Clone {
  fn negate(self) -> Self;

  fn negate_clone(&self) -> Self {
    self.clone().negate()
  }
}

/// Trait for entities that can be reciprocated, but not in place
pub trait Recip : Clone {
  fn recip(self) -> Self;

  fn recip_clone(&self) -> Self {
    self.clone().recip()
  }
}

pub trait AddHash {

  fn hash_add(&self) -> usize;

}