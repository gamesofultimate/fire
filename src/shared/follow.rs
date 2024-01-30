use std::cmp::Ordering;

use engine::application::components::{PhysicsComponent, SelfComponent};
use engine::application::scene::TransformComponent;
use engine::systems::physics::PhysicsController;
use engine::utils::units::{Meters, Mps, Rps, Seconds};
use engine::{
  application::{
    behavior::{behavior_registry::Access, Behavior, BehaviorNode, Status},
    scene::Scene,
  },
  systems::{Backpack, Initializable, Inventory, Middleware},
  Entity,
};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use tagged::{Registerable, Schema, Duplicate};

#[derive(Debug, Clone, Serialize, Deserialize, Registerable, Schema, Duplicate)]
pub struct Follow {
  pub children: Vec<BehaviorNode>,
  pub speed: Mps,
  pub rotation_speed: Rps,
  pub detection_radius: Meters,
  pub attack_range: Meters,
  pub timer: Seconds,
  pub attack_cooldown: Seconds,
  pub pacing_direction: Option<Vector3<f32>>,
  pub pacing_time_remaining: Seconds,
  pub pacing_speed: Mps,
}

impl Behavior for Follow {
  fn run(
    &mut self,
    entity: Entity,
    scene: &mut Scene,
    backpack: &mut Backpack,
    _: &mut Backpack,
  ) -> Status {
    log::debug!("LET'S FOLLOW");

    // let controller = backpack.get_mut::<PhysicsController>().unwrap();
    // let mut targets: Vec<Vector3<f32>> = vec![];
    // for (_, (_, transform)) in scene.query_mut::<(&SelfComponent, &TransformComponent)>() {
    //     targets.push(transform.translation.clone())
    // }

    // let (transform, physics) = match scene.get_components::<(&mut TransformComponent, &PhysicsComponent)>(entity) {
    //     Some((transform, physics)) => (transform, physics),
    //     None => return Status::Failure,
    // };

    // let mut chase_executed = false;

    // if let Some((closest_target, closest_distance)) = targets
    //     .iter()
    //     .map(|&target| (target, (transform.translation - target).magnitude()))
    //     .filter(|&(_, distance)| distance <= *self.detection_radius)
    //     .min_by(|&(_, dist_a), &(_, dist_b)| dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal))
    // {
    //     let direction = (closest_target - transform.translation).normalize();
    //     controller.rotate_and_move_towards(
    //         physics,
    //         transform.rotation,
    //         direction,
    //         closest_target,
    //         direction,
    //         self.speed,
    //         self.rotation_speed,
    //     );
    //     chase_executed = true;
    // }

    // // Process child behaviors regardless of whether chase was executed
    // for child in &mut self.children {
    //     let status = child.run(entity, scene, backpack);
    //     if status != Status::Success {
    //         return status;
    //     }
    // }

    // if chase_executed { Status::Success } else { Status::Failure }
    Status::Success
  }
}

pub struct MayhemBehaviors;

impl Initializable for MayhemBehaviors {
  fn initialize(_: &Inventory) -> Self {
    Self
  }
}

impl Middleware for MayhemBehaviors {
  fn provide(&mut self, _: &Inventory) {
    Follow::register();
  }
}
