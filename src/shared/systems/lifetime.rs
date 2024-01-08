use crate::shared::components::lifetime_component::LifetimeComponent;
use engine::{
  application::{
    gamefile::current,
    scene::{component_registry::Access, Scene},
  },
  systems::{Backpack, Initializable, Inventory, System},
  utils::units::{Radians, Seconds, Time},
};
pub struct LifetimeSystem {}

impl Initializable for LifetimeSystem {
  fn initialize(_: &Inventory) -> Self {
    Self {}
  }
}

impl System for LifetimeSystem {
  fn provide(&mut self, inventory: &Inventory) {
    LifetimeComponent::register();
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let dt = **backpack.get::<Time>().unwrap();

    let mut entities_to_remove = vec![];
    for (current_entity, (lifetime)) in scene.query_mut::<(&mut LifetimeComponent)>() {
      if lifetime.is_running == true {
        lifetime.timer += Seconds::new(dt);
        if lifetime.timer > lifetime.duration {
          entities_to_remove.push(current_entity);
        }
      } else {
        lifetime.is_running = true;
      }
    }
    for ent in entities_to_remove {
      scene.remove_entity(ent);
    }
  }
}
