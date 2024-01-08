use engine::{
  application::{components::SkyLightComponent, scene::Scene},
  systems::{Backpack, Initializable, Inventory, System},
  utils::units::Radians,
};

pub struct SkySystem {
  timing: f32,
}

impl Initializable for SkySystem {
  fn initialize(_: &Inventory) -> Self {
    Self { timing: 0.0 }
  }
}

impl System for SkySystem {
  fn run(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    /*
    for (_, sky) in scene.query_mut::<&mut SkyLightComponent>() {
      match sky {
        SkyLightComponent::Dynamic {
          id: _,
          intensity: _,
          turbidity: _,
          azimuth: _,
          inclination,
        } => {
          // *azimuth = Radians::new(self.timing.cos() * 180.0);
          *inclination = Radians::new(self.timing.cos() * 90.0);
        }
        _ => {}
      }
    }
    */

    self.timing += 0.0005;
  }
}
