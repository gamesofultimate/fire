use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, hash_map::Entry};

use engine::{
  application::{
    scene::{Scene, TransformComponent, UnpackEntity},
    goap::{Sensor, Action, Goal, Planner, Blackboard},
  },
  systems::{Backpack, Initializable, Inventory, System},
  utils::units::{Radians, Time, Meters},
  Entity,
};
use crate::shared::components::movement_component::MovementComponent;
use engine::application::components::{PhysicsComponent, SelfComponent};
use engine::systems::physics::PhysicsController;

use nalgebra::{Point3, Vector3, UnitQuaternion, Unit};
use tagged::{Registerable, Schema};

use super::components::FireComponent;

#[derive(Debug)]
struct FireLocation(pub Vector3<f32>, Meters);

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct SenseFire {
  max_distance: Meters,
}

impl SenseFire {
  pub fn new(max_distance: Meters) -> Self {
    Self {
      max_distance,
    }
  }
}

impl Sensor for SenseFire {
  fn name(&self) -> &'static str {
    "SenseFire"
  }

  fn sense(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    let entity_transform = match scene.get_components::<&TransformComponent>(entity) {
      Some(transform) => transform.clone(),
      None => return,
    };

    let mut fire_distance = None;

    for (entity, (transform, _)) in scene.query_mut::<(
      &TransformComponent,
      &FireComponent,
    )>() {
      let distance = nalgebra::distance(
        &Point3::from(entity_transform.translation),
        &Point3::from(transform.translation),
      );

      match fire_distance {
        Some((_, current_distance)) if distance < current_distance => fire_distance = Some((transform.translation, distance)), 
        None => fire_distance = Some((transform.translation, distance)),
        _ => {}
      }
    }

    match fire_distance {
      Some((translation, distance)) if distance < *self.max_distance => {
        local.insert(FireLocation(translation, Meters::new(distance)));
      },
      _ => {
        local.take::<FireLocation>();
      },
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct StayWarm {}
impl Goal for StayWarm {
  fn name() -> &'static str {
    "StayWarm"
  }

  fn get_goal(
    &self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
  ) -> Blackboard {
    let mut blackboard = Blackboard::new();
    blackboard.insert_bool("NearbyFire", true);
    blackboard
  }
}

impl StayWarm {
  pub fn new() -> Self {
    Self {}
  }
}


#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct SearchForFire {
}

impl SearchForFire {
  pub fn new() -> Self {
    Self {
    }
  }
}

impl Action for SearchForFire {
  fn name(&self) -> &'static str {
    "SearchForFire"
  }

  fn cost(
    &self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> i32 {
    3
  }

  fn apply_effect(
    &mut self,
    local: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("LocatedFire", true);
  }

  fn check_readyness(
    &mut self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    match local.get::<FireLocation>() {
      Some(location) => true,
      None => false,
    }
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
    let (location, distance) = if let Some(FireLocation(location, distance)) = local.get() {
      (location.clone(), distance)
    } else {
      return
    };

    if let Some(physics_controller) = backpack.get_mut::<PhysicsController>()
      && let Some((transform, physics, movement)) = scene.get_components::<(
        &TransformComponent,
        &PhysicsComponent,
        &MovementComponent,
      )>(entity) {
      let rotation_quaternion = UnitQuaternion::from_euler_angles(
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
      ) * Vector3::new(0.0, 0.0, 1.0);

      let direction_vector = Unit::new_normalize(transform.rotation - location);

      physics_controller.move_towards(
        &physics,
        transform.translation,
        location,
        movement.run_speed,
      );
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct Chill {
  max_distance: Meters,
}

impl Chill {
  pub fn new(max_distance: Meters) -> Self {
    Self {
      max_distance,
    }
  }
}

impl Action for Chill {
  fn name(&self) -> &'static str {
    "Chill"
  }

  fn cost(
    &self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> i32 {
    3
  }

  fn check_readyness(
    &mut self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    match (local.get::<FireLocation>(), blackboard.get_bool("LocatedFire")) {
      (Some(FireLocation(_, distance)), _) if *distance < self.max_distance => true,
      (_, Some(true)) => true,
      _ => false,
    }
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("NearbyFire", true);
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
    if let Some(physics_controller) = backpack.get_mut::<PhysicsController>()
      && let Some(physics) = scene.get_components::<&PhysicsComponent>(entity) {
      physics_controller.set_linvel(&physics, Vector3::zeros());
    }
  }
}
