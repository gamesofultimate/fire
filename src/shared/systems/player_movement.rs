#![cfg(target_arch = "wasm32")]
use crate::shared::{components::movement_component::MovementComponent, input::PlayerInput};
use engine::application::scene::component_registry::Access;
use engine::{
  application::{
    components::{AnimationComponent, InputComponent, PhysicsComponent, SelfComponent},
    input::DefaultInput,
    physics3d::CollisionEvent,
    scene::{Scene, TransformComponent},
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

/// This file contains code related to moving and orienting the player's physical position
/// in the game. For camera positioning, see src/client/camera.rs

pub struct PlayerMovementSystem {
  inputs: InputsReader<PlayerInput>,
  physics: PhysicsController,
  canvas: CanvasController,
  running_time: f32,
}

impl Initializable for PlayerMovementSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let physics = inventory.get::<PhysicsController>().clone();
    let canvas = inventory.get::<CanvasController>().clone();

    Self {
      inputs,
      physics,
      canvas,
      running_time: 0.0,
    }
  }
}

impl System for PlayerMovementSystem {
  fn attach(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    for (_, (_, physics)) in scene.query_mut::<(&SelfComponent, &PhysicsComponent)>() {
      self.physics.set_linear_damping(&physics, 0.6);
      self.physics.set_angular_damping(&physics, 0.6);
    }
  }

  fn provide(&mut self, inventory: &Inventory) {
    MovementComponent::register();
  }

  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let dt = **backpack.get::<Time>().unwrap();
    self.handle_input(scene, dt);
    self.running_time += dt;
  }
}

impl PlayerMovementSystem {
  fn handle_input(&mut self, scene: &mut Scene, dt: f32) {
    for (_, (physics, transform, movement, _)) in scene.query_mut::<(
      &mut PhysicsComponent,
      &mut TransformComponent,
      &mut MovementComponent,
      &mut InputComponent,
    )>() {
      let input = self.inputs.read();
      movement.dash_timer += Seconds::new(dt);

      let rotation_quaternion = UnitQuaternion::from_euler_angles(
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
      );

      let input_point = Vector3::new(input.direction_vector.x, 0.0, -input.direction_vector.z);

      let old_velocity = self.physics.linvel(&physics);
      if input_point.magnitude() > 0.0 {
        // Rotate the input direction using the player's current rotation
        let rotated_direction = rotation_quaternion.transform_vector(&input_point);

        let player_speed = if movement.is_dashing {
          *movement.dash_speed
        } else if input.sprint {
          *movement.sprint_speed
        } else {
          *movement.walk_speed
        };

        let velocity = Vector3::new(
          -rotated_direction.x * player_speed,
          old_velocity.y,
          -rotated_direction.z * player_speed,
        );

        self.physics.set_linvel(&physics, velocity);
      } else {
        // self.physics.set_linvel(&physics, Vector3::new(0.0, old_velocity.y, 0.0));
      }

      // Handle rotation based on mouse input
      let yaw_delta = -*movement.rotation_speed * dt * input.mouse_delta.x;
      self
        .physics
        .set_angvel(&physics, Vector3::new(0.0, yaw_delta, 0.0));

      // Handle altering gravity during jump.
      if movement.is_grounded {
        self.physics.set_gravity_scale(&physics, 1.0)
      }
      if !movement.is_grounded && self.physics.linvel(&physics).y < 0.0 {
        self.physics.set_gravity_scale(&physics, 2.0)
      }

      // Handle Jump
      if (input.direction_vector.y > 0.0) && movement.is_grounded {
        self
          .physics
          .apply_impulse(&physics, Vector3::new(0.0, *movement.jump_force, 0.0));
      }

      // Handle Dash
      if input.dash && *movement.dash_timer >= 3.0 {
        movement.start_dash();
      }

      if movement.is_dashing && *movement.dash_timer > 0.75 {
        movement.stop_dash();
      }
    }
  }
}
