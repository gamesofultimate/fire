use engine::utils::units::{Mps, Rps, Seconds};
use nalgebra::{Vector3, Point3};
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate};
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
  pub target_point: Option<Vector3<f32>>,
  #[serde(skip)]
  pub target_type: Option<TargetType>,
}


impl MovementComponent {
  pub fn vector3_to_point3(vector: Vector3<f32>) -> Point3<f32> {
  Point3::new(vector.x, vector.y, vector.z)
}

pub fn point3_to_vector3(point: Point3<f32>) -> Vector3<f32> {
  Vector3::new(point.x, point.y, point.z)
}

}
