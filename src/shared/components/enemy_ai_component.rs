use crate::shared::game_types::game_types::EnemyState;
use engine::{
  application::scene::PrefabId,
  utils::units::{Meters, Mps, Rps, Seconds},
};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct EnemyAiComponent {
  #[serde(skip)]
  pub state: EnemyState,
  #[schema(default = "{mps: 8.0}")]
  pub speed: Mps,
  #[schema(default = "{rps: 1.0}")]
  pub rotation_speed: Rps,
  #[schema(default = "{meters: 20.0}")]
  pub detection_radius: Meters,
  #[schema(default = "{meters: 5.0}")]
  pub attack_range: Meters,
  #[schema(default = "{seconds: 0.0}")]
  pub timer: Seconds,
  #[schema(default = "{seconds: 4.0}")]
  pub attack_cooldown: Seconds,
  #[serde(skip)]
  spawned_from: PrefabId,
  #[serde(skip)]
  pub pacing_direction: Option<Vector3<f32>>,
  #[serde(skip)]
  pub pacing_time_remaining: Seconds,
  #[serde(skip)]
  pub pacing_speed: Mps,
}

impl EnemyAiComponent {
  pub fn new() -> Self {
    Self {
      state: EnemyState::Idle,
      speed: Mps::new(10.0),
      rotation_speed: Rps::new(1.0),
      detection_radius: Meters::new(20.0),
      attack_range: Meters::new(5.0),
      timer: Seconds::new(0.0),
      attack_cooldown: Seconds::new(4.0),
      spawned_from: PrefabId::new(),
      pacing_direction: None,
      pacing_time_remaining: Seconds::new(0.0),
      pacing_speed: Mps::new(5.0),
    }
  }

  pub fn reset_timer(&mut self) {
    self.timer = Seconds::new(0.0);
  }

  // Getter which returns the good Uuid of the entity that spawned this enemy
  pub fn spawned_from(&self) -> PrefabId {
    self.spawned_from
  }

  // Setter which sets the Uuid of the entity that spawned this enemy
  pub fn set_spawned_from(&mut self, id: PrefabId) {
    self.spawned_from = id;
  }
}
