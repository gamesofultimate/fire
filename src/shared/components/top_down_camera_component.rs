use engine::utils::units::Meters;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate};

/// Exposes variables for the top down camera to the scene editor.
#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema, Duplicate)]
pub struct TopDownCameraComponent {
  #[schema(default = "{meters: 15.0}")]
  pub camera_height: Meters,
  #[schema(default = "{meters: 7.0}")]
  pub camera_back_offset: Meters,
}
