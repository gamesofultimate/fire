use engine::utils::units::{Mps, Rps, Seconds};
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

/// Specifies that an entity can move with the given speeds.
#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct MovementComponent {
  #[schema(default = "{ mps: 50.0}")]
  pub run_speed: Mps,
  #[schema(default = "{ rps: 10.0}")]
  pub rotation_speed: Rps,
  #[serde(skip)]
  pub dash_timer: Seconds,
}
