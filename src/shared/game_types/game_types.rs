/// This file contains miscellaneous helper types to keep things from getting too messy.

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ModelNames {
  Wizard,
  Spectator,
  Dreamstone,
  Bullet,
  Swampeter,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ParticleType {
  Damage,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Enemy {
  Swampeter,
  Alian,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PrefabType {
  Player,
  Enemy,
  Collectible,
  Projectile,
  Scenery,
  SpawnPoint,
  SpectatorPoint,
  ParticleSystem,
  Unknown,
}

impl From<&str> for PrefabType {
  fn from(s: &str) -> Self {
    match s {
      "Wizard" => PrefabType::Player,
      "Swampeter" | "Alian" => PrefabType::Enemy,
      "Dreamstone" => PrefabType::Collectible,
      "Projectile" => PrefabType::Projectile,
      "Scenery" => PrefabType::Scenery,
      "SpawnPoint" => PrefabType::SpawnPoint,
      "SpectatorPoint" => PrefabType::SpectatorPoint,
      "DamageParticle" => PrefabType::ParticleSystem,
      _ => PrefabType::Unknown,
    }
  }
}

#[derive(Debug, Clone)]
pub enum EnemyState {
  Chasing,
  Patrolling,
  Idle,
  Lunging,
  Attacking,
}

pub enum NpcState {
  Patrolling,
  Attacking,
  Walking,
  Idle,
  Talking,
}

impl Default for EnemyState {
  fn default() -> EnemyState {
    EnemyState::Idle
  }
}
