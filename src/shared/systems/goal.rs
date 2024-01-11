use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::goap::{Action, Goal, Planner, Blackboard};
use engine::{
  application::scene::{component_registry::Access, Scene, TransformComponent, UnpackEntity},
  systems::{Backpack, Initializable, Inventory, System},
  utils::units::{Radians, Time},
  Entity,
};

use nalgebra::Vector3;
use tagged::{Registerable, Schema};

#[derive(Debug, Clone, Serialize, Deserialize, Schema, Registerable)]
pub struct GoalComponent {
  id: Uuid,
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
    &self,
    entity: Entity,
    scene: &Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let is_axe_available = blackboard.get_bool("AxeLocation").cloned().unwrap_or(false);
    let have_axe = blackboard.get_bool("AxeState").cloned().unwrap_or(false);
    is_axe_available && !have_axe
  }

  fn apply_effect(
    &mut self,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("AxeState", true);
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
    &self,
    entity: Entity,
    scene: &Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let have_axe = blackboard.get_bool("AxeState").cloned().unwrap_or(false);
    have_axe
  }

  fn apply_effect(
    &mut self,
    blackboard: &mut Blackboard,
  ) {
    blackboard.insert_bool("AxeState", true);
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
    &self,
    entity: Entity,
    scene: &Scene,
    backpack: &Backpack,
    blackboard: &Blackboard,
  ) -> bool {
    let branches_in_viscinity = blackboard.get_number("BranchesInViscinity").cloned().unwrap_or(0);
    branches_in_viscinity > 0
  }

  fn apply_effect(
    &mut self,
    blackboard: &mut Blackboard,
  ) {
    let ownership = blackboard.get_number("FirewoodOwnership").cloned().unwrap_or(0);
    blackboard.insert_number("FirewoodOwnership", ownership + 1);
  }
}

pub struct GoalSystem {
  planner: (Planner, Blackboard),
}

impl Initializable for GoalSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let firewood = CollectFirewood {};
    let get_axe = GetAxe {};
    let chop_log = ChopLog {};
    let collect_branches = CollectBranches {};

    let mut planner = Planner::new();
    planner.insert_goal(firewood);
    planner.insert_action(chop_log);
    planner.insert_action(get_axe);
    planner.insert_action(collect_branches);

    let mut blackboard = Blackboard::new();

    Self {
      planner: (planner, blackboard),
    }
  }
}

impl GoalSystem {}

impl System for GoalSystem {
  fn provide(&mut self, inventory: &Inventory) {
    GoalComponent::register();
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
        .plan(entity, scene, backpack, &mut self.planner.1);
    }
    //log::info!("planner: {:?}", &self.planner.0);
  }
}
