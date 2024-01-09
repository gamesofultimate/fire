#![cfg(target_arch = "wasm32")]
use crate::shared::{
  components::{
    health_component::HealthComponent, inventory_component::InventoryComponent,
    movement_component::MovementComponent,
  },
  input::PlayerInput,
};
use engine::application::scene::component_registry::Access;
use engine::{
  application::{
    components::{AnimationComponent, InputComponent, PhysicsComponent, SelfComponent},
    input::DefaultInput,
    physics3d::{ColliderHandle, CollisionEvent},
    scene::{Scene, TagComponent, TransformComponent},
  },
  systems::{
    input::{CanvasController, InputsReader},
    physics::{CollisionsReader, PhysicsConfig, PhysicsController},
    Backpack, Initializable, Inventory, System,
  },
  utils::units::{Kph, Seconds, Time},
  Entity,
};
use nalgebra::{Rotation2, Rotation3, UnitQuaternion, Vector2, Vector3};
use std::{
  char::MAX,
  f32::consts::{FRAC_PI_2, PI},
};

/// This file contains code related to assigning actions based on ingame collision events.
/// This is probably a temporary solution until we can get a proper API working for it.

pub struct CollisionSystem {
  inputs: InputsReader<PlayerInput>,
  physics: PhysicsController,
  collisions_reader: CollisionsReader,
}

impl Initializable for CollisionSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let physics = inventory.get::<PhysicsController>().clone();
    let collisions_reader = inventory.get::<CollisionsReader>().clone();

    Self {
      inputs,
      physics,
      collisions_reader,
    }
  }
}

impl System for CollisionSystem {
  fn attach(&mut self, _: &mut Scene, backpack: &mut Backpack) {
    return;
  }

  fn provide(&mut self, inventory: &Inventory) {
    // CollisionComponent::register();
    InventoryComponent::register();
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let collisions = self.collisions_reader.read().collect::<Vec<_>>();
    for collision_event in collisions {
      match collision_event {
        CollisionEvent::Started(collider1, collider2, _) => {
          let (entity1, entity2, entity1_tag, entity2_tag) =
            self.get_entity_and_tag(scene, collider1, collider2);

          if entity1_tag.name == "Dreamstone" && entity2_tag.name == "Wizard" {
            self.handle_foxy_dreamstone_collision_start(scene, entity2, entity1);
          } else if entity2_tag.name == "Dreamstone" && entity1_tag.name == "Wizard" {
            self.handle_foxy_dreamstone_collision_start(scene, entity1, entity2);
          }
          if entity1_tag.name == "Terrain" && entity2_tag.name == "Wizard" {
            self.handle_foxy_terrain_collision_start(scene, entity2);
          } else if (entity2_tag.name == "Terrain" && entity1_tag.name == "Wizard") {
            self.handle_foxy_terrain_collision_start(scene, entity1);
          }

          if entity1_tag.name == "Swampeter" && entity2_tag.name == "Wizard" {
            self.handle_foxy_swampeter_collision_start(scene, entity2, entity1);
          } else if entity2_tag.name == "Swampeter" && entity1_tag.name == "Wizard" {
            self.handle_foxy_swampeter_collision_start(scene, entity1, entity2);
          }
        }

        CollisionEvent::Stopped(collider1, collider2, _) => {
          let (entity1, entity2, entity1_tag, entity2_tag) =
            self.get_entity_and_tag(scene, collider1, collider2);
          if entity1_tag.name == "plane" && entity2_tag.name == "Wizard" {
            self.handle_foxy_terrain_collision_stop(scene, entity2);
          } else if entity2_tag.name == "plane" && entity1_tag.name == "Wizard" {
            self.handle_foxy_terrain_collision_stop(scene, entity1);
          }
        }
      }
    }

    let dt = **backpack.get::<Time>().unwrap();
  }
}

impl CollisionSystem {
  fn get_entity_and_tag(
    &self,
    mut scene: &mut Scene,
    collider1: ColliderHandle,
    collider2: ColliderHandle,
  ) -> (Entity, Entity, TagComponent, TagComponent) {
    let entity1 = self
      .physics
      .get_entity_from_collider_handle(collider1)
      .unwrap();

    let entity2 = self
      .physics
      .get_entity_from_collider_handle(collider2)
      .unwrap();

    let entity1_tag = scene
      .query_one_mut::<&TagComponent>(entity1)
      .unwrap()
      .clone();

    let entity2_tag = scene
      .query_one_mut::<&TagComponent>(entity2)
      .unwrap()
      .clone();

    (entity1, entity2, entity1_tag, entity2_tag)
  }
  fn handle_foxy_terrain_collision_start(&mut self, scene: &mut Scene, foxy_entity: Entity) {
    let mut movement: &mut MovementComponent = scene
      .query_one_mut::<&mut MovementComponent>(foxy_entity)
      .unwrap();

    movement.land();
  }

  fn handle_foxy_terrain_collision_stop(&mut self, scene: &mut Scene, foxy_entity: Entity) {
    let mut movement: &mut MovementComponent = scene
      .query_one_mut::<&mut MovementComponent>(foxy_entity)
      .unwrap();

    movement.jump();
  }

  fn handle_foxy_dreamstone_collision_start(
    &mut self,
    scene: &mut Scene,
    foxy_entity: Entity,
    dreamstone_entity: Entity,
  ) {
    let physics: &mut PhysicsComponent = scene
      .query_one_mut::<&mut PhysicsComponent>(dreamstone_entity)
      .unwrap();

    self.physics.despawn(&physics);
    scene.remove_entity(dreamstone_entity);

    let player_inventory: &mut InventoryComponent = scene
      .query_one_mut::<&mut InventoryComponent>(foxy_entity)
      .unwrap();

    player_inventory.add_dreamstones(1);
  }

  fn handle_foxy_swampeter_collision_start(
    &mut self,
    scene: &mut Scene,
    foxy_entity: Entity,
    swampeter_entity: Entity,
  ) {
    let mut health = scene
      .query_one_mut::<&mut HealthComponent>(foxy_entity)
      .unwrap();

    health.pending_damage += 5.0;
  }
}
