#![cfg(target_arch = "wasm32")]
/*
This file contains code that controls various item drops.
Note: This is fairly clunky, we really need to get a better
API for handling creating new entities from prefabs. That
include generating new IDs for anything that needs it.
*/

use crate::shared::{
  components::{
    magnetic_pickup_component::MagneticPickupComponent, movement_component::MovementComponent, resource_component::ResourceComponent, inventory_component::InventoryComponent,
  },
  game_types::game_types::EnemyState,
  input::PlayerInput,
};
use engine::application::components::{
  AnimationComponent, InputComponent, PhysicsComponent, SelfComponent,
};
use engine::application::scene::component_registry::Access;
use engine::application::scene::TagComponent;
use engine::application::{
  input::DefaultInput,
  scene::{IdComponent, Scene, TransformComponent, UnpackEntity},
};
use engine::systems::{
  input::{CanvasController, InputsReader},
  physics::{PhysicsConfig, PhysicsController},
  Backpack, Initializable, Inventory, System,
};
use engine::utils::units::Kph;
use engine::utils::units::Time;
use engine::Entity;
use nalgebra::Rotation3;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;
use std::char::MAX;
use uuid::Uuid;

pub struct ItemDropSystem {
  inputs: InputsReader<PlayerInput>,
  physics: PhysicsController,
}

impl Initializable for ItemDropSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let physics = inventory.get::<PhysicsController>().clone();
    Self { inputs, physics }
  }
}

impl System for ItemDropSystem {
  fn provide(&mut self, inventory: &Inventory) {
    MagneticPickupComponent::register();
    InventoryComponent::register();
    ResourceComponent::register();
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let mut to_drop = Vec::new();

    for (_, (transform, _)) in &mut scene.query::<(&TransformComponent, &SelfComponent)>() {
      let input = self.inputs.read();
      if input.debug {
        to_drop.push(transform.translation.clone());
      }
    }

    // We really need a way to not have to do all this extra iteration
    for translation in to_drop {
      if let Some(mut collectible_prefab) = scene.get_prefab("Bullet").cloned() {
        let collectible_entity = scene.create_raw_entity("Bullet");
        collectible_prefab.id = IdComponent::new();
        collectible_prefab.transform.translation = translation;
        for component in collectible_prefab.components.iter_mut() {
          if let Some(mut physics) = component.as_any_mut().downcast_mut::<PhysicsComponent>() {
            physics.joint.id = Uuid::new_v4();
            physics.joint.body.id = Uuid::new_v4();
          }
        }
        scene.create_with_prefab(collectible_entity, collectible_prefab);
      }
    }

    //Magnetic pickups processing
    let mut targets: Vec<Vector3<f32>> = vec![];
    for (_, (_, transform)) in scene.query_mut::<(&SelfComponent, &TransformComponent)>() {
      targets.push(transform.translation.clone())
    }

    // Process each pickup
    for (_, (physics, transform, pickup, tag)) in scene.query_mut::<(
      &mut PhysicsComponent,
      &mut TransformComponent,
      &mut MagneticPickupComponent,
      &TagComponent,
    )>() {
      // Find the closest target within detection radius
      if let Some((closest_target, closest_distance)) = targets
        .iter()
        .map(|&target| (target, (transform.translation - target).magnitude()))
        .filter(|&(_, distance)| distance <= *pickup.detection_radius)
        .min_by(|&(_, dist_a), &(_, dist_b)| {
          dist_a
            .partial_cmp(&dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        })
      {
        let pickup_direction = (UnitQuaternion::from_euler_angles(
          transform.rotation.x,
          transform.rotation.y,
          transform.rotation.z,
        ) * Vector3::new(0.0, 0.0, 1.0));

        // self.physics.rotate_and_move_towards(
        //   physics,
        //   transform.rotation,
        //   transform.translation,
        //   closest_target,
        //   pickup_direction,
        //   pickup.speed,
        //   pickup.rotation_speed,
        // );
      }
    }

    //Magnetic pickups processing
    let mut targets: Vec<Vector3<f32>> = vec![];
    for (_, (_, transform)) in scene.query_mut::<(&SelfComponent, &TransformComponent)>() {
      targets.push(transform.translation.clone())
    }

    // Process each pickup
    for (_, (physics, transform, pickup, tag)) in scene.query_mut::<(
      &mut PhysicsComponent,
      &mut TransformComponent,
      &mut MagneticPickupComponent,
      &TagComponent,
    )>() {
      // Find the closest target within detection radius
      if let Some((closest_target, closest_distance)) = targets
        .iter()
        .map(|&target| (target, (transform.translation - target).magnitude()))
        .filter(|&(_, distance)| distance <= *pickup.detection_radius)
        .min_by(|&(_, dist_a), &(_, dist_b)| {
          dist_a
            .partial_cmp(&dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        })
      {
        let pickup_direction = (UnitQuaternion::from_euler_angles(
          transform.rotation.x,
          transform.rotation.y,
          transform.rotation.z,
        ) * Vector3::new(0.0, 0.0, 1.0));

      } else {
        self
          .physics
          .set_angvel(physics, Vector3::new(0.0, 0.0, 0.0));
        self
          .physics
          .set_linvel(physics, Vector3::new(0.0, 0.0, 0.0));
      }
    }
  }
}

/*

// Leaving this here in case we still want to implent this at some point
    scene.spawn_prefab("Dreamstone", |prefab| -> Prefab {
        prefab.id = Uuid::new();

        if let Some(physics) = prefab.get_mut::<PhysicsComponent>() {
            // physics
        }

        prefab.get::<PhysicsComponent>();
        prefab.remove::<PhysicsComponent>();
        prefab.add::<PhysicsComponent>();

        prefab
        });
 */
