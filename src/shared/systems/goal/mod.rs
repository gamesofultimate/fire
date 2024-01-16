use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, hash_map::Entry};

use crate::utils::goap::{Sensor, Action, Goal, Planner, Blackboard};
use engine::{
  application::scene::{component_registry::Access, Scene, TransformComponent, UnpackEntity},
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
pub struct GoalComponent {
  id: Uuid,
}

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

#[derive(Debug)]
struct FireLocation(pub Vector3<f32>, Meters);

#[derive(Debug)]
struct PlayerLocation(pub Vector3<f32>, Meters);

#[derive(Debug)]
struct SensePlayer {
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

#[derive(Debug)]
struct SenseFire {
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

#[derive(Debug)]
struct AggroCharacter {}
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

#[derive(Debug)]
struct Patrol {
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
    blackboard: &Blackboard,
  ) -> i32 {
    1
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
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

#[derive(Debug)]
struct Attack {
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
    blackboard: &Blackboard,
  ) -> i32 {
    1
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
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


#[derive(Debug)]
struct StayWarm {}
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


#[derive(Debug)]
struct SearchForFire {
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
    blackboard: &Blackboard,
  ) -> i32 {
    3
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("LocatedFire", true);
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    match backpack.get::<FireLocation>() {
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

#[derive(Debug)]
struct Chill {
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
    blackboard: &Blackboard,
  ) -> i32 {
    3
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    match (backpack.get::<FireLocation>(), blackboard.get_bool("LocatedFire")) {
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
pub struct GoalSystem {
  planners: HashMap<(Entity, Uuid), (Planner, Backpack)>,
}

impl Initializable for GoalSystem {
  fn initialize(inventory: &Inventory) -> Self {
    Self {
      planners: HashMap::new(),
    }
  }
}

impl GoalSystem {}

impl System for GoalSystem {
  fn provide(&mut self, inventory: &Inventory) {
    GoalComponent::register();
    FireComponent::register();
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    #[cfg(target_arch = "wasm32")]
    {
      let delta = backpack.get::<Time>().unwrap();

      let mut planners = vec![];

      for (entity, goal) in scene.query_mut::<&GoalComponent>() {
        planners.push((entity.clone(), goal.id))
      }

      for (entity, goal_id) in planners.drain(..) {
        match self.planners.entry((entity, goal_id)) {
          Entry::Occupied(mut entry) => {
            let planner = entry.into_mut();
            planner.0.plan(entity, scene, backpack, &mut planner.1);
          }
          Entry::Vacant(vacant) => {
            let mut planner = Planner::new();
            planner.insert_goal(StayWarm::new());
            planner.insert_action(SearchForFire::new());
            planner.insert_action(Chill::new(Meters::new(2.0)));

            planner.insert_goal(AggroCharacter::new());
            planner.insert_action(Patrol::new());
            planner.insert_action(Attack::new(Meters::new(5.0)));

            planner.insert_sensor(SenseFire::new(Meters::new(100.0)));
            planner.insert_sensor(SensePlayer::new(Meters::new(10.0)));

            let mut local = Backpack::new();

            planner.plan(entity, scene, backpack, &mut local);
            vacant.insert((planner, local));
          },
        };
      }
      }
  }
}
