use crate::shared::input::PlayerInput;
use engine::{
  application::{
    components::{CameraComponent, LightComponent, SelfComponent},
    scene::{IdComponent, Scene, TransformComponent},
  },
  systems::{
    input::InputsReader, rendering::CameraConfig, Backpack, Initializable, Inventory, System,
  },
  utils::units::{Radians, Time},
};
use nalgebra::{Isometry3, Point3, Unit, UnitQuaternion, Vector3};
use std::f32::consts::PI;

const ROTATION_SENSITIVITY: f32 = 0.1;

pub struct CameraSystem {
  inputs: InputsReader<PlayerInput>,
  // Will come back to put rotation somewhere else later but want to get it working for now.
  // There's only one camera per client anyways and this is client-side so it won't be an
  // issue for the time being.
  added_pitch: f32,
  rotation: UnitQuaternion<f32>,
}

impl Initializable for CameraSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    Self {
      inputs,
      added_pitch: 0.0,
      rotation: UnitQuaternion::identity(),
    }
  }
}

impl System for CameraSystem {
  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    for (_, (id, transform, camera, _)) in &mut scene.query::<(
      &IdComponent,
      &TransformComponent,
      &CameraComponent,
      &SelfComponent,
    )>() {
      let dt = **backpack.get::<Time>().unwrap();

      let character_front = transform.rotation;
      let character_yaw =
        UnitQuaternion::from_euler_angles(character_front.x, character_front.y, character_front.z);
      self.handle_input(dt, character_yaw);

      // Define offset vector to positsion the camera behind & above the player
      let camera_offset = Vector3::new(0.0, 5.0, -10.0);

      // Apply the player rotation to the camera offset to get the correct position
      let rotated_camera_offset = self.rotation * camera_offset;

      // Calculate the camera position by adding the rotated offset to the player's position
      let camera_position = transform.translation + rotated_camera_offset;

      // Get player translation and get current rotation quaternion using full rotation vector
      let character_position = transform.translation;

      if let CameraComponent::Perspective {fovy, zfar, znear, .. } = camera
          && let Some(camera) = backpack.get_mut::<CameraConfig>()
        {
          log::debug!("zfar {:}", zfar);

            camera.fovy = *fovy;
            camera.znear = *znear;
            camera.zfar = *zfar;
            camera.translation = camera_position;
            camera.front = Unit::new_normalize(character_position - camera_position);
            camera.up = Unit::new_normalize(Vector3::y());
        }
    }
  }
}

impl CameraSystem {
  fn handle_input(&mut self, dt: f32, player_yaw: UnitQuaternion<f32>) {
    let input = self.inputs.read();
    let pitch_speed = input.mouse_delta.y * ROTATION_SENSITIVITY * dt;
    self.added_pitch += pitch_speed;
    self.added_pitch = self.added_pitch.clamp(-PI / 3.0, PI / 8.0);

    let tilt_rotation = UnitQuaternion::from_euler_angles(self.added_pitch, 0.0, 0.0);
    self.rotation = player_yaw * tilt_rotation;
  }
}
