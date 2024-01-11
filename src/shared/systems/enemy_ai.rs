#![cfg(target_arch = "wasm32")]

use crate::shared::{
  components::{enemy_ai_component::EnemyAiComponent, movement_component::MovementComponent},
  game_types::game_types::EnemyState,
  input::PlayerInput,
};
use engine::application::scene::component_registry::Access;
use engine::{
  application::{
    components::{AnimationComponent, InputComponent, PhysicsComponent, SelfComponent},
    input::DefaultInput,
    physics3d::CollisionEvent,
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
use rand::{thread_rng, Rng};

pub struct EnemyAiSystem {
  physics: PhysicsController,
}

impl Initializable for EnemyAiSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let physics = inventory.get::<PhysicsController>().clone();
    Self { physics }
  }
}

impl System for EnemyAiSystem {
  fn attach(&mut self, _: &mut Scene, backpack: &mut Backpack) {}

  fn provide(&mut self, inventory: &Inventory) {
    EnemyAiComponent::register();
  }

  // Some things in here appear a bit wasteful, but not sure how else to do it.
  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let dt = **backpack.get::<Time>().unwrap();

    // Collect targets' (players') positions
    let mut targets: Vec<Vector3<f32>> = vec![];
    for (_, (_, transform)) in scene.query_mut::<(&SelfComponent, &TransformComponent)>() {
      targets.push(transform.translation.clone())
    }

    // Process each enemy
    for (_, (physics, transform, ai, tag)) in scene.query_mut::<(
      &mut PhysicsComponent,
      &mut TransformComponent,
      &mut EnemyAiComponent,
      &TagComponent,
    )>() {
      // Find the closest target within detection radius
      if let Some((closest_target, closest_distance)) = targets
        .iter()
        .map(|&target| (target, (transform.translation - target).magnitude()))
        .filter(|&(_, distance)| distance <= *ai.detection_radius)
        .min_by(|&(_, dist_a), &(_, dist_b)| {
          dist_a
            .partial_cmp(&dist_b)
            .unwrap_or(std::cmp::Ordering::Equal)
        })
      {
        // Update enemy state based on distance to target
        if closest_distance <= *ai.attack_range {
          ai.state = EnemyState::Attacking;
          let enemy_direction = self.get_enemy_direction(&transform);
          let direction = (closest_target - transform.translation);
          // self.physics.rotate_towards(
          //   physics,
          //   transform.rotation,
          //   enemy_direction,
          //   direction,
          //   ai.rotation_speed,
          // );
        } else {
          ai.state = EnemyState::Chasing;
          let enemy_direction = self.get_enemy_direction(&transform);

          let adjusted_target = Vector3::new(closest_target.x, 0.0, closest_target.z);
          // self.physics.rotate_and_move_towards(
          //   physics,
          //   transform.rotation,
          //   transform.translation,
          //   adjusted_target,
          //   enemy_direction,
          //   ai.speed,
          //   ai.rotation_speed,
          // );
        }
      } else {
        // No player in detection radius, remain idle or perform other actions
        ai.state = EnemyState::Idle;
        self.handle_pacing(ai, transform, physics, dt);
      }

      // Handle cooldown and other timers
      ai.timer += Seconds::new(dt);
      if ai.timer >= ai.attack_cooldown {
        ai.reset_timer();
        // Enemy can attack again
      }
    }
  }
}

impl EnemyAiSystem {
  fn get_enemy_direction(&self, transform: &TransformComponent) -> Vector3<f32> {
    (UnitQuaternion::from_euler_angles(
      transform.rotation.x,
      transform.rotation.y,
      transform.rotation.z,
    ) * Vector3::new(0.0, 0.0, 1.0))
  }

  fn handle_pacing(
    &mut self,
    ai: &mut EnemyAiComponent,
    transform: &mut TransformComponent,
    physics: &mut PhysicsComponent,
    dt: f32,
  ) {
    if ai.pacing_direction.is_none() || *ai.pacing_time_remaining <= 0.0 {
      // Start pacing in a new random direction for a random duration up to 20 seconds
      ai.pacing_direction = Some(self.pick_random_direction());
      ai.pacing_time_remaining = Seconds::new(thread_rng().gen_range(0.0..20.0));
    } else {
      // Calculate end point for this pacing step
      let pacing_step_distance = ai.pacing_speed * Seconds::new(dt); // Distance to move this frame
      let end_point =
        transform.translation + (ai.pacing_direction.unwrap() * *pacing_step_distance);

      // Rotate and move towards the calculated end point
      let enemy_direction = self.get_enemy_direction(transform);
      // self.physics.rotate_and_move_towards(
      //   physics,
      //   transform.rotation,
      //   transform.translation,
      //   end_point,
      //   enemy_direction,
      //   ai.pacing_speed,
      //   ai.rotation_speed,
      // );
      ai.pacing_time_remaining -= Seconds::new(dt);
    }
  }

  fn pick_random_direction(&self) -> Vector3<f32> {
    let mut rng = thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
    Vector3::new(angle.cos(), 0.0, angle.sin()) // Assuming movement on a flat plane
  }
}
