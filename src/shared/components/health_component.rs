use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate};

/// Manages health values

#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema, Duplicate)]
pub struct HealthComponent {
  pub health: f32,
  pub max_health: f32,
  #[serde(skip)]
  pub pending_damage: f32,
}

impl HealthComponent {}
