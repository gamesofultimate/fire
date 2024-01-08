#![cfg(target_arch = "wasm32")]
use crate::shared::{
  components::{enemy_ai_component::EnemyAiComponent, spawn_component::SpawnComponent},
  input::PlayerInput,
};
use engine::application::scene::component_registry::Access;

use engine::{
  application::{
    components::{AnimationComponent, InputComponent, PhysicsComponent, SelfComponent},
    input::DefaultInput,
    physics3d::CollisionEvent,
    scene::{IdComponent, Scene, TransformComponent},
  },
  systems::{
    input::{CanvasController, InputsReader},
    physics::{CollisionsReader, PhysicsConfig, PhysicsController},
    Backpack, Initializable, Inventory, System,
  },
  utils::units::Time,
  Entity,
};
use nalgebra::Vector3;
use rand::{distributions::Uniform, Rng};
use uuid::Uuid;

pub struct SpawnSystem {
  physics: PhysicsController,
}

impl Initializable for SpawnSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let physics = inventory.get::<PhysicsController>().clone();
    Self { physics }
  }
}

impl System for SpawnSystem {
  fn attach(&mut self, _: &mut Scene, _: &mut Backpack) {
    return;
  }

  fn provide(&mut self, inventory: &Inventory) {
    SpawnComponent::register();
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let dt = **backpack.get::<Time>().unwrap();
    let mut rng = rand::thread_rng();
    let mut swampeter_prefab;
    if let Some(prefab) = scene.get_prefab("Swampeter").cloned() {
      swampeter_prefab = prefab;
    } else {
      return;
    }

    let mut new_transform;

    // First pass: Collect the entities and their new positions
    let mut new_entities = Vec::new();
    for (entity, (spawn, transform, id)) in
      scene.query_mut::<(&mut SpawnComponent, &TransformComponent, &IdComponent)>()
    {
      spawn.timer += dt;

      if spawn.timer >= spawn.interval && spawn.spawn_count() < spawn.max_enemies {
        let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        let distance = rng.gen_range(2.0..spawn.radius);
        let offset_x = distance * angle.cos();
        let offset_z = distance * angle.sin();

        let mut new_prefab = swampeter_prefab.clone();

        new_transform = transform.clone();
        new_transform.translation.x += offset_x;
        new_transform.translation.z += offset_z;
        new_transform.scale = Vector3::new(1.0, 1.0, 1.0);
        new_transform.rotation = Vector3::new(0.0, 0.0, 0.0);

        new_prefab.id = IdComponent::new();
        new_prefab.transform = new_transform;

        for component in new_prefab.components.iter_mut() {
          if let Some(mut ai) = component.as_any_mut().downcast_mut::<EnemyAiComponent>() {
            ai.set_spawned_from(**id)
          }
        }

        spawn.spawn_enemy();
        new_entities.push((new_prefab, new_transform));
        spawn.timer = 0.0;
      }
    }

    // Second pass: Create new entities with the collected information
    for (mut new_prefab, new_transform) in new_entities {
      for component in new_prefab.components.iter_mut() {
        if let Some(mut physics) = component.as_any_mut().downcast_mut::<PhysicsComponent>() {
          physics.joint.id = Uuid::new_v4();
          physics.joint.body.id = Uuid::new_v4();
        }
      }

      let swampeter_entity = scene.create_raw_entity("Swampeter");
      scene.create_with_prefab(swampeter_entity, new_prefab);
    }
  }
}
