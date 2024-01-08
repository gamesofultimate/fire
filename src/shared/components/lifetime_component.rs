use engine::utils::units::Seconds;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

/// Manages health values
#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema)]
pub struct LifetimeComponent {
  #[schema(default = "{Seconds: 3.0}")]
  pub duration: Seconds,

  #[serde(skip)]
  pub is_running: bool,

  #[serde(skip)]
  pub timer: Seconds,
}

impl LifetimeComponent {}
