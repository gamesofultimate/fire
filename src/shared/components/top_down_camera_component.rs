use engine::utils::units::Seconds;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

/// Exposes variables for the top down camera to the scene editor.
#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema)]
pub struct TopDownCameraComponent {
  #[schema(default = "{}")]
  pub angle: f32,
}
