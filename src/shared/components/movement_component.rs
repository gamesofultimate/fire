use engine::utils::units::{Mps, Rps, Seconds};
use nalgebra::Point3;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate, factory::Duplicate as Dup};
use crate::shared::game_types::game_types::TargetType;

/// Specifies that an entity can move with the given speeds.
#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable, Duplicate)]
pub struct MovementComponent {
  #[schema(default = "{ mps: 10.0}")]
  pub run_speed: Mps,
  #[schema(default = "{ rps: 10.0}")]
  pub rotation_speed: Rps,
  #[serde(skip)]
  pub dash_timer: Seconds,
  #[serde(skip)]
  pub target_point: Option<Point3<f32>>,
  #[serde(skip)]
  pub target_type: Option<TargetType>,
}
