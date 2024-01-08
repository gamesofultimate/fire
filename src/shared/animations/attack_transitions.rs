use crate::shared::components::attack_component::AttackComponent;
use crate::shared::components::attack_component::AttackType;
use crate::shared::components::attack_component::AttackTypeDamage;

use engine::{
  application::{
    animation::{animation_transition_registry::Access, AnimationTransition},
    scene::Scene,
  },
  systems::{Backpack, Initializable, Inventory, Middleware},
  Entity,
};

use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema};

pub struct AttackTransitions;

impl Initializable for AttackTransitions {
  fn initialize(_: &Inventory) -> Self {
    Self
  }
}

impl Middleware for AttackTransitions {
  fn provide(&mut self, _: &Inventory) {
    AttackTypeTransition::register();
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema)]
pub struct AttackTypeTransition {
  pub attack_type: AttackType,
}

impl AnimationTransition for AttackTypeTransition {
  fn should_transition(&self, entity: Entity, scene: &mut Scene, _: &Backpack) -> bool {
    if let Some(attack) = scene.get_components::<&AttackComponent>(entity) {
      if attack.attack_type_damage.attack_type == self.attack_type {
        true
      } else {
        false
      }
    } else {
      true
    }
  }
}
