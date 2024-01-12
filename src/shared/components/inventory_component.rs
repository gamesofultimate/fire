use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct InventoryComponent {
  #[schema(default = "0")]
  pub wood: u32,
  #[schema(default = "5")]
  pub max_wood: u32,
}

impl InventoryComponent {
  pub fn add_wood(&mut self, amount: u32) {
    self.wood += amount;
  }

  pub fn remove_wood(&mut self, amount: u32) {
    self.wood -= amount;
  }

  pub fn get_wood(&self) -> u32 {
    self.wood
  }

  pub fn is_full(&self) -> bool {
    self.wood >= self.max_wood
  }
}
