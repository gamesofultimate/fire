mod camera;
mod world;

use crate::shared::animations::attack_transitions::{AttackTransitions, AttackTypeTransition};
use engine::{
  application::{
    bus::BrowserBus,
    input::{DefaultInput, TrustedInput},
    scene::Prefab,
  },
  systems::{
    hdr::HdrMultiplayerPipeline, network::NetworkPlugin, rendering::RenderingPlugin,
    trusty::TrustySystem, Middleware, Scheduler,
  },
  utils::browser::grow_memory,
};

use crate::shared::{follow::MayhemBehaviors, goal::GoalSystem, input::PlayerInput};

use crate::shared::systems::{
  collisions::CollisionSystem, combat::CombatSystem, death::DeathSystem, lifetime::LifetimeSystem,
  player_movement::PlayerMovementSystem, spawn::SpawnSystem,
};

// 4k
/*
const DEFAULT_WIDTH:u32 = 3840;
const DEFAULT_HEIGHT:u32 = 2160;
*/
// 1080p
const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGHT: u32 = 1080;
/*
*/
/* 720p
const DEFAULT_WIDTH:u32 = 1280;
const DEFAULT_HEIGHT:u32 = 720;
*/

const FRAMES_PER_SECOND: u64 = 60;
const GROW_MEMORY_IN_MB: u32 = 800;

pub async fn main(
  canvas_id: String,
  assets_location: String,
  bus: BrowserBus,
  session_id: String,
  access_token: String,
  udp_url: String,
  tcp_url: String,
) {
  wasm_logger::init(wasm_logger::Config::default());
  grow_memory(GROW_MEMORY_IN_MB);
  let mut runner = Scheduler::new(FRAMES_PER_SECOND, canvas_id);

  log::debug!("assets location: {:?}", &assets_location);

  let hdr = HdrMultiplayerPipeline::<PlayerInput>::new(
    assets_location,
    session_id,
    access_token,
    udp_url,
    tcp_url,
  );

  runner.attach_plugin(hdr);
  // runner.attach_middleware::<AttackTransitions>();
  // runner.attach_middleware::<MayhemBehaviors>();
  runner.attach_system::<world::WorldSystem>();
  runner.attach_system::<PlayerMovementSystem>();
  runner.attach_system::<GoalSystem>();
  runner.attach_system::<CollisionSystem>();

  runner.attach_system::<camera::CameraSystem>();
  runner.attach_system::<CombatSystem>();
  // runner.attach_system::<LifetimeSystem>();
  runner.attach_system::<ItemDropSystem>();
  // runner.attach_system::<DeathSystem>();
  // runner.attach_system::<SpawnSystem>();
  // runner.attach_system::<EnemyAiSystem>();
  runner.run().await;
}
