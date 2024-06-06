use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimingRange {
  pub min: u32,
  pub max: u32,
}

impl TimingRange {
  pub fn new(min: u32, max: u32) -> Self {
    TimingRange { min, max }
  }

  pub fn generate(&self) -> u32 {
    let distribution = Uniform::from(self.min..self.max);
    let mut rng = rand::thread_rng();
    distribution.sample(&mut rng)
  }
}

impl Default for TimingRange {
  fn default() -> Self {
    TimingRange::new(150, 300)
  }
}
