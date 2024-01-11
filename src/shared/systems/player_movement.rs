#![cfg(target_arch = "wasm32")]
use crate::shared::{components::movement_component::MovementComponent, input::PlayerInput};
use engine::application::scene::{component_registry::Access, IdComponent, TagComponent};
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
use nalgebra::{
  Isometry3, Point3, Rotation2, Rotation3, Unit, UnitQuaternion, Vector2, Vector3, Vector4,
};
use parry3d::{query::RayCast, shape::HalfSpace};
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
    for (entity, (physics, transform, movement, input_component, _)) in scene.query_mut::<(
      &mut PhysicsComponent,
      &mut TransformComponent,
      &mut MovementComponent,
      &mut InputComponent,
      &mut SelfComponent,
    )>() {
      let camera = backpack.get_mut::<CameraConfig>().unwrap();
      let input = self.inputs.read();
      let (start, end) = self.mouse_to_ray(camera, &input);
      let debug_controller = backpack.get_mut::<DebugController>().unwrap();

      // Draw the debug ray
      debug_controller.draw_ray(
        start_vector,                     // Start point as Vector3
        (end - start),                    // Direction as Vector3
        Vector4::new(1.0, 0.0, 0.0, 1.0), // Red color for the ray
        2.0,                              // Thickness of the ray
      );

      let ray = Ray::new(start, (end - start).normalize());
      let filter = QueryFilter::default();
      let solid = false;
      let max_distance = (end - start).magnitude();

      let rotation_quaternion = UnitQuaternion::from_euler_angles(
        transform.rotation.x,
        transform.rotation.y,
        transform.rotation.z,
      ) * Vector3::new(0.0, 0.0, 1.0);

      if let Some((hit_entity, _, _)) = self.physics.raycast(&ray, max_distance, solid, filter) {
        // need to put this outside of query loop somehow.
        // let (id, tag) = scene
        //   .get_components::<(&IdComponent, &TagComponent)>(hit_entity)
        //   .unwrap();
        // log::debug!("id: {:?} tag: {:?}", id, tag);
        // log::debug!("hit_entity: {:?}", hit_entity);
      }

      let input_point = Vector3::new(input.direction_vector.x, 0.0, -input.direction_vector.z);

      // ====================================================================================================
      let plane = HalfSpace::new(Unit::new_normalize(Vector3::y()));
      let isometry = Isometry3::translation(0.0, 0.0, 0.0);
      let intersection = match plane.cast_ray(&isometry, &ray, std::f32::MAX, false) {
        Some(toi) => {
          let point_in_plane = start + direction * toi;
          Some(point_in_plane)
        }
        None => None,
      };

      // In this function, we want to make the character move towards point_in_plane
      if input.left_click {
        movement.target_point = intersection;
      }

      match movement.target_point {
        Some(point_in_plane) => {
          log::info!("target_point: {:?}", point_in_plane);
        }
        None => {
          log::info!("target_point: None");
        }
      }

      if let Some(point_in_plane) = movement.target_point {
        let direction = point_in_plane - transform.translation;
        let distance_to_target = direction.coords.magnitude();

        // Define a threshold for how close the character needs to be to the target
        let close_enough_threshold = 0.1;

        if distance_to_target > close_enough_threshold {
          // If the character is not close enough, continue moving towards the target
          let point_in_plane_vector = Vector3::new(point_in_plane.x, 0.0, point_in_plane.z);
          let direction_vector = Vector3::new(direction.x, 0.0, direction.z);

          self.physics.move_towards(
            &physics,
            transform.translation,
            point_in_plane_vector,
            movement.run_speed,
          );

          self.physics.rotate_towards(
            &physics,
            rotation_quaternion,
            direction_vector,
            movement.rotation_speed,
          )
        } else {
          // If the character is close enough, stop moving and reset the target point
          movement.target_point = None;
          self.physics.set_linvel(&physics, Vector3::zeros());
          self.physics.set_angvel(&physics, Vector3::zeros());
          // Handle rotation based on mouse input
          // let yaw_delta = -*movement.rotation_speed * dt * input.mouse_delta.x;
          // self
          //   .physics
          //   .set_angvel(&physics, Vector3::new(0.0, yaw_delta, 0.0));
        }
      }

      // ====================================================================================================

      // Handle rotation based on mouse input
      // let yaw_delta = -*movement.rotation_speed * dt * input.mouse_delta.x;
      // self
      //   .physics
      //   .set_angvel(&physics, Vector3::new(0.0, yaw_delta, 0.0));
    }
  }

  fn mouse_to_ray(
    &mut self,
    camera: &CameraConfig,
    input: &PlayerInput,
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

    (start, end)
  }
}
