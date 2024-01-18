use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, hash_map::Entry};

use engine::{
  application::scene::{Scene, TransformComponent, UnpackEntity},
  systems::{Backpack, Initializable, Inventory, System},
  utils::units::{Radians, Time, Meters},
  Entity,
};
use crate::shared::components::movement_component::MovementComponent;
use engine::application::components::{PhysicsComponent, SelfComponent};
use engine::systems::physics::PhysicsController;

use nalgebra::{Point3, Vector3, UnitQuaternion, Unit};
use tagged::{Registerable, Schema};


#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct FireComponent {
  id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct TreeComponent {
  id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct FirewoodComponent {
  id: Uuid,
}

