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
const CAMERA_HEIGHT: f32 = 10.0; // Height of the camera above the player
const CAMERA_BACK_OFFSET: f32 = 5.0; // Distance behind the player

pub struct CameraSystem {
    inputs: InputsReader<PlayerInput>,
}

impl Initializable for CameraSystem {
    fn initialize(inventory: &Inventory) -> Self {
        let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
        Self { inputs }
    }
}

impl System for CameraSystem {
    fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
        for (_, (_, transform, camera, _)) in &mut scene.query::<(
            &IdComponent,
            &TransformComponent,
            &CameraComponent,
            &SelfComponent,
        )>() {
            // Calculate the camera position: Fixed above and slightly behind the player
            let camera_position = Vector3::new(
                transform.translation.x,
                CAMERA_HEIGHT,
                transform.translation.z - CAMERA_BACK_OFFSET,
            );

            // Direction from camera to player
            let direction_to_player = transform.translation - camera_position;

            // Calculate the camera's orientation using look_at_lh
            let camera_orientation = UnitQuaternion::look_at_lh(&direction_to_player, &Vector3::y_axis());

            if let CameraComponent::Perspective { fovy, zfar, znear, .. } = camera
                && let Some(camera_config) = backpack.get_mut::<CameraConfig>() {
                    camera_config.fovy = *fovy;
                    camera_config.znear = *znear;
                    camera_config.zfar = *zfar;
                    camera_config.translation = camera_position;
                    // Front vector is the normalized opposite direction
                    camera_config.front = Unit::new_normalize(direction_to_player);
                    // The camera's up direction is set to the global y-axis
                    camera_config.up = Unit::new_normalize(Vector3::y());
            }
        }
    }
}
