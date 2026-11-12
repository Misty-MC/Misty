//! Services : toute la logique du launcher, sans interface.
//! Chaque fichier correspond à un service de l'ancienne version C#.

pub mod account;
pub mod app_info;
pub mod catalogue;
pub mod client;
pub mod content;
pub mod curseforge;
pub mod discord;
pub mod export;
pub mod download;
pub mod horizon;
pub mod http;
pub mod instances;
pub mod journal;
pub mod lanceur;
pub mod loc;
pub mod modpack;
pub mod modrinth;
pub mod musique;
pub mod packs;
pub mod partage;
pub mod paths;
pub mod presence;
pub mod pseudos;
pub mod settings;
pub mod shortcut;
pub mod skins;
pub mod storage;
pub mod system_info;
pub mod telemetry;
pub mod texture_categories;
pub mod update;
pub mod version_compat;

/// Fonction de progression (texte affiché à l'utilisateur), utilisable depuis n'importe quel thread.
pub type Progress = std::sync::Arc<dyn Fn(String) + Send + Sync>;

