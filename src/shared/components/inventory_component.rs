use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct InventoryComponent {
  #[serde(skip)]
  pub dreamstones: u32,
}

impl InventoryComponent {
  pub fn new() -> Self {
    InventoryComponent { dreamstones: 0 }
  }

  pub fn add_dreamstones(&mut self, amount: u32) {
    self.dreamstones += amount;
  }

  pub fn remove_dreamstones(&mut self, amount: u32) {
    self.dreamstones -= amount;
  }

  pub fn get_dreamstones(&self) -> u32 {
    self.dreamstones
  }
}
