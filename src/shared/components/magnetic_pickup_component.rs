use engine::utils::units::{Meters, Mps, Rps};
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct MagneticPickupComponent {
  #[schema(default = "{mps: 3.0}")]
  pub speed: Mps,
  #[schema(default = "{rps: 1.0}")]
  pub rotation_speed: Rps,
  #[schema(default = "{meters: 3.0}")]
  pub detection_radius: Meters,
}

impl MagneticPickupComponent {}
