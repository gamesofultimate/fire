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
    rendering::{CameraConfig, DebugController},
    Backpack, Initializable, Inventory, System,
  },
  utils::units::{Kph, Seconds, Time},
  Entity,
};
use nalgebra::{Point3, Rotation2, Rotation3, UnitQuaternion, Vector2, Vector3, Vector4};
use rapier3d::prelude::{QueryFilter, Ray};
/// This file contains code related to moving and orienting the player's physical position
/// in the game. For camera positioning, see src/client/camera.rs

pub struct PlayerMovementSystem {
  inputs: InputsReader<PlayerInput>,
  physics: PhysicsController,
  running_time: f32,
}

impl Initializable for PlayerMovementSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let physics = inventory.get::<PhysicsController>().clone();

    Self {
      inputs,
      physics,
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
    self.handle_input(scene, dt, backpack);
    self.running_time += dt;
  }
}

impl PlayerMovementSystem {
  fn handle_input(&mut self, scene: &mut Scene, dt: f32, backpack: &mut Backpack) {
    for (_, (physics, transform, movement, input_component)) in scene.query_mut::<(
      &mut PhysicsComponent,
      &mut TransformComponent,
      &mut MovementComponent,
      &mut InputComponent,
    )>() {
      let camera = backpack.get_mut::<CameraConfig>().unwrap();
      let input = self.inputs.read();

      let (start, end) =
      self.mouse_to_ray(camera, &transform.translation, &input, &input_component);

      let debug_controller = backpack.get_mut::<DebugController>().unwrap();
      // Convert Point3 to Vector3 for the start point
      let start_vector = start - Point3::origin();

      // Draw the debug ray
      debug_controller.draw_ray(
        start_vector,                     // Start point as Vector3
        (end - start),             // Direction as Vector3
        Vector4::new(1.0, 0.0, 0.0, 1.0), // Red color for the ray
        2.0,                              // Thickness of the ray
      );

      let ray = Ray::new(start, (end - start).normalize());
      let filter = QueryFilter::default(); // Adjust this as needed
      let solid = false; // Example boolean value
      let max_distance = (end - start).magnitude();

      let rotation_quaternion = UnitQuaternion::from_euler_angles(
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
      );

      if let Some((hit_entity, _, _)) = self.physics.raycast(&ray, max_distance, solid, filter) {
        log::info!("hit_entity: {:?}", hit_entity);
      }

      let input_point = Vector3::new(input.direction_vector.x, 0.0, -input.direction_vector.z);

      let old_velocity = self.physics.linvel(&physics);
      if input_point.magnitude() > 0.0 {
        // Rotate the input direction using the player's current rotation
        let rotated_direction = rotation_quaternion.transform_vector(&input_point);

        let velocity = Vector3::new(
          -rotated_direction.x * *movement.run_speed,
          old_velocity.y,
          -rotated_direction.z * *movement.run_speed,
        );
        self.physics.set_linvel(&physics, velocity);
      } else {
        self.physics.set_linvel(&physics, Vector3::zeros());
      }

      // Handle rotation based on mouse input
      let yaw_delta = -*movement.rotation_speed * dt * input.mouse_delta.x;
      self
        .physics
        .set_angvel(&physics, Vector3::new(0.0, yaw_delta, 0.0));
    }
  }

  fn mouse_to_ray(
    &mut self,
    camera: &CameraConfig,
    transform_translation: &Vector3<f32>,
    input: &PlayerInput,
    input_component: &InputComponent,
  ) -> (Point3<f32>, Point3<f32>) {
    let projection_matrix = camera.get_projection().to_homogeneous(); // Get the projection matrix
    let view_matrix = camera.get_view(); // Get the view matrix
    let inverse = (projection_matrix * view_matrix).try_inverse().unwrap(); // Get the inverse of the view-projection matrix

    let (mouse_x, mouse_y) = (input.mouse_position.x as u32, input.mouse_position.y as u32); // Destructure the mouse position coordinates
    let pixel = camera.canvas_to_screenspace(mouse_x, mouse_y); // Convert the mouse position to screenspace

    let view_start = {
      // Calculate the start of the ray
      let screenspace_position = Vector4::new(pixel.x, pixel.y, -1.0, 1.0);
      let projected = inverse * screenspace_position;
      projected / projected.w
    };

    let view_end = {
      // Calculate the end of the ray
      let screenspace_position = Vector4::new(pixel.x, pixel.y, 1.0, 1.0);
      let projected = inverse * screenspace_position;
      projected / projected.w
    };

    let start = Point3::new(view_start.x, view_start.y, view_start.z);
    let end = Point3::new(view_end.x, view_end.y, view_end.z);

    log::info!("start: {:?}, end: {:?}", start, end);
    (start, end)
  }

  fn perform_raycast(
    &self,
    scene: &mut Scene,
    physics: &PhysicsController,
    start: Vector3<f32>,
    end: Vector3<f32>,
  ) {
    // Calculate the direction of the ray
    let direction = (end - start).normalize();

    // Define the maximum distance of the ray
    let max_distance = (end - start).magnitude();

    // Perform the raycast
    // Note: The actual method to perform a raycast depends on your physics engine
    // and how it's integrated into your system. This is a generic example.
    // physics.raycast(scene, start, direction, max_distance);
  }
}
