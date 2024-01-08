#![cfg(target_arch = "wasm32")]
use crate::shared::input::PlayerInput;
use serde::{Deserialize, Serialize};

use engine::{
  application::{
    components::LightComponent,
    // input::DefaultInput,
    scene::{Scene, UnpackEntity},
  },
  systems::{
    input::{CanvasController, InputsReader},
    network::{ServerReceiver, ServerSender},
    Backpack, Initializable, Inventory, System,
  },
  utils::units::{Radians, Time},
};
use nalgebra::Vector3;

// use crate::shared::player_controller::PlayerController;
use crate::shared::systems::combat::CombatSystem;
use crate::shared::systems::player_movement::PlayerMovementSystem;

pub struct WorldSystem {
  inputs: InputsReader<PlayerInput>,
  canvas: CanvasController,
}

impl Initializable for WorldSystem {
  fn initialize(inventory: &Inventory) -> Self {
    let inputs = inventory.get::<InputsReader<PlayerInput>>().clone();
    let canvas = inventory.get::<CanvasController>().clone();
    Self { inputs, canvas }
  }
}

impl WorldSystem {
  fn capture_mouse(&mut self, input: &PlayerInput) {
    self.canvas.capture_mouse(true);
    // self.canvas.request_fullscreen(true);
  }
}

impl System for WorldSystem {
  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    let delta = backpack.get::<Time>().unwrap();
    let input = self.inputs.read();

    if input.left_click && !input.mouse_lock {
      self.capture_mouse(&input);
    }

    for (_, light) in scene.query_mut::<&mut LightComponent>() {
      if let LightComponent::Directional {
        inclination,
        azimuth,
        ..
      } = light
      {
        //*inclination += Radians::new(0.001);
        //*azimuth += Radians::new(0.1);
      }
    }
    /*
    let mut directional_radiance = Vector3::zeros();

    let mut sky_azimuth = Radians::new(0.0);
    let mut sky_inclination = Radians::new(0.0);

    // Set the lighting orientation
    // This particular system sets it so that it's always moving
    // so we can showcase the realtime lighting system
    for (_, light) in scene.query_mut::<&LightComponent>() {
      if let LightComponent::Directional { radiance, azimuth, inclination, .. } = light {
        sky_azimuth = *azimuth;
        sky_inclination = *inclination;
        directional_radiance = radiance.clone();
      }
    }
    // Set the lighting orientation
    */
  }
}
