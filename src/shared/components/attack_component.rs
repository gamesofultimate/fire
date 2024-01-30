use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate};

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable, PartialEq, Eq, Duplicate)]
pub enum AttackType {
  None,
  Light,
  Heavy,
  Air,
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable, PartialEq, Duplicate)]
pub struct AttackTypeDamage {
  pub attack_type: AttackType,
  pub damage_multiplier: f32,
}

pub const NO_ATTACK: AttackTypeDamage = AttackTypeDamage {
  attack_type: AttackType::None,
  damage_multiplier: 0.0,
};

pub const LIGHT_ATTACK: AttackTypeDamage = AttackTypeDamage {
  attack_type: AttackType::Light,
  damage_multiplier: 1.0,
};

pub const HEAVY_ATTACK: AttackTypeDamage = AttackTypeDamage {
  attack_type: AttackType::Heavy,
  damage_multiplier: 3.0,
};

pub const AIR_ATTACK: AttackTypeDamage = AttackTypeDamage {
  attack_type: AttackType::Air,
  damage_multiplier: 3.0,
};

impl Default for AttackTypeDamage {
  fn default() -> Self {
    AttackTypeDamage {
      attack_type: AttackType::None,
      damage_multiplier: 0.0,
    }
  }
}

/// Manages attack values for combat system
#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct AttackComponent {
  #[schema(default = "0.0")]
  pub cooldown: f32,
  #[schema(default = "2.0")]
  pub heavy_timer_max: f32,
  #[schema(default = "5.0")]
  pub damage: f32,
  #[schema(default = "3.0")]
  pub max_distance: f32,
  #[schema(default = "600.0")]
  pub light_anim_start_time: f32,
  #[schema(default = "1800.0")]
  pub light_anim_end_time: f32,
  #[schema(default = "1200.0")]
  pub heavy_anim_start_time: f32,
  #[schema(default = "2200.0")]
  pub heavy_anim_end_time: f32,

  #[serde(skip)]
  pub attack_type_damage: AttackTypeDamage,

  #[serde(skip)]
  pub attacked: bool,
  #[serde(skip)]
  pub animation_fired: bool,
  #[serde(skip)]
  pub cooldown_timer: f32,
  #[serde(skip)]
  pub hit: f32,
  #[serde(skip)]
  pub heavy_timer: f32,
  #[serde(skip)]
  pub air_timer: f32,
}

impl AttackComponent {}
