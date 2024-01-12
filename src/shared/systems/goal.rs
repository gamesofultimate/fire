use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::goap::{Action, Goal, Planner, Blackboard};
use engine::{
  application::scene::{component_registry::Access, Scene, TransformComponent, UnpackEntity},
  systems::{Backpack, Initializable, Inventory, System},
  utils::units::{Radians, Time, Meters},
  Entity,
};

use nalgebra::{Point3, Vector3};
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
    match blackboard.get_bool("HeadingTowardsFire") {
      Some(true) => true,
      _ => false,
    }
    /*
    let fire_translation = match backpack.get::<&FireLocation>() {
      Some(FireLocation(Some(transform))) => transform.clone(),
      _ => return false,
    };

    let entity_transform = match scene.get_components::<&TransformComponent>(entity) {
      Some(transform) => transform.clone(),
      None => return false,
    };

    let distance = nalgebra::distance(
      &Point3::from(entity_transform.translation),
      &Point3::from(fire_translation),
    );

    distance > *self.max_distance
    */
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
  ) {
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
    (2.0 * self.distance) as i32
  }

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    log::debug!("test {:?}", blackboard.get_bool("KnowFireLocation"));
    match blackboard.get_bool("KnowFireLocation") {
      Some(true) => {
        log::debug!("found fire");
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
  ) {
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
  ) {
  }
}

pub struct GoalSystem {
  planner: (Planner, Backpack, Blackboard),
}

impl Initializable for GoalSystem {
  fn initialize(inventory: &Inventory) -> Self {
    /*
    let firewood = CollectFirewood {};
    let get_axe = GetAxe {};
    let chop_log = ChopLog {};
    let collect_branches = CollectBranches {};
    */

    let mut planner = Planner::new();
    planner.insert_goal(StayWarm::new());
    planner.insert_action(SearchForFire::new(Meters::new(100.0)));
    planner.insert_action(GoTowardsFire::new());
    planner.insert_action(Chill::new(Meters::new(2.0)));

    let mut backpack = Backpack::new();
    let mut blackboard = Blackboard::new();

    Self {
      planner: (planner, backpack, blackboard),
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
    let delta = backpack.get::<Time>().unwrap();

    let mut entities = vec![];

    for (entity, (_, transform)) in scene.query_mut::<(&GoalComponent, &TransformComponent)>() {
      entities.push((entity.clone(), transform.clone()));
    }

    for (entity, transform) in entities.drain(..) {
      self
        .planner
        .0
        .plan(entity, scene, &mut self.planner.1, &mut self.planner.2);
    }
    //log::info!("planner: {:?}", &self.planner.0);
  }
}
