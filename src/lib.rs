pub mod components;
pub mod component_manager;
pub mod asset_manager;
pub mod renderer;
pub mod loaders;
pub mod window;
pub mod world;
pub mod systems;
pub mod ray;


use uuid::Uuid;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct EntityHandle(Uuid);
