use engine::utils::units::{Mps, Newtons, Rps, Seconds};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

/// Specifies that an entity can move with the given speeds.
#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct MovementComponent {
  #[schema(default = "{ mps: 8.0}")]
  pub sprint_speed: Mps,
  #[schema(default = "{ mps: 4.0}")]
  pub walk_speed: Mps,
  #[schema(default = "{ mps: 50.0}")]
  pub dash_speed: Mps,
  #[schema(default = "{ rps: 10.0}")]
  pub rotation_speed: Rps,
  #[schema(default = "{ newtons: 30.0}")]
  pub jump_force: Newtons,
  #[schema(default = "100.0")]
  pub knockback_intensity: f32,
  #[serde(skip)]
  pub dash_timer: Seconds,
  #[serde(skip)]
  pub is_dashing: bool,
  #[serde(skip)]
  pub is_grounded: bool,
  #[serde(skip)]
  pub air_attack: bool,
  #[serde(skip)]
  pub knockback_direction: Vector3<f32>,
}

impl MovementComponent {
  pub fn start_dash(&mut self) {
    self.is_dashing = true;
    self.dash_timer = Seconds::zero();
  }

  pub fn stop_dash(&mut self) {
    self.is_dashing = false;
  }

  pub fn jump(&mut self) {
    self.is_grounded = false;
  }

  pub fn land(&mut self) {
    self.is_grounded = true;
  }
}
