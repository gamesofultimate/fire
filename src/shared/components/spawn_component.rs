use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};
// might need an is_active bool.
// todo: switch radius to Meters type

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct SpawnComponent {
  #[schema(default = "10.0")]
  pub radius: f32,
  #[schema(default = "5.0")]
  pub interval: f32,
  #[serde(skip)]
  pub timer: f32,
  #[schema(default = "25")]
  pub max_enemies: usize,
  #[serde(skip)]
  spawn_count: usize,
}

impl SpawnComponent {
  pub fn spawn_enemy(&mut self) {
    self.spawn_count += 1;
  }

  pub fn remove_enemy(&mut self) {
    self.spawn_count -= 1;
  }

  pub fn spawn_count(&self) -> usize {
    self.spawn_count
  }
}
