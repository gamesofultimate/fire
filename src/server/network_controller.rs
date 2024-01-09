use crate::shared::game_types::game_types::ModelNames;
use crate::shared::game_types::game_types::ParticleType;

use async_trait::async_trait;
use engine::application::gamefile::Gamefile;
use engine::application::scene::{IdComponent, Prefab, PrefabId, TransformComponent};
use engine::systems::Backpack;
use engine::{
  application::{
    assets::{AssetPack, Store},
    components::SelfComponent,
    config::Config,
    downloader::DownloadSender,
    input::TrustedInput,
    scene::{Scene, UnpackEntity},
  },
  networking::connection::{PlayerId, Protocol},
  renderer::resources::{
    animation::{Animation, AnimationDefinition, AnimationId},
    background::DynamicDefinition,
    fs::Resources,
    model::{Model, ModelDefinition, ModelId},
    particles::{Particle, ParticleDefinition, ParticleId},
    terrain::TerrainDefinition,
    texture::{Texture, TextureDefinition, TextureId},
  },
  systems::{
    network::{ChannelEvents, ClientSender},
    Initializable, Inventory,
  },
  Entity,
};
use std::collections::HashMap;
use std::collections::HashSet;

pub struct NetworkController {
  spectator_points: Vec<TransformComponent>,
  spawn_points: Vec<TransformComponent>,
  assigned_spawns: Vec<Option<PlayerId>>,
  prefabs: HashMap<ModelNames, Prefab>,
  particle_prefabs: HashMap<ParticleType, Prefab>,
  download_sender: DownloadSender,
  client_sender: ClientSender<TrustedInput>,
  config: Option<Config>,
  store: Store,
}

impl Initializable for NetworkController {
  fn initialize(inventory: &Inventory) -> Self {
    let download_sender = inventory.get::<DownloadSender>().clone();
    let client_sender = inventory.get::<ClientSender<TrustedInput>>().clone();
    let store = Store::new();
    Self {
      client_sender,
      download_sender,
      store,
      spectator_points: vec![],
      spawn_points: vec![],
      assigned_spawns: vec![],
      prefabs: HashMap::new(),
      particle_prefabs: HashMap::new(),
      config: None,
    }
  }
}

impl NetworkController {
  fn sync_world(&self, scene: &mut Scene, player_id: &PlayerId) {
    let mut definitions = vec![];

    for (id, definition) in self.store.iter_assets() {
      let packed = AssetPack::pack(definition);
      definitions.push(packed);
    }

    let entities_data: Vec<_> = scene
      .iter()
      .map(|entity| entity.entity())
      .collect::<Vec<Entity>>();

    let mut entities = vec![];

    for entity in entities_data {
      let mut prefab = Prefab::pack(scene, entity).unwrap();
      let is_self = **player_id == **prefab.id;
      if is_self {
        prefab.components.push(Box::new(SelfComponent {}));
      }
      entities.push(prefab);
    }

    let mut prefabs = vec![];
    for (name, prefab) in scene.iter_prefabs() {
      prefabs.push((name.clone(), prefab.clone()));
    }

    log::info!(
      "SYNC WORLD WITH {:?}\n{:#?}\n{:#?}\n{:#?}",
      &player_id,
      &definitions,
      &entities,
      &prefabs
    );

    if let Some(config) = &self.config {
      self.client_sender.send_reliable(
        *player_id,
        TrustedInput::Config {
          config: config.clone(),
        },
      );
    }

    self.client_sender.send_reliable(
      *player_id,
      TrustedInput::Assets {
        assets: definitions,
        trigger_loading: true,
      },
    );

    self
      .client_sender
      .send_reliable(*player_id, TrustedInput::Prefabs { prefabs });

    self
      .client_sender
      .send_reliable(*player_id, TrustedInput::Entities { entities });
  }
}

#[async_trait]
impl ChannelEvents for NetworkController {
  fn on_session_start(&mut self, scene: &mut Scene, backpack: &mut Backpack) {
    log::info!("Connected to sidecar!!!");

    let gamefile = Gamefile::from_file(&self.download_sender, "arena.lvl");
    //let gamefile = Gamefile::from_file(&self.download_sender, "gym-level.lvl");
    //log::info!("loaded: {:?}", gamefile);
    //let gamefile = Gamefile::from_file(&self.resources, "fps.lvl").await;

    self.config = Some(gamefile.config.clone());

    for (id, asset) in gamefile.scene.heightfields {
      self.store.insert_asset(asset.id, asset);
    }
    for (id, trimesh) in gamefile.scene.trimeshes {
      self.store.insert_asset(id, trimesh);
    }
    for (id, model) in gamefile.scene.models {
      self.store.insert_asset(id, model);
    }
    for (id, background) in gamefile.scene.dynamic_backgrounds {
      self.store.insert_asset(id, background);
    }
    for (id, animation) in gamefile.scene.animations {
      self.store.insert_asset(id, animation);
    }
    for (id, state_machine) in gamefile.scene.animation_state {
      self.store.insert_asset(id, state_machine);
    }
    for (id, textures) in gamefile.scene.textures {
      self.store.insert_asset(textures.id, textures);
    }
    for (id, particles) in gamefile.scene.particles {
      self.store.insert_asset(particles.id, particles);
    }
    for (id, asset) in gamefile.scene.terrains {
      self.store.insert_asset(asset.id, asset);
    }
    for (id, asset) in gamefile.scene.behavior_tree {
      self.store.insert_asset(asset.id, asset);
    }

    for (id, prefab) in gamefile.scene.prefabs {
      match prefab.tag.name.as_str() {
        "spectator-spawn-1" | "spectator-spawn-2" | "spectator-spawn-3" | "spectator-spawn-4" => {
          log::info!("creating spectator points {:?}", prefab.tag.name);
          self.spectator_points.push(prefab.transform);
        }
        "DamageParticle" => {
          log::info!("creating particle_system prefab: {:?}", prefab.tag.name);
          self
            .particle_prefabs
            .insert(ParticleType::Damage, prefab.clone());
          scene.store_prefab("DamageParticle", prefab);
        }
        // "Swampeter" => {
        //   log::info!("creating swampeter prefab: {:?}", prefab.tag.name);
        //   self.prefabs.insert(ModelNames::Swampeter, prefab.clone());
        //   scene.store_prefab("Swampeter", prefab);
        // }
        "EnemySpawn1" | "EnemySpawn2" | "EnemySpawn3" | "EnemySpawn4" => {
          log::info!("creating spawn points {:?}", prefab.tag.name);
          self.spawn_points.push(prefab.transform);
          self.assigned_spawns.push(None);
        }
        "spectator" => {
          log::info!("creating prefab: {:?}", prefab.tag.name);
          self.prefabs.insert(ModelNames::Spectator, prefab.clone());
        }
        "Wizard" => {
          log::info!("creating foxy prefab: {:?}", prefab.tag.name);
          self.prefabs.insert(ModelNames::Wizard, prefab.clone());
        }
        // "Dreamstone" => {
        //   log::info!("creating dreamstone prefab: {:?}", prefab.tag.name);
        //   self.prefabs.insert(ModelNames::Dreamstone, prefab.clone());
        //   scene.store_prefab("Dreamstone", prefab);
        // }
        "Bullet" => {
          log::info!("creating bullet prefab: {:?}", prefab.tag.name);
          self.prefabs.insert(ModelNames::Bullet, prefab.clone());
          scene.store_prefab("Bullet", prefab);
        }
        "arena-collider" => {
          log::info!("receiving entity {:?}", prefab.tag.name);
          let entity = scene.create_raw_entity("tmp");
          scene.create_with_prefab(entity, prefab);
        }
        _ => {
          log::info!("receiving entity {:?}", prefab.tag.name);
          let entity = scene.create_raw_entity("tmp");
          scene.create_with_prefab(entity, prefab);
        }
      }
    }

    /*
    let spectator_prefab = self.prefabs.get(&ModelNames::Spectator).unwrap().clone();
    let mut spectators = vec![];
    for point in &self.spectator_points {
      let mut prefab = spectator_prefab.clone();
      prefab.id.id = Uuid::new_v4();
      prefab.transform = Some(*point);
      spectators.push(prefab);
      //let local_transform = prefab.transform.clone().unwrap();
    }

    for spectator in spectators.drain(..) {
      self.receive_prefab(&spectator);
    }

    */
  }

  fn on_player_joined(
    &mut self,
    scene: &mut Scene,
    backpack: &mut Backpack,
    _: &HashSet<PlayerId>,
    entity: Entity,
    player_id: PlayerId,
    username: String,
    protocol: Protocol,
  ) {
    let mut prefab: Prefab = self.prefabs.get(&ModelNames::Wizard).unwrap().clone();
    log::info!("Player joined! New prefab: {:#?}", &prefab);

    if self.spawn_points.len() != 0 {
      let mut spawn_index = 0;
      for (index, assigned_spawn) in self.assigned_spawns.iter_mut().enumerate() {
        if *assigned_spawn == None {
          *assigned_spawn = Some(player_id);
          spawn_index = index;
          break;
        }
      }

      let spawn = self.spawn_points[spawn_index];

      let transform = &mut prefab.transform;

      transform.translation = spawn.translation;
      transform.rotation = spawn.rotation;
    }

    prefab.id = IdComponent::with_id(PrefabId::with_id(*player_id));
    let entity = scene.create_raw_entity(&username);
    scene.create_with_prefab(entity, prefab);
    self.sync_world(scene, &player_id);

    // let entity = scene.create_entity_with_id(prefab.id, &prefab.tag.name);

    /*
    // We need to send some information to clients, but that will be done in a
    // separate PR, so I'm leaving this here for now as a reference
    for (room_player_id, _network_entity) in &self.player_info {
      let mut prefab = prefab.clone();
      prefab.id.is_self = *room_player_id == player_id;
      let scene = vec![prefab];
      let assets = vec![];

      self.update_player(room_player_id, scene, assets);
    }
    */
  }

  fn on_player_left(
    &mut self,
    scene: &mut Scene,
    backpack: &mut Backpack,
    entity: Entity,
    player_id: PlayerId,
    protocol: Protocol,
  ) {
    log::info!("[on player left] Player left {player_id:?}");

    for assigned_spawn in &mut self.assigned_spawns {
      if *assigned_spawn == Some(player_id) {
        *assigned_spawn = None
      }
    }
    let _ = scene.despawn(entity);
  }
}
