use engine::utils::units::Seconds;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate};

// Manages resource values
#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema, Duplicate)]
pub struct ResourceComponent {
  #[schema(default = "10")]
  pub available: u32,
  #[schema(default = "10")]
  pub max: u32,
  #[schema(default = "10")]
  pub regen_amount: u32,
  #[schema(default = "{seconds: 30}")]
  pub regen_interval: Seconds,
}

impl ResourceComponent {
  pub fn gather_wood(&mut self) {
    self.available -= 1;
  }

  pub fn restore_wood(&mut self) {
    self.available += self.regen_amount;
  }

  pub fn increase_max(&mut self, amount: u32) {
    self.max += amount;
  }

  pub fn decrease_max(&mut self, amount: u32) {
    self.max -= amount;
  }
}
