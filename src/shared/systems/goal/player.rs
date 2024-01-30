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
use tagged::{Registerable, Schema, Duplicate};

#[derive(Debug)]
struct PlayerLocation(pub Vector3<f32>, Meters);

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable, Duplicate)]
pub struct SensePlayer {
  max_distance: Meters,
}

impl SensePlayer {
  pub fn new(max_distance: Meters) -> Self {
    Self {
      max_distance,
    }
  }
}

impl Sensor for SensePlayer {
  fn name(&self) -> &'static str {
    "SensePlayer"
  }

  fn sense(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    //log::debug!("Sensing Player");
    let entity_transform = match scene.get_components::<&TransformComponent>(entity) {
      Some(transform) => transform.clone(),
      None => return,
    };

    let mut player_distance = None;

    for (entity, (transform, _)) in scene.query_mut::<(
      &TransformComponent,
      &SelfComponent,
    )>() {
      let distance = nalgebra::distance(
        &Point3::from(entity_transform.translation),
        &Point3::from(transform.translation),
      );

      match player_distance {
        Some((_, current_distance)) if distance < current_distance => player_distance = Some((transform.translation, distance)),
        None => player_distance = Some((transform.translation, distance)),
        _ => {}
      }

      //log::info!("{:} player_location? {:?} -- {:?}", distance, &transform, player_distance);
    }

    match player_distance {
      Some((translation, distance)) if distance < *self.max_distance => {
        local.insert(PlayerLocation(translation, Meters::new(distance)));
      },
      _ => {
        local.take::<PlayerLocation>();
      },
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable, Duplicate)]
pub struct AggroCharacter {}

impl Goal for AggroCharacter {
  fn name() -> &'static str {
    "AggroCharacter"
  }

  fn get_goal(
    &self,
    entity: Entity,
    scene: &mut Scene,
    local: &mut Backpack,
  ) -> Blackboard {
    let mut blackboard = Blackboard::new();
    blackboard.insert_bool("NearbyPlayer", true);
    blackboard
  }
}

impl AggroCharacter {
  pub fn new() -> Self {
    Self {}
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable, Duplicate)]
pub struct Patrol {
}

impl Patrol {
  pub fn new() -> Self {
    Self {
    }
  }
}

impl Action for Patrol {
  fn name(&self) -> &'static str {
    "Patrol"
  }

  fn cost(
    &self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> i32 {
    1
  }

  fn check_readyness(
    &mut self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    //log::debug!("patrol");
    match local.get::<PlayerLocation>() {
      Some(location) => true,
      None => false,
    }
  }

  fn apply_effect(
    &mut self,
    local: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("KnowPlayerLocation", true);
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
    let (location, _) = if let Some(PlayerLocation(location, distance)) = local.get() {
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

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable, Duplicate)]
pub struct Attack {
  max_distance: Meters,
}

impl Attack {
  pub fn new(max_distance: Meters) -> Self {
    Self {
      max_distance,
    }
  }
}

impl Action for Attack {
  fn name(&self) -> &'static str {
    "Attack"
  }

  fn cost(
    &self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> i32 {
    1
  }

  fn check_readyness(
    &mut self,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    //log::debug!("attack");
    match (local.get::<PlayerLocation>(), blackboard.get_bool("KnowPlayerLocation")) {
      (Some(PlayerLocation(_, distance)), _) if *distance < self.max_distance => true,
      (_, Some(true)) => true,
      _ => false,
    }
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("NearbyPlayer", true);
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
