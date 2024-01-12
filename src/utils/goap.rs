use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use priority_queue::PriorityQueue;
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
    backpack: &Backpack,
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


#[derive(Debug)]
pub struct Planner {
  actions: Vec<Box<dyn Action>>,
  goals: Vec<Box<dyn Goal>>,
}

impl Planner {
  pub fn new() -> Self {
    Self {
      actions: vec![],
      goals: vec![],
    }
  }

  pub fn insert_action(&mut self, action: impl Action + 'static) {
    self.actions.push(Box::new(action));
  }

  pub fn insert_goal(&mut self, goal: impl Goal + 'static) {
    self.goals.push(Box::new(goal));
  }

  pub fn plan(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
    //let mut goals = vec![];
    //let mut plan = vec![];
    let mut plan = PriorityQueue::new();

    'goal_loop: for goal in &self.goals {
      let goal_blackboard = goal.get_goal(entity, scene, backpack);

      let mut open_set = PriorityQueue::new();
      let mut closed_set = HashSet::new();
      let mut parents = HashMap::new();

      open_set.push(PlanningNode {
        name: "root",
        blackboard: blackboard.clone(),
        action: None,
        parent: None,
      }, 0);

      let mut iterations = 0;

      while let Some((current_node, cost)) = open_set.pop() {
        if MAX_ITERATIONS == 0 || iterations > MAX_ITERATIONS { continue 'goal_loop; }

        // NOTE: Order matters here. goal_blackboard must come first
        if goal_blackboard == current_node.blackboard {
          let mut curr = current_node.action;

          while let Some(node_index) = curr {
            curr = parents[&node_index];
            plan.push(node_index, cost);
          }

          continue 'goal_loop;
        }

        if !closed_set.contains(&current_node.blackboard) {
          /// NOTE: I think we can move this to the end of the for loop
          /// and avoid the clone that way
          closed_set.insert(current_node.blackboard.clone());

          for (index, action) in self.actions.iter_mut().enumerate() {
            if action.check_readyness(entity, scene, backpack, &current_node.blackboard) {
              let mut next_blackboard = current_node.blackboard.clone();
              let next_cost = cost + action.cost(&next_blackboard);
              action.apply_effect(backpack, &mut next_blackboard);

              if !closed_set.contains(&next_blackboard) {

                parents.insert(index, current_node.action);
                open_set.push(PlanningNode {
                  name: action.name(),
                  blackboard: next_blackboard,
                  action: Some(index),
                  parent: current_node.action,
                }, next_cost);
              }
            }
          }
        }
        iterations += 1;
      }
    }

    if let Some((action_index, _)) = plan.pop() {
      let mut action = &mut self.actions[action_index];
      action.execute(entity, scene, backpack);
    }
  }

  pub fn execute(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    blackboard: &mut Blackboard,
  ) {
  }
}

/*
/// A node in the planner graph.
#[derive(PartialEq, Eq, Clone)]
struct PlanNode<'a> {
  current_state: State,
  action: Option<&'a Action>,
}

impl<'a> Hash for PlanNode<'a> {
  fn hash<H>(&self, state: &mut H)
  where
    H: Hasher,
  {
    if let Some(action) = self.action {
      action.name.hash(state);
    }

    for (key, value) in &self.current_state {
      key.hash(state);
      value.hash(state);
    }
  }
}

impl<'a> PlanNode<'a> {
  /// Makes an initial plan node without a parent.
  fn initial(initial_state: &'a State) -> PlanNode<'a> {
    PlanNode {
      current_state: initial_state.clone(),
      action: None,
    }
  }

  /// Makes a plan node from a parent state and an action applied to that state.
  fn child(parent_state: State, action: &'a Action) -> PlanNode<'a> {
    let mut child = PlanNode {
      current_state: parent_state.clone(),
      action: Some(action),
    };

    // Applies the post-condition of the action applied on our parent state.
    for (name, value) in &action.post_conditions {
      child.current_state.insert(name.clone(), value.clone());
    }

    child
  }

  /// Returns all possible nodes from this current state, along with the cost to get there.
  fn possible_next_nodes(&self, actions: &'a [Action]) -> Vec<(PlanNode<'a>, usize)> {
    let mut nodes: Vec<(PlanNode<'a>, usize)> = vec![];
    for action in actions {
      if self.matches(&action.pre_conditions) {
        nodes.push((
          PlanNode::child(self.current_state.clone(), action),
          action.cost,
        ));
      }
    }

    nodes
  }

  /// Count the number of states in this node that aren't matching the given target.
  fn mismatch_count(&self, target: &State) -> usize {
    let mut count: usize = 0;
    for (name, target_value) in target {
      if let Some(current_value) = self.current_state.get(name) {
        if current_value != target_value {
          count += 1;
        }
      } else {
        count += 1;
      }
    }

    count
  }

  /// Returns `true` if the current node is a full match for the given target.
  fn matches(&self, target: &State) -> bool {
    self.mismatch_count(target) == 0
  }
}
*/

/*
/// Formulates a plan to get from an initial state to a goal state using a set of allowed actions.
pub fn plan<'a>(initial_state: &'a State,
                goal_state: &State,
                allowed_actions: &'a [Action])
                -> Option<Vec<&'a Action>> {
    // Builds our initial plan node.
    let start = PlanNode::initial(initial_state);

    // Runs our search over the states graph.
    if let Some((plan, _)) = astar(&start,
                                   |ref node| node.possible_next_nodes(allowed_actions),
                                   |ref node| node.mismatch_count(goal_state),
                                   |ref node| node.matches(goal_state)) {
        Some(plan.into_iter().skip(1).map(|ref node| node.action.unwrap()).collect())
    } else {
        None
    }
}
*/
