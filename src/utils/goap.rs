use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use priority_queue::{DoublePriorityQueue, PriorityQueue};
use std::cmp::PartialEq;

use engine::{application::scene::Scene, systems::Backpack, Entity};

const MAX_ITERATIONS:usize = 50;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Value {
  Bool(bool),
  Number(u32),
  Str(String),
}

#[derive(Debug, Eq, Clone)]
pub struct Blackboard {
  map: HashMap<String, Value>,
}

impl PartialEq for Blackboard {
  fn eq(&self, other: &Self) -> bool {
    for (key, value) in &self.map {
      if let Some(other_value) = other.map.get(&*key) {
        if value != other_value {
          return false
        }
      } else {
        return false
      }
    }

    return true
  }
}

impl Hash for Blackboard {
  fn hash<H: Hasher>(&self, state: &mut H) {
    // Iterate over key-value pairs and include them in the hash calculation
    for (key, value) in &self.map {
      key.hash(state);
      value.hash(state);
    }
  }
}

impl Blackboard {
  pub fn new() -> Self {
    Self {
      map: HashMap::new(),
    }
  }

  pub fn insert_bool(&mut self, key: &str, data: bool) {
    self.map.insert(String::from(key), Value::Bool(data));
  }

  pub fn insert_number(&mut self, key: &str, data: u32) {
    self.map.insert(String::from(key), Value::Number(data));
  }

  pub fn insert_str(&mut self, key: &str, data: &str) {
    self.map.insert(String::from(key), Value::Str(String::from(data)));
  }

  pub fn get_bool(&self, key: &str) -> Option<&bool> {
    match self.map.get(key) {
      Some(Value::Bool(v)) => Some(v),
      _ => None,
    }
  }

  pub fn get_number(&self, key: &str) -> Option<&u32> {
    match self.map.get(key) {
      Some(Value::Number(v)) => Some(v),
      _ => None,
    }
  }

  pub fn get_str(&self, key: &str) -> Option<&String> {
    match self.map.get(key) {
      Some(Value::Str(v)) => Some(v),
      _ => None,
    }
  }

  pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
    self.map.get_mut(key)
  }

  pub fn take(&mut self, key: &str) -> Option<Value> {
    self.map.remove(key)
  }
}

/// Sense the world, and creates an initial set of effects to put
/// inside of the blackboard. This helps make the creation of actions
/// that are a bit easier to maintain.
/// They always run, and they must modify the blackboard, if their sensor
/// is triggered.
/// They must not modify the world
pub trait Sensor: Debug + Sync + Send {
  fn name(&self) -> &'static str;

  fn sense(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
    blackboard: &mut Blackboard,
  );
}

pub trait Action: Debug + Sync + Send {
  fn name(&self) -> &'static str;

  fn cost(
    &self,
    blackboard: &Blackboard,
  ) -> i32;

  fn check_readyness(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    local: &Backpack,
    blackboard: &Blackboard,
  ) -> bool;

  fn apply_effect(
    &mut self,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  );

  fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  );
}

impl Eq for dyn Action { }

impl PartialEq for dyn Action {
  fn eq(&self, other: &Self) -> bool {
    self.name() == other.name()
  }
}

impl Hash for dyn Action {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.name().hash(state);
  }
}

pub trait Goal: Debug + Sync + Send {
  fn name() -> &'static str where Self:Sized;

  fn get_goal(
    &self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
  ) -> Blackboard;
}

#[derive(Debug)]
struct PlanningNode {
  name: &'static str,
  blackboard: Blackboard,
  cost: i32,
  action: Option<usize>,
  parent: Option<usize>,
}

impl Hash for PlanningNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.blackboard.hash(state);
    self.name.hash(state);
  }
}

impl Eq for PlanningNode {
}

impl PartialEq for PlanningNode {
  fn eq(&self, other: &Self) -> bool {
    self.blackboard == other.blackboard && self.name == other.name
  }
}

fn put<T>(v: &mut Vec<T>, item: T) -> usize {
  let idx = v.len();
  v.push(item);
  idx
}


#[derive(Debug)]
pub struct Planner {
  actions: Vec<Box<dyn Action>>,
  goals: Vec<Box<dyn Goal>>,
  sensors: Vec<Box<dyn Sensor>>,
}

impl Planner {
  pub fn new() -> Self {
    Self {
      actions: vec![],
      goals: vec![],
      sensors: vec![],
    }
  }

  pub fn insert_action(&mut self, action: impl Action + 'static) {
    self.actions.push(Box::new(action));
  }

  pub fn insert_goal(&mut self, goal: impl Goal + 'static) {
    self.goals.push(Box::new(goal));
  }

  pub fn insert_sensor(&mut self, sensor: impl Sensor + 'static) {
    self.sensors.push(Box::new(sensor));
  }

  pub fn plan(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    local: &mut Backpack,
  ) {
    //let mut goals = vec![];
    //let mut plan = vec![];
    let mut plan = PriorityQueue::new();

    let mut blackboard = Blackboard::new();

    for sensor in &mut self.sensors {
      sensor.sense(entity, scene, backpack, local, &mut blackboard);
    }

    'goal_loop: for goal in &self.goals {
      let goal_blackboard = goal.get_goal(entity, scene, local);

      let mut blackboard = blackboard.clone();
      let mut open_set = PriorityQueue::new();
      let mut closed_set = HashSet::new();
      let mut parents = vec![];

      let root = PlanningNode {
        name: "root",
        blackboard: blackboard.clone(),
        cost: 0,
        action: None,
        parent: None,
      };
      let root_index = put(&mut parents, root);
      open_set.push(root_index, 0);

      let mut iterations = 0;

      while let Some((current_index, cost)) = open_set.pop() {
        //let current_node = &parents[current_index];
        if MAX_ITERATIONS == 0 || iterations > MAX_ITERATIONS { continue 'goal_loop; }

        //log::debug!("{:?}: blackboard: {:?} -> goal: {:?}", &parents[current_index].action, &parents[current_index].blackboard, &goal_blackboard);
        // NOTE: Order matters here. goal_blackboard must come first
        if goal_blackboard == parents[current_index].blackboard {
          let mut curr = current_index;
          let mut inner_plan = vec![];

          loop {
            //while let Some(node_index) = curr {
            if let PlanningNode { action: Some(node_index), parent: Some(parent), cost, .. } = parents[curr] {
              inner_plan.push((node_index, cost));
              curr = parent;
            } else {
              if let Some((index, cost)) = inner_plan.pop() {
                plan.push(index, cost);
              }
              continue 'goal_loop;
            }
          }
        }

        if !closed_set.contains(&parents[current_index].blackboard) {
          /// NOTE: I think we can move this to the end of the for loop
          /// and avoid the clone that way
          closed_set.insert(parents[current_index].blackboard.clone());

          for (index, action) in self.actions.iter_mut().enumerate() {
            if action.check_readyness(entity, scene, local, &parents[current_index].blackboard) {
              let mut next_blackboard = parents[current_index].blackboard.clone();
              let next_cost = cost - action.cost(&next_blackboard);
              action.apply_effect(local, &mut next_blackboard);

              //log::debug!("{:}: blackboard: {:?}", &action.name(), &next_blackboard);

              if !closed_set.contains(&next_blackboard) {
                let node = PlanningNode {
                  name: action.name(),
                  blackboard: next_blackboard,
                  cost: next_cost,
                  action: Some(index),
                  parent: Some(current_index),
                };
                let tree_index = put(&mut parents, node);
                open_set.push(tree_index, next_cost);
              }
            }
          }
        }
        iterations += 1;
      }
    }

    #[cfg(feature = "debug-goap")]
    {
      let mut names = vec![];

      for ((index, priority)) in &plan {
        let mut action = &mut self.actions[*index];
        names.push((priority, action.name()));
      }

      log::debug!("plan: {:?}", &names);
    }

    if let Some((action_index, _)) = plan.pop() {
      let mut action = &mut self.actions[action_index];
      //log::info!("executing {:}", &action.name());
      action.execute(entity, scene, backpack, local);
    }
  }
}
