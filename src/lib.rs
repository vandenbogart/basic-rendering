pub mod components;
pub mod component_manager;
pub mod asset_manager;
pub mod renderer;
pub mod loaders;
pub mod window;
pub mod world;
pub mod systems;


use uuid::Uuid;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct EntityHandle(Uuid);
