#![cfg(target_arch = "wasm32")]
use crate::shared::{
  components::{
    enemy_ai_component::EnemyAiComponent, health_component::HealthComponent,
    movement_component::MovementComponent, spawn_component::SpawnComponent,
  },
  game_types::game_types::PrefabType,
  input::PlayerInput,
};
use engine::application::scene::component_registry::Access;

use engine::{
  application::{
    components::{InputComponent, PhysicsComponent, SelfComponent},
    input::DefaultInput,
    physics3d::CollisionEvent,
    scene::{IdComponent, Scene, TagComponent, TransformComponent},
  },
  systems::{
    input::{CanvasController, InputsReader},
    physics::{CollisionsReader, PhysicsConfig, PhysicsController},
    Backpack, Initializable, Inventory, System,
  },
  Entity,
};
use nalgebra::{Rotation2, Rotation3, UnitQuaternion, Vector2, Vector3};
use uuid::Uuid;

pub struct DeathSystem {
  inputs: InputsReader<PlayerInput>,
  physics: PhysicsController,
}

impl Initializable for DeathSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let physics = inventory.get::<PhysicsController>().clone();
    Self { inputs, physics }
  }
}

impl System for DeathSystem {
  fn attach(&mut self, _: &mut Scene, backpack: &mut Backpack) {
    return;
  }

  fn provide(&mut self, inventory: &Inventory) {
    return;
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let mut dead_entities = vec![];
    for (entity, (health, tag, physics, transform)) in scene.query_mut::<(
      &mut HealthComponent,
      &TagComponent,
      &mut PhysicsComponent,
      &TransformComponent,
    )>() {
      if health.health <= 0.0 {
        dead_entities.push((
          entity.clone(),
          (physics.clone(), transform.clone(), tag.clone()),
        ));
      }
    }

    for (entity, (physics, transform, tag)) in dead_entities {
      if let Some(mut collectible_prefab) = scene.get_prefab("Dreamstone").cloned() {
        let collectible_entity = scene.create_raw_entity("Dreamstone");
        collectible_prefab.id = IdComponent::new();
        collectible_prefab.transform.translation = transform.translation;
        for component in collectible_prefab.components.iter_mut() {
          if let Some(mut physics) = component.as_any_mut().downcast_mut::<PhysicsComponent>() {
            physics.joint.id = Uuid::new_v4();
            physics.joint.body.id = Uuid::new_v4();
          }
        }

        if PrefabType::from(tag.name.as_str()) == PrefabType::Enemy {
          let ai: &mut EnemyAiComponent = scene
            .query_one_mut::<&mut EnemyAiComponent>(entity)
            .unwrap();

          let spawn = ai.spawned_from();

          let mut spawn_entity = match scene.get_entity_mut(spawn) {
            Some(entity) => Some(entity.clone()),
            None => None,
          };

          if spawn_entity.is_some() {
            match scene.query_one_mut::<&mut SpawnComponent>(spawn_entity.unwrap()) {
              Ok(component) => {
                component.remove_enemy();
              }
              Err(_) => (),
            };
          }
        }
        scene.create_with_prefab(collectible_entity, collectible_prefab);
        scene.remove_entity(entity);
        self.physics.despawn(&physics);
      }
    }
  }
}

impl DeathSystem {}
