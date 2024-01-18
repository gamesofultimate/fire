mod components;
mod player;
mod fire;

use engine::{
  systems::{Backpack, Initializable, Inventory, System, Registry},
};

use nalgebra::{Point3, Vector3, UnitQuaternion, Unit};

use crate::shared::systems::goal::{
  components::{FireComponent, TreeComponent, FirewoodComponent},
  fire::{SenseFire, StayWarm, SearchForFire, Chill},
  player::{SensePlayer, AggroCharacter, Patrol, Attack},
};

pub struct GoalRegistry {
}

impl Registry for GoalRegistry {
  fn register() {
    {
      use engine::application::scene::component_registry::Access;
      TreeComponent::register();
      FireComponent::register();
      FirewoodComponent::register();
    }

    {
      use engine::application::goap::goal_registry::Access;
      StayWarm::register();
      AggroCharacter::register();
    }

    {
      use engine::application::goap::action_registry::Access;
      SearchForFire::register();
      Chill::register();
      Patrol::register();
      Attack::register();
    }

    {
      use engine::application::goap::sensor_registry::Access;
      SenseFire::register();
      SensePlayer::register();
    }
  }
}

/*
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
*/
