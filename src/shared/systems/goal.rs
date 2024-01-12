use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::{HashMap, hash_map::Entry};

use crate::utils::goap::{Action, Goal, Planner, Blackboard};
use engine::{
  application::scene::{component_registry::Access, Scene, TransformComponent, UnpackEntity},
  systems::{Backpack, Initializable, Inventory, System},
  utils::units::{Radians, Time, Meters},
  Entity,
};
use crate::shared::components::movement_component::MovementComponent;
use engine::application::components::PhysicsComponent;
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
struct FireLocation(pub Option<Vector3<f32>>);

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

/// NOTE: Action
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
    2
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let entity_transform = match scene.get_components::<&TransformComponent>(entity) {
      Some(transform) => transform.clone(),
      None => return false,
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
        Some(current_distance) if distance < current_distance => fire_distance = Some(distance), 
        None => fire_distance = Some(distance),
        _ => {}
      }

      //log::info!("{:} fire_location? {:?} -- {:?}", distance, &transform, fire_distance);
    }

    match (fire_distance, blackboard.get_bool("HeadingTowardsFire")) {
      (Some(distance), _) if distance < *self.max_distance => true,
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

/// NOTE: Action
#[derive(Debug)]
struct GoTowardsFire {
  distance: f32,
}

impl GoTowardsFire {
  pub fn new() -> Self {
    Self {
      distance: 0.0,
    }
  }
}

impl Action for GoTowardsFire {
  fn name(&self) -> &'static str {
    "GoTowardsFire"
  }

  fn cost(
    &self,
    blackboard: &Blackboard,
  ) -> i32 {
    2
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    match blackboard.get_bool("KnowFireLocation") {
      Some(true) => {
        //log::debug!("found fire");
        /*
        let entity_transform = match scene.get_components::<&TransformComponent>(entity) {
          Some(transform) => transform,
          None => return false,
        };

        let fire_translation = match backpack.get::<&FireLocation>() {
          Some(FireLocation(Some(transform))) => transform.clone(),
          _ => return false,
        };

        self.distance = nalgebra::distance(
          &Point3::from(entity_transform.translation),
          &Point3::from(fire_translation),
        );
        */

        true
      },
      _ => false,
    }
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("HeadingTowardsFire", true);
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
  }
}

/// NOTE: Sensor
#[derive(Debug)]
struct SearchForFire {
  max_distance: Meters,
  fire_location: Option<Vector3<f32>>,
}

impl SearchForFire {
  pub fn new(max_distance: Meters) -> Self {
    Self {
      max_distance,
      fire_location: None,
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

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let entity_transform = match scene.get_components::<&TransformComponent>(entity) {
      Some(transform) => transform.clone(),
      None => return false,
    };

    let mut fire_location = None;

    for (entity, (transform, _)) in scene.query_mut::<(
      &TransformComponent,
      &FireComponent,
    )>() {
      let distance = nalgebra::distance(
        &Point3::from(entity_transform.translation),
        &Point3::from(transform.translation),
      );

      if distance > *self.max_distance { continue }

      //log::info!("{:} fire_location? {:?}", distance, &transform);

      match (fire_location, distance) {
        (Some(_), fire_distance) if distance < fire_distance => {
          fire_location = Some((transform.translation, distance));
        },
        (None, _) => {
          fire_location = Some((transform.translation, distance));
        },
        _ => { },
      }
    }

    //log::info!("fire_location? {:?}", &fire_location);


    match fire_location {
      Some((transform, _)) => {
        self.fire_location = Some(transform);
        true
      },
      None => false,
    }
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    if let Some(_) = self.fire_location {
      backpack.insert(FireLocation(self.fire_location));
      blackboard.insert_bool("KnowFireLocation", true);
    }
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
    if let Some(physics_controller) = backpack.get_mut::<PhysicsController>()
      && let Some(fire_location) = self.fire_location
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

      let direction_vector = Unit::new_normalize(transform.rotation - fire_location);

      physics_controller.move_towards(
        &physics,
        transform.translation,
        fire_location,
        movement.run_speed,
      );
    }
  }
}

#[derive(Debug)]
struct CollectFirewood {}
impl Goal for CollectFirewood {
  fn name() -> &'static str {
    "CollectFirewood"
  }

  fn get_goal(
    &self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
  ) -> Blackboard {
    let mut blackboard = Blackboard::new();
    blackboard.insert_number("FirewoodOwnership", 8);
    blackboard
  }
}

#[derive(Debug)]
struct GetAxe {}
impl Action for GetAxe {
  fn name(&self) -> &'static str {
    "GetAxe"
  }

  fn cost(
    &self,
    blackboard: &Blackboard,
  ) -> i32 {
    2
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let is_axe_available = blackboard.get_bool("AxeLocation").cloned().unwrap_or(false);
    let have_axe = blackboard.get_bool("AxeState").cloned().unwrap_or(false);
    is_axe_available && !have_axe
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("AxeState", true);
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
  }
}

#[derive(Debug)]
struct ChopLog {}
impl Action for ChopLog {
  fn name(&self) -> &'static str {
    "ChopLog"
  }

  fn cost(
    &self,
    blackboard: &Blackboard,
  ) -> i32 {
    4
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let have_axe = blackboard.get_bool("AxeState").cloned().unwrap_or(false);
    have_axe
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("AxeState", true);
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
  }
}

#[derive(Debug)]
struct CollectBranches {}
impl Action for CollectBranches {
  fn name(&self) -> &'static str {
    "CollectBranches"
  }

  fn cost(
    &self,
    blackboard: &Blackboard,
  ) -> i32 {
    8
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let branches_in_viscinity = blackboard.get_number("BranchesInViscinity").cloned().unwrap_or(0);
    branches_in_viscinity > 0
  }

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    let ownership = blackboard.get_number("FirewoodOwnership").cloned().unwrap_or(0);
    blackboard.insert_number("FirewoodOwnership", ownership + 1);
  }

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
  }
}

pub struct GoalSystem {
  planners: HashMap<(Entity, Uuid), (Planner, Backpack, Blackboard)>,
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
            let mut blackboard = Blackboard::new();
            planner.0.plan(entity, scene, backpack, &mut planner.1, &mut blackboard);
          }
          Entry::Vacant(vacant) => {
            let mut planner = Planner::new();
            planner.insert_goal(StayWarm::new());
            planner.insert_action(SearchForFire::new(Meters::new(100.0)));
            planner.insert_action(GoTowardsFire::new());
            planner.insert_action(Chill::new(Meters::new(2.0)));

            let mut local = Backpack::new();
            let mut blackboard = Blackboard::new();

            planner.plan(entity, scene, backpack, &mut local, &mut blackboard);
            vacant.insert((planner, local, blackboard));
          },
        };
      }
      }
  }
}
