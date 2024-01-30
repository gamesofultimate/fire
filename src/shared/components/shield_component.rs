use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate};

/// Manages shield values and timers
#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema, Duplicate)]
pub struct ShieldComponent {
  pub shield: f32,
  pub max_shield: f32,
  pub shield_regen_per_second: f32,
  pub current_undamaged_duration: f32,

  #[serde(skip)]
  pub time_last_damage: f32,
}

impl ShieldComponent {}
