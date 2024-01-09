#![cfg(target_arch = "wasm32")]

use crate::shared::animations::attack_transitions::AttackTransitions;
use crate::shared::components::attack_component::AttackComponent;
use crate::shared::components::attack_component::AttackType;
use crate::shared::components::attack_component::AttackTypeDamage;
use crate::shared::components::attack_component::AIR_ATTACK;
use crate::shared::components::attack_component::HEAVY_ATTACK;
use crate::shared::components::attack_component::LIGHT_ATTACK;
use crate::shared::components::attack_component::NO_ATTACK;
use crate::shared::components::movement_component::MovementComponent;

use crate::shared::components::health_component::HealthComponent;
use crate::shared::components::lifetime_component::LifetimeComponent;
use crate::shared::components::shield_component::ShieldComponent;

use crate::shared::game_types::game_types::PrefabType;
use crate::shared::input::PlayerInput;
use engine::application::components::StateMachineComponent;
use engine::application::scene::component_registry::Access;
use engine::renderer::resources::animation::AnimationId;
use engine::systems::rendering::DebugController;
use engine::{
  application::{
    components::{
      AnimationComponent, InputComponent, ParticleComponent, PhysicsComponent, SelfComponent,
    },
    input::DefaultInput,
    physics3d::Physics3d,
    scene::{IdComponent, Scene, TagComponent, TransformComponent},
  },
  systems::{
    input::{CanvasController, InputsReader},
    physics::{PhysicsConfig, PhysicsController},
    Backpack, Initializable, Inventory, System,
  },
  utils::units::{Kph, Time},
  Entity,
};
use nalgebra::Rotation3;
use nalgebra::Unit;
use nalgebra::Vector3;
use nalgebra::Vector4;
use rapier3d::prelude::{vector, QueryFilter, Ray};
use std::collections::HashSet;
use uuid::uuid;

pub struct CombatSystem {
  damage_inflicted: f32,
  inputs: InputsReader<PlayerInput>,
  physics: PhysicsController,
  anim_timer: f32,
  anim_running: bool,
}

impl CombatSystem {
  pub fn new(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let mut physics = inventory.get::<PhysicsController>().clone();

    Self {
      damage_inflicted: 0.0,
      inputs,
      physics,
      anim_timer: 0.0,
      anim_running: false,
    }
  }

  pub fn damage_health(&mut self, scene: &mut Scene, damage: f32) {
    for (_, hp) in scene.query_mut::<&mut HealthComponent>() {
      hp.health -= damage;
    }
  }

  pub fn damage_shield(&mut self, scene: &mut Scene, damage: f32) {
    for (_, shield) in scene.query_mut::<&mut ShieldComponent>() {
      shield.shield -= damage;
    }
  }

  fn reset_animation(&mut self, attack: &mut AttackComponent) {
    self.anim_running = false;
    self.anim_timer = 0.0;
    attack.attack_type_damage = NO_ATTACK;
    attack.animation_fired = false;
  }

  fn handle_input(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let mut ent = Entity::DANGLING;
    let dt = **backpack.get::<Time>().unwrap();

    for (current_entity, (_, attack, movement, physics)) in scene.query_mut::<(
      &SelfComponent,
      &mut AttackComponent,
      &mut MovementComponent,
      &PhysicsComponent,
    )>() {
      if self.anim_running == true {
        self.anim_timer += dt * 1000.0;
      }

      if self.anim_timer > attack.light_anim_end_time && attack.attack_type_damage == LIGHT_ATTACK {
        self.reset_animation(attack);
      } else if self.anim_timer > attack.heavy_anim_end_time
        && attack.attack_type_damage == HEAVY_ATTACK
      {
        self.reset_animation(attack);
      } else if self.anim_timer > 1800.0 && attack.attack_type_damage == AIR_ATTACK {
        self.reset_animation(attack);
      }

      if attack.attacked == true {
        attack.cooldown_timer += dt;
      }

      if attack.cooldown_timer >= attack.cooldown {
        attack.attacked = false;
      }

      let input = self.inputs.read();

      attack.air_timer += dt;
      if (attack.attack_type_damage == LIGHT_ATTACK
        && self.anim_timer > attack.light_anim_start_time
        && attack.animation_fired == false)
        || (attack.attack_type_damage == HEAVY_ATTACK
          && self.anim_timer > attack.heavy_anim_start_time
          && attack.animation_fired == false)
        || (attack.attack_type_damage == AIR_ATTACK && attack.animation_fired == false)
      {
        ent = current_entity;

        let mut physics = self.physics.clone();
        let max_distance = attack.max_distance;
        let damage = attack.damage;
        let damage_multiplier = attack.attack_type_damage.damage_multiplier;
        attack.animation_fired = true;
        self.handle_attack(
          scene,
          ent,
          &mut physics,
          damage * damage_multiplier,
          max_distance,
          backpack,
        );
      }
      break;
    }
  }

  fn handle_raycast(
    &mut self,
    scene: &mut Scene,
    entity: Entity,
    world_physics: &mut PhysicsController,
    rays: Vec<Ray>,
    damage: f32,
    knockback_direction: Vector3<f32>,
    filter: QueryFilter,
    max_distance: f32,
  ) {
    let solid = false;
    let mut hit_entities = HashSet::new();
    for ray in rays {
      let mut hit = false;
      if let Some((entity, collider_handle, intersection)) =
        world_physics.raycast(&ray, max_distance, solid, filter)
      {
        if let Some((id, physics, health, shield, tag, movement)) = scene
          .query_mut::<(
            &IdComponent,
            &mut PhysicsComponent,
            &mut HealthComponent,
            &mut ShieldComponent,
            &TagComponent,
            Option<&mut MovementComponent>,
          )>()
          .view()
          .get_mut(entity)
        {
          if !hit_entities.contains(&entity) {
            health.pending_damage += damage;
            hit_entities.insert(entity);
          }
        }
      }
    }
  }

  fn handle_attack(
    &mut self,
    scene: &mut Scene,
    entity: Entity,
    world_physics: &mut PhysicsController,
    damage: f32,
    max_distance: f32,
    backpack: &mut Backpack,
  ) {
    let mut query = scene.query_mut::<(&TransformComponent, &PhysicsComponent)>();
    let mut direction_vectors = vec![];
    if let Some((transform, physics)) = query.view().get_mut(entity) {
      let center = transform.translation + Vector3::new(0.0, 0.5, 0.0);
      let rigid_body_handle = world_physics.get_rigid_body(&physics.joint.body.id);
      let filter = QueryFilter::default().exclude_rigid_body(rigid_body_handle.unwrap());
      let range = (damage as i32) / 2;

      let mut rot = Rotation3::from_euler_angles(
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
      );

      let mut direction = Unit::new_normalize(rot * Vector3::z());

      for i in -range..(range + 1) {
        let step = i as f32;
        rot = Rotation3::from_euler_angles(
          transform.rotation.x,
          transform.rotation.y + (step * 0.05),
          transform.rotation.z,
        );
        direction = Unit::new_normalize(rot * Vector3::z());

        direction_vectors.push(direction);
      }
      let debug_directions = direction_vectors.clone();
      if let Some(debug_controller) = backpack.get_mut::<DebugController>() {
        for dir in debug_directions {
          debug_controller.draw_ray(
            center.into(),
            dir.into_inner() * max_distance,
            Vector4::new(1.0, 1.0, 0.0, 1.0),
            10.0,
          );
        }
      }

      let knockback_direction =
        transform.get_euler_direction().into_inner() + vector![0.0, 1.0, 0.0];
      let knockback_direction = knockback_direction.normalize();

      let mut ray_vector = vec![];
      for dir in direction_vectors {
        let mut ray = Ray::new(center.into(), dir.into_inner());
        ray_vector.push(ray);
      }

      self.handle_raycast(
        scene,
        entity,
        world_physics,
        ray_vector,
        damage,
        knockback_direction,
        filter,
        max_distance,
      );
    }
  }

  fn handle_components(
    &mut self,
    scene: &mut Scene,
    backpack: &mut Backpack,
    world_physics: &mut PhysicsController,
  ) {
    let dt = **backpack.get::<Time>().unwrap();
    let mut positions = vec![];

    for (current_entity, (health, shield, tag, physics, transform, maybe_movement)) in scene
      .query_mut::<(
        &mut HealthComponent,
        &mut ShieldComponent,
        &TagComponent,
        &mut PhysicsComponent,
        &mut TransformComponent,
        Option<&MovementComponent>,
      )>()
    {
      if health.pending_damage > 0.0 {
        let delta_shield = shield.shield - health.pending_damage;

        if delta_shield >= 0.0 {
          shield.shield = delta_shield;
        } else {
          shield.shield = 0.0;
          health.health += delta_shield;
        }
        shield.time_last_damage = 0.0;
        health.pending_damage = 0.0;
        // PARTICLE SPAWN START
        let mut spawn_translation = transform.translation;
        spawn_translation.y += 0.5;
        positions.push(spawn_translation);
        // PARTICLE SPAWN END

        log::info!("{:?} Current Health {:?}", tag.name, health.health);
        log::info!("{:?} Current Shield {:?}", tag.name, shield.shield);
      }

      shield.time_last_damage += dt;
      if health.health > 0.0 {
        if shield.time_last_damage >= shield.current_undamaged_duration {
          shield.shield += shield.shield_regen_per_second;
          shield.time_last_damage -= 1.0;
        }
        if shield.shield >= shield.max_shield {
          shield.shield = shield.max_shield;
          shield.time_last_damage = 0.0;
        }
      }
    }

    for pos in &positions {
      if let Some(mut collectible_prefab) = scene.get_prefab("DamageParticle").cloned() {
        let collectible_entity = scene.create_raw_entity("DamageParticle");
        collectible_prefab.id = IdComponent::new();
        collectible_prefab.transform.translation = *pos;
        for component in collectible_prefab.components.iter_mut() {
          if let Some(mut lifetime) = component.as_any_mut().downcast_mut::<LifetimeComponent>() {
            lifetime.is_running = true;
          }
        }
        scene.create_with_prefab(collectible_entity, collectible_prefab);
      }
    }
    positions.clear();
  }
}

impl Initializable for CombatSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let mut physics = inventory.get::<PhysicsController>().clone();
    Self {
      damage_inflicted: 0.0,
      inputs,
      physics,
      anim_timer: 0.0,
      anim_running: false,
    }
  }
}

impl System for CombatSystem {
  fn provide(&mut self, inventory: &Inventory) {
    AttackComponent::register();
    HealthComponent::register();
    ShieldComponent::register();
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let input = self.inputs.read();
    let mut physics = self.physics.clone();

    self.handle_input(scene, backpack);
    self.handle_components(scene, backpack, &mut physics);
  }
}
